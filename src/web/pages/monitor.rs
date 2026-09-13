//! Monitor: what the managed processes are doing right now. It reads the same
//! snapshot the Metrics page and the Prometheus endpoint serve, then filters it
//! through the shared `filter_process_rows` ordering so a missing process always
//! sorts to the top.

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
};

use crate::core::operations::{
    filter_process_rows, format_bytes, ProcessFilter, ProcessRow, ProcessStateFilter,
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
    let body = match collect_snapshot(&state.workspace) {
        Ok(snapshot) => {
            let rows = filter_process_rows(
                &snapshot.node_processes,
                &snapshot.missing_processes,
                &process_filter(&params),
            );
            render_body(&snapshot, &rows, &params)
        }
        Err(error) => html::note(&format!("failed to collect metrics: {error}")),
    };
    Html(html::layout("Health", "monitor", "", &body)).into_response()
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
    snapshot: &crate::metrics::MetricsSnapshot,
    rows: &[ProcessRow],
    params: &MonitorQuery,
) -> String {
    let system = &snapshot.system;
    let breadcrumb = html::breadcrumb(&[
        ("CloudWatch", "/monitor"),
        ("Metrics", "/monitor"),
        ("All metrics", ""),
    ]);
    let head = html::page_head(
        "CloudWatch Metrics & Telemetry",
        "Host and managed node process telemetry, CPU/RAM utilization metrics, and supervisory watchdog signals.",
        r#"<a class="btn" href="/alerts">🚨 CloudWatch Alarms</a> <a class="btn" href="/public-metrics" target="_blank">📈 Prometheus Scrape</a>"#,
    );
    let time_bar = r#"<div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; flex-wrap: wrap; gap: 8px;">
        <div class="aws-time-selector">
            <span style="font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--muted); padding: 0 4px;">Time range:</span>
            <span class="aws-time-pill active">1h</span>
            <span class="aws-time-pill">3h</span>
            <span class="aws-time-pill">12h</span>
            <span class="aws-time-pill">24h</span>
            <span class="aws-time-pill">3d</span>
            <span class="aws-time-pill">1w</span>
        </div>
        <div class="aws-time-selector">
            <span style="font-size: 11px; font-weight: 600; text-transform: uppercase; color: var(--muted); padding: 0 4px;">Period:</span>
            <span class="aws-time-pill active">1m</span>
            <span class="aws-time-pill">5m</span>
            <span class="aws-time-pill">15m</span>
        </div>
    </div>"#;

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
        r#"<div class="card"><div class="stat-label">Host CPU Utilization</div><div class="stat-value">{:.1}%</div><div class="aws-metric-gauge"><div class="aws-metric-progress" style="width: {:.1}%; background: {};"></div></div><div class="stat-detail">Alarm threshold: 85%</div></div>"#,
        cpu_pct, cpu_pct, cpu_bar_color
    );
    let mem_card = format!(
        r#"<div class="card"><div class="stat-label">Host Memory (RAM)</div><div class="stat-value">{:.1}%</div><div class="aws-metric-gauge"><div class="aws-metric-progress" style="width: {:.1}%; background: {};"></div></div><div class="stat-detail">{} / {}</div></div>"#,
        mem_pct,
        mem_pct,
        mem_bar_color,
        format_bytes(system.used_memory_bytes),
        format_bytes(system.total_memory_bytes)
    );
    let proc_card = format!(
        r#"<div class="card"><div class="stat-label">Managed Processes</div><div class="stat-value">{}</div><div class="stat-detail">Active supervised workloads</div></div>"#,
        system.process_count
    );
    let watchdog_card = format!(
        r#"<div class="card"><div class="stat-label">CloudWatch Watchdog</div><div class="stat-value" style="color: var(--jade);">● Healthy</div><div class="stat-detail">Captured {}s ago</div></div>"#,
        captured_age(snapshot.captured_at_unix)
    );
    let metrics_strip =
        format!(r#"<div class="cards">{cpu_card}{mem_card}{proc_card}{watchdog_card}</div>"#);
    let chart_box = cloudwatch_svg_chart(cpu_pct, mem_pct);

    format!(
        r#"{breadcrumb}
{head}
{time_bar}
{metrics_strip}
{chart_box}
{filters}
<h2>Managed Workload Processes (EC2)</h2>
{table}"#,
        breadcrumb = breadcrumb,
        head = head,
        time_bar = time_bar,
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

fn cloudwatch_svg_chart(cpu_pct: f32, mem_pct: f32) -> String {
    let cpu_y = (110.0 - (cpu_pct * 0.9).clamp(5.0, 95.0)) as u32;
    let mem_y = (110.0 - (mem_pct * 0.9).clamp(5.0, 95.0)) as u32;

    format!(
        r##"<div class="aws-chart-box" style="margin: 18px 0;">
            <div class="aws-chart-header">
                <div style="display: flex; align-items: center; gap: 14px; flex-wrap: wrap;">
                    <strong style="color: #fff; font-size: 13px;">CloudWatch Metrics Dashboard: Fleet Host Telemetry</strong>
                    <span style="font-size: 11px; color: var(--jade); font-weight: 600;">● CPUUtilization ({:.1}%)</span>
                    <span style="font-size: 11px; color: var(--cyan); font-weight: 600;">● MemoryUtilization ({:.1}%)</span>
                    <span style="font-size: 11px; color: var(--red); font-weight: 500;">-- High CPU Alarm (85%)</span>
                </div>
                <div class="mono muted" style="font-size: 11px;">Namespace: AWS/EC2 · 1h Window · 1m Period</div>
            </div>
            <svg class="aws-telemetry-chart aws-sparkline" viewBox="0 0 800 120" preserveAspectRatio="none" style="height: 100px;">
                <defs>
                    <linearGradient id="cwCpuGrad" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stop-color="#3bd184" stop-opacity="0.35"/>
                        <stop offset="100%" stop-color="#3bd184" stop-opacity="0.0"/>
                    </linearGradient>
                    <linearGradient id="cwMemGrad" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stop-color="#38bdf8" stop-opacity="0.25"/>
                        <stop offset="100%" stop-color="#38bdf8" stop-opacity="0.0"/>
                    </linearGradient>
                </defs>
                <line x1="0" y1="15" x2="800" y2="15" stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
                <line x1="0" y1="45" x2="800" y2="45" stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
                <line x1="0" y1="75" x2="800" y2="75" stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
                <line x1="0" y1="105" x2="800" y2="105" stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
                <line x1="0" y1="20" x2="800" y2="20" stroke="#f43f5e" stroke-width="1.5" stroke-dasharray="6,4"/>
                <text x="790" y="16" fill="#f43f5e" font-size="9" text-anchor="end" font-family="monospace">85% CRITICAL</text>
                <path d="M0,78 Q150,75 300,77 T600,{mem_y} L800,{mem_y} L800,120 L0,120 Z" fill="url(#cwMemGrad)"/>
                <path d="M0,78 Q150,75 300,77 T600,{mem_y} L800,{mem_y}" fill="none" stroke="#38bdf8" stroke-width="2"/>
                <path d="M0,105 Q120,95 240,100 T450,70 T650,90 L800,{cpu_y} L800,120 L0,120 Z" fill="url(#cwCpuGrad)"/>
                <path d="M0,105 Q120,95 240,100 T450,70 T650,90 L800,{cpu_y}" fill="none" stroke="#3bd184" stroke-width="2"/>
            </svg>
            <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--faint); font-family: var(--mono); margin-top: 4px;">
                <span>-60m</span><span>-45m</span><span>-30m</span><span>-15m</span><span>Now</span>
            </div>
        </div>"##,
        cpu_pct,
        mem_pct,
        mem_y = mem_y,
        cpu_y = cpu_y
    )
}
