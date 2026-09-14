//! Host health: what this machine and the managed processes are doing.
//!
//! Three things on this page were fabricated. The chart drew a **fixed SVG
//! path** — the same curve on every host at every moment — with only its
//! right-hand endpoint bound to anything, under axis labels reading `-60m …
//! Now` and a caption reading `1h Window · 1m Period` over a product that
//! persisted no host metric at all. The 1h/3h/12h/24h/3d/1w and Period pills
//! were bare `<span>`s with a hover restyle and no handler, and `MonitorQuery`
//! had no time field to receive them. And the watchdog tile was a constant
//! `● Healthy` that never read `WatchdogStatus`, so it stayed green while
//! `WatchdogExhausted` sat in the journal.
//!
//! The readings themselves were always real. What is new is that they are now
//! **kept**: the server holds one collector, the supervision tick samples it on
//! an interval, and the chart plots what was actually recorded. The pills are
//! gone rather than wired, because one hour is the whole of what is retained
//! and a selector offering a week of history nobody has is the same lie in a
//! new shape.

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
};

use crate::{
    core::operations::{
        filter_process_rows, format_bytes, ProcessFilter, ProcessRow, ProcessStateFilter,
    },
    metrics::{HostSample, SAMPLE_INTERVAL},
};

use super::super::{html, pages::metrics_page::collect_snapshot, WebState};

#[derive(Default, serde::Deserialize)]
pub struct MonitorQuery {
    #[serde(default)]
    state: String,
    #[serde(default)]
    high_cpu: String,
    #[serde(default)]
    high_memory: String,
    #[serde(default)]
    q: String,
}

pub async fn monitor(
    State(state): State<WebState>,
    Query(params): Query<MonitorQuery>,
) -> Response {
    let body = match collect_snapshot(&state) {
        Ok(snapshot) => {
            let rows = filter_process_rows(
                &snapshot.node_processes,
                &snapshot.missing_processes,
                &process_filter(&params),
            );
            render_body(&state, &snapshot, &rows, &params)
        }
        Err(error) => html::note(&format!("failed to collect metrics: {error}")),
    };
    Html(html::layout("Host health", "monitor", "", &body)).into_response()
}

fn process_filter(params: &MonitorQuery) -> ProcessFilter {
    let state = match params.state.trim().to_ascii_lowercase().as_str() {
        "observed" => Some(ProcessStateFilter::Observed),
        "missing" => Some(ProcessStateFilter::Missing),
        _ => None,
    };
    ProcessFilter::new(
        state,
        is_on(&params.high_cpu),
        is_on(&params.high_memory),
        params.q.trim(),
    )
}

fn is_on(raw: &str) -> bool {
    matches!(raw.trim().to_ascii_lowercase().as_str(), "on" | "1" | "yes")
}

