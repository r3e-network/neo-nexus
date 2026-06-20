use super::*;

#[path = "runtime_federation/catalog.rs"]
mod catalog;
#[path = "runtime_federation/downloads.rs"]
mod downloads;
#[path = "runtime_federation/package.rs"]
mod package;
#[path = "runtime_federation/profiles.rs"]
mod profiles;
#[path = "runtime_federation/remote.rs"]
mod remote;
#[path = "runtime_federation/repository.rs"]
mod repository;
#[path = "runtime_federation/upgrade_planning.rs"]
mod upgrade_planning;

fn runtime_release_json(
    id: &str,
    node_type: NodeType,
    version: &str,
    platform: &RuntimePlatform,
    hash_digit: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "label": format!("{node_type} {version}"),
        "node_type": node_type.to_string(),
        "version": version,
        "platform": {
            "os": platform.os,
            "arch": platform.arch
        },
        "url": format!("https://downloads.example.com/{id}"),
        "file_name": format!("{id}.bin"),
        "executable_name": "neo-node",
        "expected_sha256": hash_digit.repeat(64)
    })
}

fn runtime_install(
    package_id: &str,
    node_type: NodeType,
    version: &str,
    platform: RuntimePlatform,
    installed_at_unix: u64,
) -> RuntimeInstallation {
    RuntimeInstallation {
        package_id: package_id.to_string(),
        label: format!("{node_type} {version}"),
        node_type,
        version: version.to_string(),
        platform,
        binary_path: PathBuf::from(format!("/opt/neonexus/{package_id}/binary")),
        sha256: "a".repeat(64),
        signature_verified: false,
        signer_public_key: None,
        bytes: 1024,
        installed_at_unix,
    }
}

fn alternate_platform(platform: &RuntimePlatform) -> RuntimePlatform {
    RuntimePlatform {
        os: if platform.os == "linux" {
            "windows".to_string()
        } else {
            "linux".to_string()
        },
        arch: platform.arch.clone(),
    }
}
