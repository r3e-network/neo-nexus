use super::*;

#[test]
fn filter_process_rows_prioritizes_missing_then_resource_pressure() {
    let rows = filter_process_rows(
        &[
            observed("low", "Low CPU", 10.0, 256),
            observed("hot", "Hot CPU", 76.0, 768),
        ],
        &[missing("gone", "Gone PID")],
        &ProcessFilter::default(),
    );

    assert_eq!(rows[0].node_id(), "gone");
    assert_eq!(rows[1].node_id(), "hot");
    assert_eq!(rows[2].node_id(), "low");
}

#[test]
fn filter_process_rows_applies_state_pressure_and_query_filters() {
    let observed_rows = [
        observed("hot-rpc", "RPC Hot", 88.0, 1024),
        observed("cold", "Cold Node", 4.0, 128),
    ];
    let missing_rows = [missing("missing-rpc", "RPC Missing")];

    let rows = filter_process_rows(
        &observed_rows,
        &missing_rows,
        &ProcessFilter::new(Some(ProcessStateFilter::Observed), true, true, "rpc"),
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].node_id(), "hot-rpc");

    let missing = filter_process_rows(
        &observed_rows,
        &missing_rows,
        &ProcessFilter::new(Some(ProcessStateFilter::Missing), false, false, "rpc"),
    );
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].node_id(), "missing-rpc");
}

fn observed(id: &str, name: &str, cpu: f32, memory_mib: u64) -> NodeProcessMetrics {
    NodeProcessMetrics {
        node_id: id.to_string(),
        node_name: name.to_string(),
        pid: 100,
        cpu_usage_percent: cpu,
        memory_bytes: memory_mib * 1024 * 1024,
        virtual_memory_bytes: memory_mib * 2 * 1024 * 1024,
        run_time_seconds: 60,
        status: "run".to_string(),
    }
}

fn missing(id: &str, name: &str) -> MissingProcessMetric {
    MissingProcessMetric {
        node_id: id.to_string(),
        node_name: name.to_string(),
        pid: 200,
    }
}
