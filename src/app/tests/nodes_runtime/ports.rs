use super::super::*;
use super::neo_rs_app_node;

#[test]
fn draft_auto_ports_skip_existing_node_ports() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    app.repository.create_node(neo_rs_app_node(
        "existing port owner",
        30_332,
        30_333,
        Some(30_334),
    ))?;
    app.reload_nodes();
    app.draft.rpc_port = "30332".to_string();
    app.draft.p2p_port = "30333".to_string();
    app.draft.ws_port = "30334".to_string();

    app.auto_assign_draft_ports();

    let rpc_port = app.draft.rpc_port.parse::<u16>()?;
    assert_ne!(rpc_port, 30_332);
    assert_eq!(app.draft.p2p_port.parse::<u16>()?, rpc_port + 1);
    assert_eq!(app.draft.ws_port.parse::<u16>()?, rpc_port + 2);
    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("Draft ports assigned")));

    Ok(())
}

#[test]
fn selected_node_fix_ports_persists_available_block_and_audits() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    app.repository
        .create_node(neo_rs_app_node("port owner", 31_332, 31_333, Some(31_334)))?;
    let target =
        app.repository
            .create_node(neo_rs_app_node("needs ports", 31_332, 31_333, Some(31_334)))?;
    app.reload_nodes();
    app.selected_node = Some(target.id.clone());

    app.assign_available_ports_to_selected_node();

    let updated = app
        .repository
        .list_nodes()?
        .into_iter()
        .find(|node| node.id == target.id)
        .ok_or_else(|| anyhow::anyhow!("target node should still exist"))?;
    assert_ne!(updated.rpc_port, 31_332);
    assert_eq!(updated.p2p_port, updated.rpc_port + 1);
    assert_eq!(updated.ws_port, Some(updated.rpc_port + 2));
    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("ports assigned")));
    let events =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "node-ports-assigned", 10))?;
    assert!(events.iter().any(|event| {
        event.kind == EventKind::NodePortsAssigned
            && event.node_id.as_deref() == Some(target.id.as_str())
    }));

    Ok(())
}
