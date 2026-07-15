//! Network primary surface: Federation, private-network roles, and wallets
//! share one top-level destination with an in-page hub control.

use eframe::egui;

use crate::app::{view::View, widgets::page_chrome, NeoNexusApp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::app) enum NetworkHubSection {
    #[default]
    Federation,
    Roles,
    Wallets,
}

impl NetworkHubSection {
    pub(in crate::app) const ALL: [Self; 3] = [Self::Federation, Self::Roles, Self::Wallets];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Federation => "Federation",
            Self::Roles => "Private Net",
            Self::Wallets => "Wallets",
        }
    }

    pub(in crate::app) fn from_view(view: View) -> Option<Self> {
        match view {
            View::Federation => Some(Self::Federation),
            View::Roles => Some(Self::Roles),
            View::Wallets => Some(Self::Wallets),
            _ => None,
        }
    }
}

impl NeoNexusApp {
    /// Render the Network primary: hub tabs then the selected surface.
    pub(super) fn render_network_hub(&mut self, ui: &mut egui::Ui) {
        if let Some(section) = NetworkHubSection::from_view(self.session.selected_view) {
            self.session.network_hub_section = section;
        }

        let mut index = self.session.network_hub_section as usize;
        let labels = NetworkHubSection::ALL.map(NetworkHubSection::label);
        if page_chrome(ui, None, Some((&labels, &mut index))) {
            self.session.network_hub_section = NetworkHubSection::ALL[index];
            self.session.selected_view = match self.session.network_hub_section {
                NetworkHubSection::Federation => View::Federation,
                NetworkHubSection::Roles => View::Roles,
                NetworkHubSection::Wallets => View::Wallets,
            };
        }

        match self.session.network_hub_section {
            NetworkHubSection::Federation => self.render_federation(ui),
            NetworkHubSection::Roles => self.render_roles(ui),
            NetworkHubSection::Wallets => self.render_wallets(ui),
        }
    }
}
