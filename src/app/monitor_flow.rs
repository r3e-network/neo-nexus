use super::*;

impl NeoNexusApp {
    pub(in crate::app) fn monitor_process_filter(&self) -> ProcessFilter {
        ProcessFilter::new(
            self.monitor_process_state_filter,
            self.monitor_process_high_cpu_filter,
            self.monitor_process_high_memory_filter,
            self.monitor_process_query.as_str(),
        )
    }

    pub(in crate::app) fn filtered_monitor_process_rows(&self) -> Vec<ProcessRow> {
        filter_process_rows(
            &self.metrics_snapshot.node_processes,
            &self.metrics_snapshot.missing_processes,
            &self.monitor_process_filter(),
        )
    }

    pub(in crate::app) fn ensure_valid_monitor_process_selection(&mut self) {
        let rows = self.filtered_monitor_process_rows();
        let selected_exists = self
            .selected_monitor_process
            .as_ref()
            .is_some_and(|id| rows.iter().any(|row| row.node_id() == id));
        if !selected_exists {
            self.selected_monitor_process = rows.first().map(|row| row.node_id().to_string());
            self.monitor_process_page = 0;
        }
        self.monitor_process_page = clamp_page(
            self.monitor_process_page,
            rows.len(),
            MONITOR_PROCESS_PAGE_SIZE,
        );
    }

    pub(in crate::app) fn has_active_monitor_process_filter(&self) -> bool {
        self.monitor_process_state_filter.is_some()
            || self.monitor_process_high_cpu_filter
            || self.monitor_process_high_memory_filter
            || !self.monitor_process_query.trim().is_empty()
    }

    pub(in crate::app) fn clear_monitor_process_filters(&mut self) {
        self.monitor_process_state_filter = None;
        self.monitor_process_high_cpu_filter = false;
        self.monitor_process_high_memory_filter = false;
        self.monitor_process_query.clear();
        self.monitor_process_page = 0;
        self.ensure_valid_monitor_process_selection();
    }

    pub(in crate::app) fn focus_missing_processes(&mut self) {
        self.monitor_process_state_filter = Some(ProcessStateFilter::Missing);
        self.monitor_process_high_cpu_filter = false;
        self.monitor_process_high_memory_filter = false;
        self.monitor_process_query.clear();
        self.monitor_process_page = 0;
        self.selected_monitor_process = self
            .metrics_snapshot
            .missing_processes
            .first()
            .map(|process| process.node_id.clone());
        self.fleet.selected_node = self.selected_monitor_process.clone();
        self.ensure_valid_monitor_process_selection();
    }
}
