use neo_nexus::private_network::PrivateNetworkDeploymentExport;

pub(super) fn assert_unix_scripts(
    export: &PrivateNetworkDeploymentExport,
    first_config_sha256: &str,
) {
    let preflight_unix = std::fs::read_to_string(&export.preflight_unix_path).unwrap();
    assert!(preflight_unix.contains("check_file 'neo-rs-validator-1 binary'"));
    assert!(preflight_unix.contains("check_file 'neo-rs-validator-1 config'"));
    assert!(preflight_unix.contains("check_sha256 'neo-rs-validator-1 config'"));
    assert!(preflight_unix.contains(first_config_sha256));
    assert!(preflight_unix.contains("sha256sum"));
    assert!(preflight_unix.contains("shasum -a 256"));
    assert!(preflight_unix.contains("openssl dgst -sha256"));
    assert!(preflight_unix.contains("check_dir 'neo-rs-validator-1 workdir'"));
    assert!(preflight_unix.contains("check_pid 'neo-rs-validator-1'"));
    assert!(preflight_unix.contains("check_port 'neo-rs-validator-1 RPC' 30332"));
    assert!(preflight_unix.contains("check_port 'neo-rs-validator-1 P2P' 30333"));
    assert!(preflight_unix.contains("check_port 'neo-rs-validator-1 WS' 30334"));
    assert!(preflight_unix.contains("nc -z 127.0.0.1"));
    assert!(preflight_unix.contains("NEONEXUS_SIGNER_TIMEOUT_SECONDS"));
    assert!(preflight_unix.contains("# Committee signer references"));
    assert!(preflight_unix.contains("check_signer_command 'committee-signer-1' 'neo-signer'"));
    assert!(preflight_unix.contains(
        "check_file 'committee-signer-1 wallet' '/secure/neonexus/validator-1.wallet.json'"
    ));
    assert!(preflight_unix.contains(
        "committee-signer-1 endpoint will be checked after sidecar start: http://127.0.0.1:9021"
    ));
    assert!(
        preflight_unix.contains(
            "committee-signer-2 wallet path is Windows-style; skipping on Unix: C:\\neo\\validator-2.wallet.json"
        )
    );
    assert!(preflight_unix.contains(
        "check_signer_endpoint 'committee-signer-2' 'https://signer.example.test/validator-2'"
    ));
    assert!(preflight_unix.contains("Preflight failed"));
    let health_unix = std::fs::read_to_string(&export.health_unix_path).unwrap();
    assert!(health_unix.contains("NEONEXUS_HEALTH_TIMEOUT_SECONDS"));
    assert!(health_unix.contains("wait_signer 'committee-signer-1' 'http://127.0.0.1:9021'"));
    assert!(health_unix.contains("signer_pid_alive()"));
    assert!(health_unix.contains("rpc_ready()"));
    assert!(health_unix.contains("set +e\n  rpc_ready"));
    assert!(health_unix.contains("\"method\":\"getblockcount\""));
    assert!(health_unix.contains("wait_node 'neo-rs-validator-1'"));
    assert!(health_unix.contains("30332 30333 30334"));
    assert!(health_unix.contains("Health check failed"));
    let start_unix = std::fs::read_to_string(&export.start_unix_path).unwrap();
    assert!(start_unix.contains("#!/usr/bin/env sh"));
    assert!(start_unix.contains("./preflight-unix.sh"));
    assert!(start_unix.contains("./health-unix.sh"));
    assert!(start_unix.contains("NEONEXUS_SKIP_PREFLIGHT"));
    assert!(start_unix.contains("NEONEXUS_SKIP_HEALTH"));
    assert!(start_unix.contains("start_signer 'committee-signer-1' 'neo-signer' '--wallet'"));
    assert!(!start_unix.contains("nohup sh -c"));
    assert!(start_unix.contains("nohup \"$binary\" \"$@\""));
    assert!(start_unix.contains("nohup \"$@\""));
    assert!(start_unix.contains("start_node 'neo-rs-validator-1'"));
    assert!(start_unix.contains("'/opt/neo-rs/neo-node' '--config'"));
    assert!(start_unix.contains(".out.log"));
    assert!(start_unix.contains(".pid"));
    let stop_unix = std::fs::read_to_string(&export.stop_unix_path).unwrap();
    assert!(stop_unix.contains("kill \"$pid\""));
    assert!(stop_unix.contains("stop_signer 'committee-signer-1'"));
    assert!(
        stop_unix.find("stop_node 'neo-rs-observer-7'")
            < stop_unix.find("stop_node 'neo-rs-validator-1'")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = std::fs::metadata(&export.start_unix_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o755);
        let preflight_mode = std::fs::metadata(&export.preflight_unix_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(preflight_mode, 0o755);
        let health_mode = std::fs::metadata(&export.health_unix_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(health_mode, 0o755);
        for script_path in [
            &export.preflight_unix_path,
            &export.health_unix_path,
            &export.start_unix_path,
            &export.stop_unix_path,
        ] {
            let status = std::process::Command::new("sh")
                .arg("-n")
                .arg(script_path)
                .status()
                .unwrap();
            assert!(
                status.success(),
                "{} is not valid sh",
                script_path.display()
            );
        }
    }
}

pub(super) fn assert_windows_scripts(
    export: &PrivateNetworkDeploymentExport,
    first_config_sha256: &str,
) {
    let preflight_windows = std::fs::read_to_string(&export.preflight_windows_path).unwrap();
    assert!(preflight_windows.contains("Test-NeoNexusFile -Label 'neo-rs-validator-1 binary'"));
    assert!(preflight_windows.contains("Test-NeoNexusSha256 -Label 'neo-rs-validator-1 config'"));
    assert!(preflight_windows.contains(first_config_sha256));
    assert!(preflight_windows.contains("Get-FileHash -LiteralPath $Path -Algorithm SHA256"));
    assert!(
        preflight_windows.contains("Test-NeoNexusDirectory -Label 'neo-rs-validator-1 workdir'")
    );
    assert!(preflight_windows.contains("Test-NeoNexusPid -Name 'neo-rs-validator-1'"));
    assert!(
        preflight_windows.contains("Test-NeoNexusPort -Label 'neo-rs-validator-1 RPC' -Port 30332")
    );
    assert!(
        preflight_windows.contains("Test-NeoNexusPort -Label 'neo-rs-validator-1 WS' -Port 30334")
    );
    assert!(preflight_windows.contains("[System.Net.Sockets.TcpListener]"));
    assert!(preflight_windows.contains("[System.Net.Http.HttpClient]"));
    assert!(preflight_windows
        .contains("Test-NeoNexusSignerCommand -Label 'committee-signer-1' -Binary 'neo-signer'"));
    assert!(
        preflight_windows.contains(
            "Warn-NeoNexusCheck 'committee-signer-1 wallet path is POSIX-style; skipping on Windows: /secure/neonexus/validator-1.wallet.json'"
        )
    );
    assert!(
        preflight_windows.contains(
            "Test-NeoNexusFile -Label 'committee-signer-2 wallet' -Path 'C:\\neo\\validator-2.wallet.json'"
        )
    );
    assert!(preflight_windows.contains(
        "committee-signer-1 endpoint will be checked after sidecar start: http://127.0.0.1:9021"
    ));
    assert!(
        preflight_windows.contains(
            "Test-NeoNexusSignerEndpoint -Label 'committee-signer-2' -Endpoint 'https://signer.example.test/validator-2'"
        )
    );
    assert!(preflight_windows.contains("Preflight failed"));
    let health_windows = std::fs::read_to_string(&export.health_windows_path).unwrap();
    assert!(health_windows.contains("NEONEXUS_HEALTH_TIMEOUT_SECONDS"));
    assert!(health_windows.contains(
        "Wait-NeoNexusSigner -Label 'committee-signer-1' -Endpoint 'http://127.0.0.1:9021'"
    ));
    assert!(health_windows.contains("Test-NeoNexusSignerPidAlive"));
    assert!(health_windows.contains("Test-NeoNexusRpcReady"));
    assert!(health_windows.contains("getblockcount"));
    assert!(health_windows.contains("Wait-NeoNexusNode -Name 'neo-rs-validator-1'"));
    assert!(health_windows.contains("-RpcPort 30332 -P2pPort 30333 -WsPort 30334"));
    assert!(health_windows.contains("Health check failed"));
    let start_windows = std::fs::read_to_string(&export.start_windows_path).unwrap();
    assert!(start_windows.contains("Start-Process"));
    assert!(start_windows.contains("preflight-windows.ps1"));
    assert!(start_windows.contains("health-windows.ps1"));
    assert!(start_windows.contains("NEONEXUS_SKIP_PREFLIGHT"));
    assert!(start_windows.contains("NEONEXUS_SKIP_HEALTH"));
    assert!(start_windows.contains(
        "Start-NeoNexusSigner -Label 'committee-signer-1' -Binary 'neo-signer' -Args @('--wallet'"
    ));
    assert!(!start_windows.contains("-FilePath 'powershell'"));
    assert!(start_windows.contains("Start-Process -FilePath $Binary -ArgumentList $Args"));
    assert!(start_windows.contains("Start-NeoNexusNode -Name 'neo-rs-validator-1'"));
    assert!(start_windows.contains("-Binary '/opt/neo-rs/neo-node'"));
    assert!(start_windows.contains("-Args @('--config'"));
    let stop_windows = std::fs::read_to_string(&export.stop_windows_path).unwrap();
    assert!(stop_windows.contains("Stop-Process"));
    assert!(stop_windows.contains("Stop-NeoNexusSigner -Label 'committee-signer-1'"));
    assert!(
        stop_windows.find("Stop-NeoNexusNode -Name 'neo-rs-observer-7'")
            < stop_windows.find("Stop-NeoNexusNode -Name 'neo-rs-validator-1'")
    );
}