fn render_body(
    state: &WebState,
    snapshot: &crate::metrics::MetricsSnapshot,
    rows: &[ProcessRow],
    params: &MonitorQuery,
) -> String {
    let system = &snapshot.system;
    let breadcrumb = html::breadcrumb(&[("NeoNexus", "/"), ("Host health", "")]);
    let head = html::page_head(
        "Host health",
        "What this machine and the processes NeoNexus supervises are doing. Chain health — what each node says when asked — is on the node itself.",
        r#"<a class="btn" href="/">Fleet overview</a> <a class="btn" href="/public-metrics" target="_blank">Prometheus scrape</a>"#,
    );

    let cpu_pct = system.cpu_usage_percent.clamp(0.0, 100.0);
    let mem_pct = system.memory_usage_percent.clamp(0.0, 100.0);
    let cpu_bar_color = if cpu_pct > 85.0 {
        "var(--red)"
    } else if cpu_pct > 70.0 {
        "var(--amber)"
    } else {
        "var(--jade)"
    };
    let mem_bar_color = if mem_pct > 85.0 {
        "var(--red)"
    } else if mem_pct > 70.0 {
        "var(--amber)"
    } else {
        "var(--cyan)"
    };

    let cpu_card = format!(
        r#"<div class="card"><div class="stat-label">Host CPU</div><div class="stat-value">{:.1}%</div><div class="aws-metric-gauge"><div class="aws-metric-progress" style="width: {:.1}%; background: {};"></div></div><div class="stat-detail">read {} ago</div></div>"#,
        cpu_pct,
        cpu_pct,
        cpu_bar_color,
        // "Alarm threshold: 85%" named a threshold nothing evaluated. What is
        // worth saying about a reading is how old it is.
        html::escape(&crate::core::node_health::duration_label(captured_age(
            snapshot.captured_at_unix
        ))),
    );
    let mem_card = format!(
        r#"<div class="card"><div class="stat-label">Host memory</div><div class="stat-value">{:.1}%</div><div class="aws-metric-gauge"><div class="aws-metric-progress" style="width: {:.1}%; background: {};"></div></div><div class="stat-detail">{} / {}</div></div>"#,
        mem_pct,
        mem_pct,
        mem_bar_color,
        format_bytes(system.used_memory_bytes),
        format_bytes(system.total_memory_bytes)
    );
    let proc_card = format!(
        r#"<div class="card"><div class="stat-label">Processes on this host</div><div class="stat-value">{}</div><div class="stat-detail">all of them, not only ours</div></div>"#,
        system.process_count
    );
    let watchdog_card = watchdog_card(state);
    let metrics_strip =
        format!(r#"<div class="cards">{cpu_card}{mem_card}{proc_card}{watchdog_card}</div>"#);
    let chart_box = host_chart(&state.metrics().history(), snapshot.captured_at_unix);

    format!(
        r#"{breadcrumb}
{head}
{metrics_strip}
{chart_box}
<h2>Supervised processes</h2>
{filters}
{table}"#,
        breadcrumb = breadcrumb,
        head = head,
        metrics_strip = metrics_strip,
        chart_box = chart_box,
        filters = html::typed_filter_form(
            "/monitor",
            &[],
            &[
                html::FilterControl::Select {
                    label: "Process state",
                    name: "state",
                    selected: &params.state,
                    options: &[
                        ("", "All managed processes"),
                        ("observed", "Observed"),
                        ("missing", "Missing"),
                    ],
                },
                html::FilterControl::Checkbox {
                    label: "High CPU",
                    name: "high_cpu",
                    checked: is_on(&params.high_cpu),
                },
                html::FilterControl::Checkbox {
                    label: "High memory",
                    name: "high_memory",
                    checked: is_on(&params.high_memory),
                },
                html::FilterControl::Search {
                    label: "Search",
                    name: "q",
                    value: &params.q,
                    placeholder: "Node, process, or status",
                },
            ],
        ),
        table = process_table(rows),
    )
}

fn process_table(rows: &[ProcessRow]) -> String {
    if rows.is_empty() {
        return html::note("No managed processes match this filter.");
    }
    let rendered = rows
        .iter()
        .map(|row| match row {
            ProcessRow::Observed(process) => html::row(&[
                html::cell(&process.node_name),
                html::raw_cell(&html::status_badge("Running")),
                html::cell(&process.status),
                html::cell(&process.pid.to_string()),
                html::cell(&percent_label(process.cpu_usage_percent)),
                html::cell(&format_bytes(process.memory_bytes)),
                html::cell(&format_bytes(process.virtual_memory_bytes)),
                html::cell(&format_uptime(process.run_time_seconds)),
            ]),
            ProcessRow::Missing(process) => html::row(&[
                html::cell(&process.node_name),
                html::raw_cell(&html::status_badge("Error")),
                html::cell("process not found"),
                html::cell(&process.pid.to_string()),
                html::cell("—"),
                html::cell("—"),
                html::cell("—"),
                html::cell("—"),
            ]),
        })
        .collect::<Vec<_>>();
    html::table(
        &[
            "Node", "State", "Detail", "PID", "CPU", "Memory", "Virtual", "Uptime",
        ],
        &rendered,
    )
}

fn percent_label(percent: f32) -> String {
    format!("{:.1}%", percent)
}

fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else if minutes > 0 {
        format!("{minutes}m {}s", seconds % 60)
    } else {
        format!("{seconds}s")
    }
}

fn captured_age(captured_at_unix: u64) -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(captured_at_unix);
    now.saturating_sub(captured_at_unix)
}

