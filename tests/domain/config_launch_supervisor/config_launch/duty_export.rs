//! Every duty the support matrix offers must produce a config that can be
//! written.
//!
//! The generator switches sections on from the duty and the wallet; the
//! validator decides whether the result may be written. When the two disagree
//! the export fails and **no file is written at all** — so a duty that looks
//! available in the workbench is unusable, and the failure only shows up when an
//! operator tries to apply it.
//!
//! Two such disagreements shipped: a neo-rs node on Consensus duty wrote
//! `consensus.enabled = true` against a validator that expected `false`, and
//! every neo-cli duty with an unlockable wallet wrote `UnlockWallet.IsActive =
//! true` against a validator that hard-coded `false`. Both were reachable from
//! the GUI. This walks the whole matrix so the next one cannot hide.

use crate::*;

fn node_of(repo: &Repository, node_type: NodeType) -> NodeConfig {
    let id = create_node(repo, &node_type.to_string(), node_type);
    repo.list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == id)
        .unwrap()
}

/// The support matrix is a promise. Anything it marks supported has to survive
/// generation *and* validation, for every client, with and without a wallet
/// whose password is available.
#[test]
fn every_supported_duty_exports_for_every_client() {
    let repo = create_repo();
    let temp = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();

    for node_type in NodeType::ALL {
        let node = node_of(&repo, node_type);
        for role in NodeRole::ALL {
            if !role_availability(node_type, role).is_supported() {
                continue;
            }
            for (label, context) in [
                ("no wallet", GenerationContext::for_role(role)),
                (
                    "wallet assigned, locked",
                    GenerationContext::for_role(role)
                        .with_wallet(ServiceWallet::at("/wallets/node.json")),
                ),
                (
                    "wallet unlocked",
                    GenerationContext::for_role(role).with_wallet(
                        ServiceWallet::at("/wallets/node.json").unlocked_with("hunter2"),
                    ),
                ),
            ] {
                let path = temp
                    .path()
                    .join(format!("{node_type}-{role:?}-{}.cfg", label.len()));
                if let Err(error) = ConfigExporter::write_node_config_to_path_with_context(
                    &path,
                    &node,
                    &[],
                    None,
                    &context,
                ) {
                    failures.push(format!("{node_type} + {role} ({label}): {error}"));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "duties the support matrix offers cannot be exported, so applying them \
         writes no config at all:\n  {}",
        failures.join("\n  "),
    );
}

/// A duty a client cannot perform must be refused by the matrix rather than by
/// a validation error four steps later.
#[test]
fn unsupported_duties_are_refused_by_the_matrix_not_by_validation() {
    for node_type in NodeType::ALL {
        for role in NodeRole::ALL {
            let availability = role_availability(node_type, role);
            if availability.is_supported() {
                continue;
            }
            assert!(
                availability
                    .reason()
                    .is_some_and(|reason| !reason.is_empty()),
                "{node_type} refuses {role} without saying why",
            );
        }
    }
}

/// The validator has to agree with the generator about the same duty, which is
/// only possible if it is given the same context. Passing no context must keep
/// describing a plain relaying node.
#[test]
fn a_duty_free_context_still_validates_as_a_relaying_node() {
    let repo = create_repo();
    for node_type in NodeType::ALL {
        let node = node_of(&repo, node_type);
        let rendered = ConfigGenerator::render_for_node(&node, &[]).unwrap();
        let report = ConfigValidator::validate_rendered(&node, &rendered);
        assert!(
            report.is_success(),
            "{node_type} without a duty: {}",
            report.operator_summary()
        );
    }
}
