//! The one test that walks every node type through the same three gates.
//!
//! "Supported" is a claim per type, and the cheapest way to keep it honest is
//! to make each type clear the identical pipeline here: the workspace can
//! render its config, the render passes the current expectations, and the
//! launch planner produces a runnable command. A new `NodeType` that misses a
//! gate fails here instead of at an operator's launch.

use super::*;

#[test]
fn every_node_type_renders_validates_and_plans() {
    let repo = create_repo();
    for node_type in NodeType::ALL {
        let node_id = create_node(&repo, &format!("matrix-{node_type:?}"), node_type);
        let node = repo
            .list_nodes()
            .unwrap()
            .into_iter()
            .find(|node| node.id == node_id)
            .unwrap();

        // Gate 1 — render: the workspace can produce this node's config.
        let rendered = ConfigGenerator::render_for_node_with_context(
            &node,
            &[],
            None,
            &GenerationContext::default(),
        );
        assert!(
            rendered.is_ok(),
            "{node_type} config generation failed: {:?}",
            rendered.as_ref().unwrap_err()
        );
        let rendered = rendered.unwrap();
        assert!(
            !rendered.text.trim().is_empty(),
            "{node_type} rendered an empty config"
        );

        // Gate 2 — validate: the render satisfies the current expectations,
        // the same check a launch runs before it writes anything.
        let validation = ConfigValidator::validate_rendered(&node, &rendered);
        assert!(
            validation.is_success(),
            "{node_type} render failed validation: {}",
            validation.operator_summary()
        );

        // Gate 3 — plan: a runnable command exists, and the client's config is
        // pinned. neo-cli legitimately plans zero arguments — it reads
        // `config.json` from its working directory — so "args or a pinned
        // config" is the runnable invariant, and the pinned config is the one
        // gate every client shares.
        let plan = LaunchPlanner::plan(
            &node,
            PathBuf::from("/opt/neonexus/nodes/config"),
            PathBuf::from("/opt/neonexus/nodes"),
        );
        assert_eq!(plan.binary_path, node.binary_path);
        assert!(
            plan.managed_config_path.is_some(),
            "{node_type} planned a launch with no pinned managed config"
        );
    }
}

/// The render is stable: re-rendering the same node produces byte-identical
/// text, which is what lets `--config-drift` treat "differs from a fresh
/// render" as a finding rather than generator noise.
#[test]
fn every_node_type_renders_deterministically() {
    let repo = create_repo();
    for node_type in NodeType::ALL {
        let node_id = create_node(&repo, &format!("stable-{node_type:?}"), node_type);
        let node = repo
            .list_nodes()
            .unwrap()
            .into_iter()
            .find(|node| node.id == node_id)
            .unwrap();
        let first = ConfigGenerator::render_for_node_with_context(
            &node,
            &[],
            None,
            &GenerationContext::default(),
        )
        .unwrap();
        let second = ConfigGenerator::render_for_node_with_context(
            &node,
            &[],
            None,
            &GenerationContext::default(),
        )
        .unwrap();
        assert_eq!(
            first.text, second.text,
            "{node_type} render is not deterministic"
        );
        assert!(line_drift(&first.text, &second.text).is_empty());
    }
}
