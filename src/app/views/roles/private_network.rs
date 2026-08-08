mod actions;
mod grid;
mod section;
mod status;

use eframe::egui;

use crate::app::domain::{
    CommitteeRoster, NodeConfig, NodeType, PrivateNetworkPlan, PrivateNetworkPlanner,
    PrivateNetworkTemplate,
};

use super::super::super::{theme::muted_text, NeoNexusApp};

pub(in crate::app) use section::PrivateNetworkSection;

impl NeoNexusApp {
    /// One stage of the private-network workflow, chosen by the caller's tabs.
    ///
    /// All seven blocks used to render together, which put ~414pt of them below
    /// a panel that does not scroll — including every button that writes the
    /// network out. See [`PrivateNetworkSection`].
    pub(super) fn render_private_network_plan(&mut self, ui: &mut egui::Ui) {
        let plan = PrivateNetworkPlanner::plan(
            self.private_network_template,
            self.private_network_runtime,
        );
        let source = self.private_network_template_source_node().cloned();
        let conflicts = plan.conflicts_with(&self.fleet.nodes);
        let materialized_count = self.private_network_materialized_count(&plan);
        let launch_pack_ready = self.private_network_launch_pack_ready(&plan, materialized_count);
        let signer_handoff = CommitteeRoster::from_public_keys_and_references(
            &self.private_network_committee_keys,
            &self.private_network_signer_refs,
        );

        match self.sections.private_network {
            PrivateNetworkSection::Plan => {
                self.render_private_network_plan_controls(ui);
                status::render_plan_status(ui, &plan);
                ui.separator();
                grid::render_plan_grid(ui, &plan);
            }
            PrivateNetworkSection::Signers => {
                actions::render_signer_inputs(self, ui);
            }
            PrivateNetworkSection::Deploy => {
                let can_create_nodes = source.is_some() && conflicts.is_empty();
                status::render_deploy_status(self, ui, &plan, materialized_count, &signer_handoff);
                status::render_source_status(self, ui, source.as_ref(), conflicts.first());
                status::render_sidecar_status(self, ui);
                actions::render_plan_actions(
                    self,
                    ui,
                    can_create_nodes,
                    launch_pack_ready,
                    signer_handoff.is_ok(),
                );
            }
        }
    }

    fn render_private_network_plan_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Template").color(muted_text()));
            egui::ComboBox::from_id_salt("private_network_template")
                .selected_text(self.private_network_template.label())
                .width(180.0)
                .show_ui(ui, |ui| {
                    for template in PrivateNetworkTemplate::ALL {
                        ui.selectable_value(
                            &mut self.private_network_template,
                            template,
                            template.label(),
                        );
                    }
                });

            ui.separator();
            ui.label(egui::RichText::new("Runtime").color(muted_text()));
            egui::ComboBox::from_id_salt("private_network_runtime")
                .selected_text(self.private_network_runtime.to_string())
                .width(120.0)
                .show_ui(ui, |ui| {
                    // Neo X is absent by design: its validator set comes from
                    // a genesis allocation, not from the committee roster these
                    // templates build. See `ChainFamily::has_committee_templates`.
                    for node_type in NodeType::ALL
                        .into_iter()
                        .filter(|node_type| node_type.family().has_committee_templates())
                    {
                        ui.selectable_value(
                            &mut self.private_network_runtime,
                            node_type,
                            node_type.to_string(),
                        );
                    }
                });
        });
    }

    fn private_network_materialized_count(&self, plan: &PrivateNetworkPlan) -> usize {
        plan.nodes
            .iter()
            .filter(|planned| {
                self.fleet.nodes.iter().any(|node| {
                    node.name == planned.name
                        && node.node_type == planned.node_type
                        && node.network == planned.network
                })
            })
            .count()
    }

    fn private_network_launch_pack_ready(
        &self,
        plan: &PrivateNetworkPlan,
        materialized_count: usize,
    ) -> bool {
        materialized_count == plan.nodes.len()
            && plan.nodes.iter().all(|planned| {
                self.fleet
                    .nodes
                    .iter()
                    .any(|node| node.name == planned.name && !node.status.is_active())
            })
    }
}

pub(super) type SourceNode<'a> = Option<&'a NodeConfig>;
