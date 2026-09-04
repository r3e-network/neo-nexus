use super::{ResourceReport, ResourceStatus};

/// Uses indexed labels rather than filesystem paths. Unknown or stale samples
/// have no capacity gauges, so missing telemetry cannot masquerade as zero.
pub fn prometheus(report: Option<&ResourceReport>, enabled: bool, now: u64) -> String {
    let mut output = format!("# HELP neonexus_resource_monitor_enabled Whether host resource sampling is enabled.\n# TYPE neonexus_resource_monitor_enabled gauge\nneonexus_resource_monitor_enabled {}\n", u8::from(enabled));
    let fresh = enabled && report.is_some_and(|report| report.is_fresh(now));
    output.push_str(&format!("# HELP neonexus_resource_sample_fresh Whether a current resource observation is available.\n# TYPE neonexus_resource_sample_fresh gauge\nneonexus_resource_sample_fresh {}\n",u8::from(fresh)));
    let Some(report) = report else {
        return output;
    };
    output.push_str(&format!("# HELP neonexus_resource_checked_at_seconds Last sample Unix time.\n# TYPE neonexus_resource_checked_at_seconds gauge\nneonexus_resource_checked_at_seconds {}\n",report.checked_at_unix));
    if !fresh {
        return output;
    }
    output.push_str("# HELP neonexus_resource_available_bytes Available memory or storage bytes.\n# TYPE neonexus_resource_available_bytes gauge\n# HELP neonexus_resource_capacity_bytes Observed capacity bytes.\n# TYPE neonexus_resource_capacity_bytes gauge\n# HELP neonexus_resource_pressure Current pressure: healthy=0 warning=1 critical=2 unknown=3.\n# TYPE neonexus_resource_pressure gauge\n");
    for (index, reading) in report.readings.iter().enumerate() {
        let kind = if reading.id == "memory" {
            "memory"
        } else {
            "disk"
        };
        let labels = format!("kind=\"{kind}\",resource=\"{index}\"");
        output.push_str(&format!(
            "neonexus_resource_pressure{{{labels}}} {}\n",
            match reading.status {
                ResourceStatus::Healthy => 0,
                ResourceStatus::Warning => 1,
                ResourceStatus::Critical => 2,
                ResourceStatus::Unknown => 3,
            }
        ));
        if let (Some(available), Some(total)) = (reading.available_bytes, reading.capacity_bytes) {
            output.push_str(&format!("neonexus_resource_available_bytes{{{labels}}} {available}\nneonexus_resource_capacity_bytes{{{labels}}} {total}\n"));
        }
    }
    output
}
