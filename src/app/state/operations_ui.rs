//! Operations-page list UI: section tab, action queue, readiness checks, port
//! matrix, and event journal filters/paging/selection.

use crate::app::{
    domain::{
        CheckSeverity, DiagnosticCheckKey, DiagnosticResolution, EventSeverity, Network,
        NodeStatus, ReadinessActionKey,
    },
    views::OperationsSection,
};

#[derive(Debug)]
pub(in crate::app) struct OperationsUi {
    pub(in crate::app) section: OperationsSection,
    pub(in crate::app) persisted_section: OperationsSection,
    pub(in crate::app) action_queue_page: usize,
    pub(in crate::app) action_queue_query: String,
    pub(in crate::app) action_queue_severity_filter: Option<CheckSeverity>,
    pub(in crate::app) action_queue_resolution_filter: Option<DiagnosticResolution>,
    pub(in crate::app) selected_readiness_action: Option<ReadinessActionKey>,
    pub(in crate::app) port_matrix_page: usize,
    pub(in crate::app) port_matrix_query: String,
    pub(in crate::app) port_matrix_status_filter: Option<NodeStatus>,
    pub(in crate::app) port_matrix_network_filter: Option<Network>,
    pub(in crate::app) port_matrix_health_filter: Option<CheckSeverity>,
    pub(in crate::app) readiness_check_page: usize,
    pub(in crate::app) readiness_check_query: String,
    pub(in crate::app) readiness_check_severity_filter: Option<CheckSeverity>,
    pub(in crate::app) readiness_check_resolution_filter: Option<DiagnosticResolution>,
    pub(in crate::app) selected_readiness_check: Option<DiagnosticCheckKey>,
    pub(in crate::app) event_page: usize,
    pub(in crate::app) selected_event: Option<i64>,
    pub(in crate::app) event_query: String,
    pub(in crate::app) event_severity_filter: Option<EventSeverity>,
}

impl OperationsUi {
    pub(in crate::app) fn new(section: OperationsSection) -> Self {
        Self {
            section,
            persisted_section: section,
            action_queue_page: 0,
            action_queue_query: String::new(),
            action_queue_severity_filter: None,
            action_queue_resolution_filter: None,
            selected_readiness_action: None,
            port_matrix_page: 0,
            port_matrix_query: String::new(),
            port_matrix_status_filter: None,
            port_matrix_network_filter: None,
            port_matrix_health_filter: None,
            readiness_check_page: 0,
            readiness_check_query: String::new(),
            readiness_check_severity_filter: None,
            readiness_check_resolution_filter: None,
            selected_readiness_check: None,
            event_page: 0,
            selected_event: None,
            event_query: String::new(),
            event_severity_filter: None,
        }
    }
}
