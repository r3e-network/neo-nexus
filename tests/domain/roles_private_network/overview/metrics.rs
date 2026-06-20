use crate::*;

#[test]
fn dashboard_summary_counts_node_states_and_plugin_readiness() {
    let repo = create_repo();
    let running = create_node(&repo, "running", NodeType::NeoGo);
    let stopped = create_node(&repo, "stopped", NodeType::NeoCli);
    repo.update_node_status(&running, NodeStatus::Running, Some(42))
        .unwrap();
    repo.set_plugin_enabled(&stopped, PluginId::RpcServer, true)
        .unwrap();

    let summary = DashboardSummary::load(&repo).unwrap();

    assert_eq!(summary.total_nodes, 2);
    assert_eq!(summary.running_nodes, 1);
    assert_eq!(summary.stopped_nodes, 1);
    assert_eq!(summary.rpc_enabled_nodes, 1);
    assert_eq!(summary.health_percent, 50);
}

#[test]
fn metrics_format_pressure_and_node_totals_are_stable() {
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1536), "1.5 KiB");
    assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MiB");
    assert_eq!(
        ResourcePressure::from_percent(40.0),
        ResourcePressure::Nominal
    );
    assert_eq!(
        ResourcePressure::from_percent(80.0),
        ResourcePressure::Elevated
    );
    assert_eq!(
        ResourcePressure::from_percent(95.0),
        ResourcePressure::Critical
    );

    let snapshot = MetricsSnapshot {
        captured_at_unix: 42,
        system: SystemMetrics {
            cpu_usage_percent: 12.0,
            total_memory_bytes: 100,
            used_memory_bytes: 82,
            available_memory_bytes: 18,
            memory_usage_percent: 82.0,
            process_count: 10,
        },
        node_processes: vec![
            NodeProcessMetrics {
                node_id: "node-a".to_string(),
                node_name: "alpha".to_string(),
                pid: 100,
                cpu_usage_percent: 3.5,
                memory_bytes: 2048,
                virtual_memory_bytes: 4096,
                run_time_seconds: 9,
                status: "run".to_string(),
            },
            NodeProcessMetrics {
                node_id: "node-b".to_string(),
                node_name: "beta".to_string(),
                pid: 101,
                cpu_usage_percent: 1.5,
                memory_bytes: 1024,
                virtual_memory_bytes: 4096,
                run_time_seconds: 5,
                status: "sleep".to_string(),
            },
        ],
        missing_processes: vec![MissingProcessMetric {
            node_id: "node-c".to_string(),
            node_name: "gamma".to_string(),
            pid: 102,
        }],
    };

    assert_eq!(snapshot.node_process("node-a").unwrap().pid, 100);
    assert_eq!(snapshot.total_node_memory_bytes(), 3072);
    assert!((snapshot.total_node_cpu_usage_percent() - 5.0).abs() < f32::EPSILON);
    assert_eq!(
        snapshot.system.memory_pressure(),
        ResourcePressure::Elevated
    );
    assert_eq!(snapshot.missing_processes.len(), 1);

    let quiet_snapshot = MetricsSnapshot {
        captured_at_unix: 43,
        system: SystemMetrics {
            cpu_usage_percent: 0.0,
            total_memory_bytes: 0,
            used_memory_bytes: 0,
            available_memory_bytes: 0,
            memory_usage_percent: 0.0,
            process_count: 0,
        },
        node_processes: vec![NodeProcessMetrics {
            node_id: "node-d".to_string(),
            node_name: "delta".to_string(),
            pid: 103,
            cpu_usage_percent: -0.01,
            memory_bytes: 0,
            virtual_memory_bytes: 0,
            run_time_seconds: 0,
            status: "idle".to_string(),
        }],
        missing_processes: Vec::new(),
    };

    assert_eq!(quiet_snapshot.total_node_cpu_usage_percent(), 0.0);
    assert!(quiet_snapshot
        .to_cli_text()
        .contains("node-cpu-total: 0.0%"));
    assert!(!quiet_snapshot.to_cli_text().contains("-0.0%"));
}

#[test]
fn metrics_prometheus_text_escapes_labels_and_reports_health() {
    let snapshot = MetricsSnapshot {
        captured_at_unix: 1_800_000_000,
        system: SystemMetrics {
            cpu_usage_percent: 12.5,
            total_memory_bytes: 100,
            used_memory_bytes: 64,
            available_memory_bytes: 36,
            memory_usage_percent: 64.25,
            process_count: 42,
        },
        node_processes: vec![NodeProcessMetrics {
            node_id: "node-a".to_string(),
            node_name: "alpha \"one\"".to_string(),
            pid: 100,
            cpu_usage_percent: 3.5,
            memory_bytes: 2048,
            virtual_memory_bytes: 4096,
            run_time_seconds: 9,
            status: "run".to_string(),
        }],
        missing_processes: vec![MissingProcessMetric {
            node_id: "node-c".to_string(),
            node_name: "gamma\nline\\tail".to_string(),
            pid: 102,
        }],
    };

    let text = snapshot.to_prometheus_text();

    assert!(text.contains("# HELP neonexus_workspace_status"));
    assert!(text.contains("neonexus_workspace_status 0\n"));
    assert!(text.contains("neonexus_system_cpu_usage_percent 12.5\n"));
    assert!(text.contains("neonexus_system_memory_usage_percent 64.25\n"));
    assert!(text.contains("neonexus_system_processes 42\n"));
    assert!(text.contains(
        "neonexus_node_process_cpu_usage_percent{node_id=\"node-a\",node_name=\"alpha \\\"one\\\"\",pid=\"100\",status=\"run\"} 3.5\n"
    ));
    assert!(text.contains(
        "neonexus_node_missing_process{node_id=\"node-c\",node_name=\"gamma\\nline\\\\tail\",pid=\"102\"} 1\n"
    ));
}