/// The watchdog tile, read from the policy and the restart ledger.
///
/// It was a constant `● Healthy` that never consulted `WatchdogStatus`, so it
/// stayed green with automatic restart switched off entirely and stayed green
/// while a node sat `WatchdogExhausted` in the journal — the two moments an
/// operator most needs it not to be.
fn watchdog_card(state: &WebState) -> String {
    let Ok(policy) = state.workspace.load_watchdog_policy() else {
        return card(
            "Automatic restart",
            "unreadable",
            "the policy could not be read",
            "var(--amber)",
        );
    };
    if !policy.enabled {
        return card(
            "Automatic restart",
            "Off",
            "a node that exits stays down",
            "var(--idle)",
        );
    }
    let exhausted = state
        .workspace
        .nodes_the_watchdog_gave_up_on()
        .unwrap_or_default();
    if exhausted.is_empty() {
        card(
            "Automatic restart",
            "On",
            &format!("up to {} attempts per node", policy.max_restart_attempts),
            "var(--jade)",
        )
    } else {
        card(
            "Automatic restart",
            "Gave up",
            &match exhausted.as_slice() {
                [node] => format!("{} is past its retry budget", node.name),
                nodes => format!("{} nodes are past their retry budget", nodes.len()),
            },
            "var(--red)",
        )
    }
}

fn card(label: &str, value: &str, detail: &str, colour: &str) -> String {
    format!(
        r#"<div class="card"><div class="stat-label">{label}</div><div class="stat-value" style="color: {colour};">{value}</div><div class="stat-detail">{detail}</div></div>"#,
        label = html::escape(label),
        value = html::escape(value),
        detail = html::escape(detail),
    )
}

/// The recorded readings, plotted.
///
/// Every point is a sample the supervision tick actually took. Where the ring
/// is not yet full the axis says how much it covers rather than claiming an
/// hour, because a page that labels four minutes of data `-60m` is the same
/// fabrication the fixed path was.
fn host_chart(history: &[HostSample], captured_at_unix: u64) -> String {
    if history.len() < 2 {
        return html::note(&format!(
            "Only {} reading has been taken so far. The host is sampled every {}s; \
             a line needs at least two points.",
            history.len(),
            SAMPLE_INTERVAL.as_secs()
        ));
    }
    let oldest = history.first().map_or(captured_at_unix, |s| s.at_unix);
    let newest = history.last().map_or(captured_at_unix, |s| s.at_unix);
    let span = newest.saturating_sub(oldest).max(1);

    let point = |sample: &HostSample, value: f32| {
        let x = (sample.at_unix.saturating_sub(oldest) as f64 / span as f64) * 800.0;
        let y = 115.0 - f64::from(value.clamp(0.0, 100.0)) * 1.05;
        format!("{x:.1},{y:.1}")
    };
    let cpu_line = history
        .iter()
        .map(|sample| point(sample, sample.cpu_usage_percent))
        .collect::<Vec<_>>()
        .join(" ");
    let memory_line = history
        .iter()
        .map(|sample| point(sample, sample.memory_usage_percent))
        .collect::<Vec<_>>()
        .join(" ");

    let covered = crate::core::node_health::duration_label(span);
    format!(
        r##"<div class="aws-chart-box" style="margin: 18px 0;">
            <div class="aws-chart-header">
                <div style="display: flex; align-items: center; gap: 14px; flex-wrap: wrap;">
                    <strong style="color: #fff; font-size: 13px;">Host load, as recorded</strong>
                    <span style="font-size: 11px; color: var(--jade); font-weight: 600;">● CPU</span>
                    <span style="font-size: 11px; color: var(--cyan); font-weight: 600;">● Memory</span>
                </div>
                <div class="mono muted" style="font-size: 11px;">{points} readings · {covered} · one every {interval}s</div>
            </div>
            <svg class="aws-telemetry-chart aws-sparkline" viewBox="0 0 800 120" preserveAspectRatio="none" style="height: 100px;" role="img" aria-label="Host CPU and memory over the recorded window">
                <line x1="0" y1="15" x2="800" y2="15" stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
                <line x1="0" y1="45" x2="800" y2="45" stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
                <line x1="0" y1="75" x2="800" y2="75" stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
                <line x1="0" y1="105" x2="800" y2="105" stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
                <polyline points="{memory_line}" fill="none" stroke="#38bdf8" stroke-width="2"/>
                <polyline points="{cpu_line}" fill="none" stroke="#3bd184" stroke-width="2"/>
            </svg>
            <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--faint); font-family: var(--mono); margin-top: 4px;">
                <span>-{covered}</span><span>now</span>
            </div>
        </div>"##,
        points = history.len(),
        covered = html::escape(&covered),
        interval = SAMPLE_INTERVAL.as_secs(),
    )
}
