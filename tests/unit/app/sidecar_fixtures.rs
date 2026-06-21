use std::{
    io::{Read, Write},
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    thread,
};

pub(super) fn write_app_launch_pack_sidecar_manifest(root: &Path) -> anyhow::Result<PathBuf> {
    write_app_launch_pack_sidecar_manifest_with_args(
        root,
        &["-c", "echo app-sidecar-ready; sleep 5"],
    )
}

fn write_app_launch_pack_sidecar_manifest_with_args(
    root: &Path,
    arguments: &[&str],
) -> anyhow::Result<PathBuf> {
    write_app_launch_pack_sidecar_manifest_with_endpoint_and_args(
        root,
        "http://127.0.0.1:9021",
        arguments,
    )
}

pub(super) fn write_app_launch_pack_sidecar_manifest_with_endpoint_and_args(
    root: &Path,
    endpoint: &str,
    arguments: &[&str],
) -> anyhow::Result<PathBuf> {
    let binary = write_app_bundled_sidecar_launcher(root)?;
    write_app_launch_pack_sidecar_manifest_with_endpoint_binary_and_args(
        root, endpoint, &binary, arguments,
    )
}

fn write_app_bundled_sidecar_launcher(root: &Path) -> anyhow::Result<String> {
    let relative = PathBuf::from("signer-bin").join("app-sidecar.sh");
    let script_path = root.join(&relative);
    if let Some(parent) = script_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&script_path, "#!/bin/sh\nexec /bin/sh \"$@\"\n")?;
    let mut permissions = std::fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&script_path, permissions)?;
    Ok(relative.to_string_lossy().to_string())
}

pub(super) fn write_app_launch_pack_sidecar_manifest_with_endpoint_binary_and_args(
    root: &Path,
    endpoint: &str,
    binary: &str,
    arguments: &[&str],
) -> anyhow::Result<PathBuf> {
    let public_key = format!("02{}", "a".repeat(64));
    let manifest_path = root.join("manifest.json");
    std::fs::create_dir_all(root)?;
    std::fs::write(
        &manifest_path,
        serde_json::json!({
            "schema_version": 10,
            "generated_at_unix": 1_800_000_000_u64,
            "template": "Single validator",
            "runtime": "neo-rs",
            "network": "private",
            "network_magic": 1_230_301_u32,
            "validators_count": 1,
            "seed_nodes": ["127.0.0.1:30333"],
            "committee": {
                "signer_count": 1,
                "wallet_reference_count": 1,
                "endpoint_reference_count": 1,
                "sidecar_command_count": 1,
                "public_keys": [public_key],
                "secret_material_policy": "references-only-no-private-keys-or-passwords",
                "preflight_policy": "check-native-wallet-paths-http-endpoints-and-sidecar-commands",
                "signers": [{
                    "label": "committee-signer-1",
                    "public_key": format!("02{}", "a".repeat(64)),
                    "wallet_path": "wallets/validator-1.wallet.json",
                    "signer_endpoint": endpoint,
                    "signer_command_template": "/bin/sh -c 'echo {label}; sleep 5'",
                    "signer_command": format!("/bin/sh -c 'echo committee-signer-1; sleep 5 --listen {endpoint}'"),
                    "signer_command_plan": {
                        "execution_policy": "argv-no-shell",
                        "binary": binary,
                        "arguments": arguments
                    }
                }]
            },
            "secret_provisioning": {
                "schema_version": 1,
                "policy": "operator-provided-wallets-no-secret-material-in-launch-pack",
                "wallet_provisioning_file": "wallet-provisioning.json",
                "wallet_instructions_file": "wallets/README.md",
                "recommended_wallet_root": "wallets",
                "required_wallet_count": 1,
                "wallet_reference_count": 1,
                "missing_wallet_reference_count": 0,
                "generated_secret_count": 0
            },
            "scripts": {
                "runbook": "RUNBOOK.md",
                "preflight_unix": "preflight-unix.sh",
                "preflight_windows": "preflight-windows.ps1",
                "health_unix": "health-unix.sh",
                "health_windows": "health-windows.ps1",
                "start_unix": "start-unix.sh",
                "stop_unix": "stop-unix.sh",
                "start_windows": "start-windows.ps1",
                "stop_windows": "stop-windows.ps1"
            },
            "artifacts": [],
            "nodes": []
        })
        .to_string(),
    )?;
    Ok(manifest_path)
}

pub(super) fn spawn_one_shot_http_server() -> anyhow::Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    thread::spawn(move || {
        if let Ok((mut stream, _peer)) = listener.accept() {
            let mut buffer = [0_u8; 1024];
            let _ = stream.read(&mut buffer);
            let _ = stream.write_all(
                b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
        }
    });
    Ok(format!("http://{address}/health"))
}
