use crate::{
    app::{short_path, NeoNexusApp},
    events::{EventKind, EventSeverity},
    runtime::RuntimePackageManager,
};

impl NeoNexusApp {
    pub(in crate::app) fn install_runtime_package(&mut self) {
        let manifest = match self.runtime_package_draft.to_manifest() {
            Ok(manifest) => manifest,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };

        match RuntimePackageManager::install(&manifest, self.runtime_install_root()) {
            Ok(installation) => {
                if let Err(error) = self.repository.upsert_runtime_installation(&installation) {
                    self.notice = Some(error.to_string());
                    return;
                }
                self.selected_runtime_installation = Some(installation.package_id.clone());
                let trusted_signer =
                    self.mark_runtime_signer_used_by_key(installation.signer_public_key.as_deref());
                let signer_suffix = trusted_signer
                    .as_ref()
                    .map_or(String::new(), |label| format!("; signer {label}"));
                let message = format!(
                    "Runtime installed: {} {} at {}{}",
                    installation.node_type,
                    installation.version,
                    short_path(&installation.binary_path, 54),
                    signer_suffix
                );
                self.record_event(
                    None,
                    None,
                    EventKind::RuntimeInstalled,
                    EventSeverity::Info,
                    message.clone(),
                );
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
