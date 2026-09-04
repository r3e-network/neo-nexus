//! Host resource observations and policy form. Collection runs in the engine.
use super::super::{html, WebState};
use crate::{
    metrics::format_bytes,
    repository::Repository,
    resource_health::{ResourcePolicy, ResourceReport},
};
use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

pub fn summary(repository: &Repository) -> anyhow::Result<String> {
    let policy = repository.load_resource_policy()?;
    let report = repository.latest_resource_report()?;
    let now = now();
    let fresh = policy.enabled
        && report
            .as_ref()
            .is_some_and(|report| report.policy == policy && report.is_fresh(now));
    let notice = if !policy.enabled {
        "Resource monitor is disabled."
    } else if !fresh {
        "No fresh resource sample. Keep the workbench running and inspect storage access if this persists."
    } else {
        "Critical pressure alerts immediately. Warnings and recovery require two consecutive samples; no process is stopped and no files are removed."
    };
    let rows = report
        .as_ref()
        .map(|report| {
            report
                .readings
                .iter()
                .map(|reading| {
                    html::row(&[
                        html::cell(&reading.label),
                        html::cell(if fresh {
                            reading.status.label()
                        } else {
                            "Stale / unavailable"
                        }),
                        html::cell(
                            &reading
                                .available_bytes
                                .map_or("Unknown".into(), format_bytes),
                        ),
                        html::cell(
                            &reading
                                .capacity_bytes
                                .map_or("Unknown".into(), format_bytes),
                        ),
                        html::cell(&reading.message),
                    ])
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(format!("<h2>Storage and memory</h2>{}<p><a href=\"/settings#resources\">Resource thresholds and external storage directories</a></p>{}{}",
        html::note(notice),
        report.as_ref().map_or(String::new(), |report|format!("<p>Last sample: {} seconds ago.</p>",now.saturating_sub(report.checked_at_unix))),
        html::table(&["Resource", "Observed status", "Available", "Capacity", "Detail"],&rows)))
}

pub fn form(repository: &Repository) -> anyhow::Result<String> {
    let policy = repository.load_resource_policy()?;
    // Keep policy labels identical to the other settings forms.
    let enabled = html::choice_field(
        "Status",
        "enabled",
        &super::settings::enabled_choices(),
        super::settings::enabled_label(policy.enabled),
    );
    let paths = policy
        .storage_paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        r#"<h2 id="resources">Host resource alerts</h2>
<p>The workspace volume is always checked. Add external chain-data directories below. Thresholds describe <strong>available</strong> capacity, not used capacity. Missing mounts are reported as unavailable.</p>
<form method="post" action="/settings/resources">{enabled}{interval}{disk_warning}{disk_critical}{memory_warning}{memory_critical}
<label>Additional storage directories (one absolute path per line)<textarea name="storage_paths" rows="4">{paths}</textarea></label><button>Save resource policy</button></form>"#,
        interval = html::text_field(
            "Interval (10–3600 seconds)",
            "interval_seconds",
            &policy.interval_seconds.to_string()
        ),
        disk_warning = html::text_field(
            "Disk warning (MiB available)",
            "disk_warning_mib",
            &policy.disk_warning_mib.to_string()
        ),
        disk_critical = html::text_field(
            "Disk critical (MiB available)",
            "disk_critical_mib",
            &policy.disk_critical_mib.to_string()
        ),
        memory_warning = html::text_field(
            "Memory warning (% available)",
            "memory_warning_percent",
            &policy.memory_warning_percent.to_string()
        ),
        memory_critical = html::text_field(
            "Memory critical (% available)",
            "memory_critical_percent",
            &policy.memory_critical_percent.to_string()
        ),
        paths = html::escape(&paths)
    ))
}

#[derive(Deserialize)]
pub struct ResourceForm {
    enabled: String,
    interval_seconds: u64,
    disk_warning_mib: u64,
    disk_critical_mib: u64,
    memory_warning_percent: u8,
    memory_critical_percent: u8,
    #[serde(default)]
    storage_paths: String,
}

pub async fn save(State(state): State<WebState>, Form(form): Form<ResourceForm>) -> Response {
    let policy = ResourcePolicy {
        enabled: super::settings::choice_is_enabled(&form.enabled),
        interval_seconds: form.interval_seconds,
        disk_warning_mib: form.disk_warning_mib,
        disk_critical_mib: form.disk_critical_mib,
        memory_warning_percent: form.memory_warning_percent,
        memory_critical_percent: form.memory_critical_percent,
        storage_paths: form
            .storage_paths
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(Into::into)
            .collect(),
    };
    let message = state
        .repository
        .save_resource_policy(&policy)
        .err()
        .map_or("Resource policy saved".into(), |error| error.to_string());
    Redirect::to(&format!("/settings{}", html::flash_query(&message))).into_response()
}

pub fn prometheus(repository: &Repository) -> anyhow::Result<String> {
    let policy = repository.load_resource_policy()?;
    let report: Option<ResourceReport> = repository
        .latest_resource_report()?
        .filter(|report| report.policy == policy);
    Ok(crate::resource_health::prometheus(
        report.as_ref(),
        policy.enabled,
        now(),
    ))
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}
