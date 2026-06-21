use crate::app::domain::{RuntimePackageManager, RuntimePlatform};

use super::{bytes_to_mib_ceil, default_runtime_node_type, RuntimePackageDraft};

impl Default for RuntimePackageDraft {
    fn default() -> Self {
        let platform = RuntimePlatform::current();
        Self {
            id: "neo-rs-local".to_string(),
            label: "neo-rs local build".to_string(),
            node_type: default_runtime_node_type(),
            version: "latest".to_string(),
            os: platform.os,
            arch: platform.arch,
            source_path: String::new(),
            executable_name: "neo-node".to_string(),
            expected_sha256: String::new(),
            signature_path: String::new(),
            ed25519_public_key: String::new(),
            download_url: String::new(),
            download_file_name: String::new(),
            download_max_mib: bytes_to_mib_ceil(RuntimePackageManager::DEFAULT_DOWNLOAD_MAX_BYTES),
        }
    }
}
