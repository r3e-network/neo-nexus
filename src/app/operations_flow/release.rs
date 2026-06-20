use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn package_native_release(&mut self) {
        match ReleasePackager::package_current_executable(self.release_package_dir()) {
            Ok(package) => {
                let message = format!(
                    "Release packaged: {}, {} archive, {}",
                    package.package_id,
                    format_bytes(package.archive_bytes),
                    short_path(&package.archive_path, 48)
                );
                self.record_event(
                    None,
                    None,
                    EventKind::ReleasePackaged,
                    EventSeverity::Info,
                    format!(
                        "Release package {} written to {}; checksum {}",
                        package.package_id,
                        package.archive_path.display(),
                        package.checksum_path.display()
                    ),
                );
                self.last_release_verification = None;
                self.last_release_package = Some(package);
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    pub(in crate::app) fn verify_native_release_package(&mut self) {
        let input = self.last_release_package.as_ref().map_or_else(
            || self.release_package_dir(),
            |package| package.manifest_path.clone(),
        );
        match ReleasePackageVerifier::verify(&input) {
            Ok(verification) => {
                let message = format!(
                    "Release verified: {}, {} archive, {}",
                    verification.package_id,
                    format_bytes(verification.archive_bytes),
                    short_path(&verification.manifest_path, 48)
                );
                self.record_event(
                    None,
                    None,
                    EventKind::ReleasePackageVerified,
                    EventSeverity::Info,
                    format!(
                        "Release package {} verified from {}; archive SHA-256 {}",
                        verification.package_id,
                        input.display(),
                        verification.archive_sha256
                    ),
                );
                self.last_release_verification = Some(verification);
                self.notice = Some(message);
            }
            Err(error) => {
                let message = format!("Release verification failed: {error}");
                self.record_event(
                    None,
                    None,
                    EventKind::ReleasePackageVerified,
                    EventSeverity::Critical,
                    message.clone(),
                );
                self.notice = Some(message);
            }
        }
    }
}
