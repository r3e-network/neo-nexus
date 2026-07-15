use eframe::egui;

use super::super::super::super::{
    theme::{self, muted_text},
    widgets::labeled_text,
    NeoNexusApp,
};

mod plan;
mod signers;

pub(super) use self::{plan::render_plan_actions, signers::render_signer_inputs};
