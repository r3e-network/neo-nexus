use super::*;
use crate::metrics::{MissingProcessMetric, NodeProcessMetrics, ProcessRow, ProcessStateFilter};

#[test]
fn monitor_process_filters_select_visible_process_rows() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    app.metrics_snapshot.node_processes = vec![
        observed_process("validator-hot", "Validator Hot", 82.0, 900),
        observed_process("rpc-cold", "RPC Cold", 12.0, 128),
    ];
    app.metrics_snapshot.missing_processes = vec![missing_process("stale-rpc", "Stale RPC")];

    app.monitor_process_state_filter = Some(ProcessStateFilter::Observed);
    app.monitor_process_high_cpu_filter = true;
    app.monitor_process_query = "validator".to_string();
    app.selected_monitor_process = Some("stale-rpc".to_string());
    app.monitor_process_page = 4;

    let visible = app.filtered_monitor_process_rows();
    assert_eq!(visible.len(), 1);
    assert!(
        matches!(&visible[0], ProcessRow::Observed(process) if process.node_id == "validator-hot")
    );

    app.ensure_valid_monitor_process_selection();
    assert_eq!(
        app.selected_monitor_process.as_deref(),
        Some("validator-hot")
    );
    assert_eq!(app.monitor_process_page, 0);

    app.monitor_process_state_filter = Some(ProcessStateFilter::Missing);
    app.monitor_process_high_cpu_filter = false;
    app.monitor_process_query = "stale".to_string();
    let missing = app.filtered_monitor_process_rows();
    assert_eq!(missing.len(), 1);
    assert!(matches!(&missing[0], ProcessRow::Missing(process) if process.node_id == "stale-rpc"));

    Ok(())
}

#[test]
fn monitor_process_focus_missing_resets_filters_and_selects_missing_row() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    app.metrics_snapshot.node_processes = vec![observed_process("rpc-live", "RPC Live", 12.0, 128)];
    app.metrics_snapshot.missing_processes = vec![
        missing_process("validator-gone", "Validator Gone"),
        missing_process("seed-gone", "Seed Gone"),
    ];
    app.monitor_process_state_filter = Some(ProcessStateFilter::Observed);
    app.monitor_process_high_cpu_filter = true;
    app.monitor_process_high_memory_filter = true;
    app.monitor_process_query = "rpc".to_string();
    app.monitor_process_page = 3;

    app.focus_missing_processes();

    assert_eq!(
        app.monitor_process_state_filter,
        Some(ProcessStateFilter::Missing)
    );
    assert!(!app.monitor_process_high_cpu_filter);
    assert!(!app.monitor_process_high_memory_filter);
    assert!(app.monitor_process_query.is_empty());
    assert_eq!(app.monitor_process_page, 0);
    assert_eq!(
        app.selected_monitor_process.as_deref(),
        Some("validator-gone")
    );
    assert_eq!(app.fleet.selected_node.as_deref(), Some("validator-gone"));
    assert_eq!(app.filtered_monitor_process_rows().len(), 2);

    app.clear_monitor_process_filters();

    assert!(!app.has_active_monitor_process_filter());
    assert_eq!(app.filtered_monitor_process_rows().len(), 3);

    Ok(())
}

fn observed_process(id: &str, name: &str, cpu: f32, memory_mib: u64) -> NodeProcessMetrics {
    NodeProcessMetrics {
        node_id: id.to_string(),
        node_name: name.to_string(),
        pid: 10_000,
        cpu_usage_percent: cpu,
        memory_bytes: memory_mib * 1024 * 1024,
        virtual_memory_bytes: memory_mib * 2 * 1024 * 1024,
        run_time_seconds: 120,
        status: "run".to_string(),
    }
}

fn missing_process(id: &str, name: &str) -> MissingProcessMetric {
    MissingProcessMetric {
        node_id: id.to_string(),
        node_name: name.to_string(),
        pid: 11_000,
    }
}
