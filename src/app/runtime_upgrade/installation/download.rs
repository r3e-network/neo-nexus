use crate::app::{
    domain::{format_bytes, EventKind, EventSeverity, RuntimePackageManager},
    short_path, NeoNexusApp,
};

impl NeoNexusApp {
    pub(in crate::app) fn download_runtime_package(&mut self) {
        let request = match self.runtime_package_draft.to_download_request() {
            Ok(request) => request,
            Err(error) => {
                self.notice = Some(error.to_string());
                return;
            }
        };

        match RuntimePackageManager::download_https(&request, self.runtime_download_dir()) {
            Ok(download) => {
                self.runtime_package_draft.source_path = download.path.display().to_string();
                let message = format!(
                    "Runtime downloaded: {} ({})",
                    short_path(&download.path, 54),
                    format_bytes(download.bytes)
                );
                self.record_event_notice(
                    EventKind::RuntimeDownloaded,
                    EventSeverity::Info,
                    message,
                );
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}
