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
    let body = match collect_snapshot(&state.repository) {
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
    Html(html::layout("Monitor", "monitor", "", &body)).into_response()
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
    format!(
        r#"<h1>Monitor</h1>
{tiles}
{filters}
<h2>Managed processes</h2>
{table}"#,
        tiles = html::cards(&[
            ("Host CPU", percent_label(system.cpu_usage_percent)),
            (
                "Host memory",
                format!("{:.0}%", system.memory_usage_percent)
            ),
            ("Used", format_bytes(system.used_memory_bytes)),
            ("Total", format_bytes(system.total_memory_bytes)),
            ("Processes", system.process_count.to_string()),
            (
                "Captured",
                format!("{}s ago", captured_age(snapshot.captured_at_unix))
            ),
        ]),
        filters = html::filter_form(
            "/monitor",
            &[
                ("state", &params.state),
                ("high_cpu", &params.high_cpu),
                ("high_memory", &params.high_memory),
                ("q", &params.q),
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
