//! Session chrome and navigation state: theme, density, active view, inspector,
//! operator notices/toasts, and Nodes workspace tab.

use super::toasts::ToastStack;
use crate::app::{
    theme::{Theme, UiDensity},
    view::View,
    views::{NetworkHubSection, NodeWorkspaceTab},
};

#[derive(Debug)]
pub(in crate::app) struct SessionUi {
    pub(in crate::app) theme: Theme,
    pub(in crate::app) density: UiDensity,
    pub(in crate::app) inspector_visible: bool,
    pub(in crate::app) selected_view: View,
    pub(in crate::app) persisted_view: View,
    pub(in crate::app) node_workspace_tab: NodeWorkspaceTab,
    pub(in crate::app) persisted_node_workspace_tab: NodeWorkspaceTab,
    pub(in crate::app) network_hub_section: NetworkHubSection,
    pub(in crate::app) notice: Option<String>,
    pub(in crate::app) toasts: ToastStack,
}

impl SessionUi {
    pub(in crate::app) fn new(
        theme: Theme,
        density: UiDensity,
        inspector_visible: bool,
        view: View,
        node_workspace_tab: NodeWorkspaceTab,
        notice: Option<String>,
    ) -> Self {
        Self {
            theme,
            density,
            inspector_visible,
            selected_view: view,
            persisted_view: view,
            node_workspace_tab,
            persisted_node_workspace_tab: node_workspace_tab,
            network_hub_section: NetworkHubSection::default(),
            notice,
            toasts: ToastStack::default(),
        }
    }
}
