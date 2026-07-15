use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn revalidate_private_network_launch_pack(&mut self) {
        let Some(root_path) = self.private_network_last_export_root.clone() else {
            self.session.notice = Some("Export a private launch pack before revalidating".to_string());
            return;
        };
        let validation_message = self.validate_private_network_launch_pack(&root_path);
        self.session.notice = Some(format!(
            "Private launch pack revalidated: {}; {}",
            short_path(&root_path, 54),
            validation_message
        ));
    }
}
