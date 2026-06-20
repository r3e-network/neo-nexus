use super::*;

pub(in crate::private_network) fn check_nodes(
    checks: &mut Vec<LaunchPackValidationCheck>,
    root_path: &Path,
    manifest: &DeploymentManifest,
) {
    if manifest.nodes.is_empty() {
        add_check(
            checks,
            "nodes",
            "count",
            LaunchPackValidationStatus::Fail,
            "manifest has no nodes".to_string(),
        );
        return;
    }

    add_check(
        checks,
        "nodes",
        "count",
        LaunchPackValidationStatus::Pass,
        format!("{} nodes", manifest.nodes.len()),
    );

    let mut ports = BTreeMap::<u16, Vec<String>>::new();
    for node in &manifest.nodes {
        let binary_path = resolve_launch_pack_reference(root_path, &node.binary_path);
        add_file_check(
            checks,
            "nodes",
            format!("{} binary", node.name),
            &binary_path,
        );

        let config_path = resolve_launch_pack_reference(root_path, &node.config_path);
        add_file_check(
            checks,
            "nodes",
            format!("{} config", node.name),
            &config_path,
        );
        add_config_integrity_check(checks, node, &config_path);

        let working_dir = resolve_launch_pack_reference(root_path, &node.working_dir);
        add_dir_check(
            checks,
            "nodes",
            format!("{} workdir", node.name),
            &working_dir,
        );

        collect_port(&mut ports, node.rpc_port, format!("{} RPC", node.name));
        collect_port(&mut ports, node.p2p_port, format!("{} P2P", node.name));
        if let Some(ws_port) = node.ws_port {
            collect_port(&mut ports, ws_port, format!("{} WS", node.name));
        }
    }

    for (port, labels) in ports {
        let status = if labels.len() == 1 {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        };
        add_check(checks, "ports", port.to_string(), status, labels.join(", "));
    }
}

fn add_config_integrity_check(
    checks: &mut Vec<LaunchPackValidationCheck>,
    node: &DeploymentNodeManifest,
    config_path: &Path,
) {
    let expected = match normalize_sha256(&node.config_sha256) {
        Ok(expected) => expected,
        Err(_) => {
            add_check(
                checks,
                "config-integrity",
                format!("{} config sha256", node.name),
                LaunchPackValidationStatus::Fail,
                "manifest config_sha256 is missing or invalid".to_string(),
            );
            return;
        }
    };

    let (actual, bytes) = match sha256_file(config_path) {
        Ok(result) => result,
        Err(error) => {
            add_check(
                checks,
                "config-integrity",
                format!("{} config sha256", node.name),
                LaunchPackValidationStatus::Fail,
                format!("unable to hash config: {error}"),
            );
            return;
        }
    };

    let status = if actual == expected {
        LaunchPackValidationStatus::Pass
    } else {
        LaunchPackValidationStatus::Fail
    };
    add_check(
        checks,
        "config-integrity",
        format!("{} config sha256", node.name),
        status,
        format!("expected {expected}, actual {actual}, bytes {bytes}"),
    );
}
