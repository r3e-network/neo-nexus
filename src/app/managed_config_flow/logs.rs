use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn exit_notice_with_log_diagnosis(
        &self,
        node: &NodeConfig,
        exit: &ProcessExit,
    ) -> String {
        let base = exit_notice(&node.name, exit);
        let Ok(snapshot) = LogReader::snapshot(self.node_log_path(node), LOG_MAX_BYTES) else {
            return base;
        };
        let diagnosis = LogReader::diagnose(&snapshot);
        if matches!(
            diagnosis.status,
            LogDiagnosisStatus::Critical | LogDiagnosisStatus::Warning
        ) {
            format!("{base}; log diagnosis: {}", diagnosis.summary)
        } else {
            base
        }
    }

    pub(in crate::app) fn clear_selected_log(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.session.notice = Some("Select a node before clearing its log".to_string());
            return;
        };

        let path = self.node_log_path(&node);
        match LogReader::clear(&path) {
            Ok(()) => {
                self.log_page = 0;
                self.session.notice = Some(format!("Log cleared: {}", short_path(&path, 48)));
                self.record_node_event(
                    &node,
                    EventKind::LogCleared,
                    EventSeverity::Warning,
                    format!("Log cleared at {}", path.display()),
                );
            }
            Err(error) => self.session.notice = Some(error.to_string()),
        }
    }
}
