use eframe::egui;

use crate::{
    app::{
        text::{short_path, truncate_middle},
        widgets::fact,
        NeoNexusApp,
    },
    private_network::{CommitteeRoster, LaunchPackValidationStatus},
    roles::PrivateNetworkPlan,
};

pub(in crate::app::views::roles::private_network) fn render_plan_status(
    app: &NeoNexusApp,
    ui: &mut egui::Ui,
    plan: &PrivateNetworkPlan,
    materialized_count: usize,
    signer_handoff: &anyhow::Result<Option<CommitteeRoster>>,
) {
    let committee_key_count = app
        .private_network_committee_keys
        .split(|character: char| character == ',' || character == ';' || character.is_whitespace())
        .filter(|value| !value.trim().is_empty())
        .count();
    let signer_reference_count = app
        .private_network_signer_refs
        .lines()
        .filter(|value| !value.trim().is_empty())
        .count();

    fact(ui, "Template", plan.template.description());
    fact(ui, "Consensus", &plan.consensus_count().to_string());
    fact(
        ui,
        "Materialized",
        &format!("{materialized_count}/{}", plan.nodes.len()),
    );
    fact(
        ui,
        "Committee keys",
        &format!("{committee_key_count}/{} minimum", plan.consensus_count()),
    );
    render_signer_status(ui, plan, signer_handoff, signer_reference_count);
    render_last_export_status(app, ui);
}

fn render_signer_status(
    ui: &mut egui::Ui,
    plan: &PrivateNetworkPlan,
    signer_handoff: &anyhow::Result<Option<CommitteeRoster>>,
    signer_reference_count: usize,
) {
    match signer_handoff {
        Ok(Some(roster)) => {
            let summary = roster.handoff_summary(plan.consensus_count());
            fact(ui, "Signer handoff", &summary.operator_summary());
            if summary.sidecar_command_count > 0 {
                fact(
                    ui,
                    "Sidecar argv",
                    &format!(
                        "{}/{} no-shell plans",
                        summary.sidecar_command_plan_count, summary.sidecar_command_count
                    ),
                );
            }
        }
        Ok(None) => fact(
            ui,
            "Signer handoff",
            &format!("{signer_reference_count} refs; committee keys pending"),
        ),
        Err(error) => fact(ui, "Signer issue", &truncate_middle(&error.to_string(), 58)),
    }
}

fn render_last_export_status(app: &NeoNexusApp, ui: &mut egui::Ui) {
    if let Some(validation) = &app.private_network_last_validation {
        let status = if validation.failed_count == 0 {
            if validation.warning_count == 0 {
                "ready"
            } else {
                "warnings"
            }
        } else {
            "blocked"
        };
        fact(
            ui,
            "Last export",
            &format!(
                "{status}: {} passed, {} warnings, {} failed",
                validation.passed_count, validation.warning_count, validation.failed_count
            ),
        );
        fact(ui, "Manifest", &short_path(&validation.manifest_path, 58));
        if let Some(issue) = validation
            .checks
            .iter()
            .find(|check| check.status == LaunchPackValidationStatus::Fail)
            .or_else(|| {
                validation
                    .checks
                    .iter()
                    .find(|check| check.status == LaunchPackValidationStatus::Warn)
            })
        {
            fact(
                ui,
                "First issue",
                &truncate_middle(
                    &format!("{} {}: {}", issue.category, issue.label, issue.message),
                    58,
                ),
            );
        }
    } else {
        fact(ui, "Last export", "not validated");
    }
}
