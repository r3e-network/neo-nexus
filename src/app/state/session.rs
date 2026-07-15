//! Session chrome and navigation state: theme, active view, inspector, and
//! operator notices/toasts. Separated from fleet and domain services so the
//! shell can reason about UI session without touching node inventory.

use super::toasts::ToastStack;
use crate::app::{
    theme::Theme,
    view::View,
    views::{NetworkHubSection, NodeWorkspaceTab},
};

#[derive(Debug)]
pub(in crate::app) struct SessionUi {
    pub(in crate::app) theme: Theme,
    pub(in crate::app) inspector_visible: bool,
    pub(in crate::app) selected_view: View,
    pub(in crate::app) persisted_view: View,
    pub(in crate::app) node_workspace_tab: NodeWorkspaceTab,
    pub(in crate::app) network_hub_section: NetworkHubSection,
    pub(in crate::app) notice: Option<String>,
    pub(in crate::app) toasts: ToastStack,
}

impl SessionUi {
    pub(in crate::app) fn new(
        theme: Theme,
        inspector_visible: bool,
        view: View,
        notice: Option<String>,
    ) -> Self {
        Self {
            theme,
            inspector_visible,
            selected_view: view,
            persisted_view: view,
            node_workspace_tab: NodeWorkspaceTab::default(),
            network_hub_section: NetworkHubSection::default(),
            notice,
            toasts: ToastStack::default(),
        }
    }
}
