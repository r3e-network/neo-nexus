use crate::app::domain::RuntimeRelease;

use super::{bytes_to_mib_ceil, RuntimePackageDraft};

impl RuntimePackageDraft {
    pub(in crate::app) fn load_release(&mut self, release: &RuntimeRelease) {
        self.id = release.id.clone();
        self.label = release.label.clone();
        self.node_type = release.node_type;
        self.version = release.version.clone();
        self.os = release.platform.os.clone();
        self.arch = release.platform.arch.clone();
        self.source_path.clear();
        self.executable_name = release.executable_name.clone();
        self.expected_sha256 = release.expected_sha256.clone();
        self.signature_path.clear();
        self.ed25519_public_key.clear();
        self.download_url = release.url.clone();
        self.download_file_name = release.file_name.clone();
        self.download_max_mib = bytes_to_mib_ceil(release.max_bytes);
    }
}
