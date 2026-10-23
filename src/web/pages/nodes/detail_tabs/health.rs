//! The health tab: the four independent facts, and the lifecycle panels.

use crate::{
    agents::HermesAgentAssociation,
    core::{node::NodeConfig, node_health::NodeChainView},
    observe::HealthTransition,
    roles::NodeRole,
    signing::SignerKeyRef,
    web::{chain_state_view as chain_view, html, WebState},
};

/// Tab 2: Status Checks View (AWS EC2 2/2 Checks)
/// The health tab: what the chain says, why, and what to do about it.
///
/// What this replaces fused four unrelated facts into one score — `🟢 2/2
/// System & Instance Checks Passed` — computed from `is_running()` and the
/// last RPC probe, beside a panel captioned `SSM Agent Ready` that measured
/// nothing at all. A node answering RPC promptly at a height that had not moved
/// in an hour read as two passing checks.
///
/// The four facts are kept apart because they are independent and an operator
/// needs each of them: a **process** can be up while the **chain** is stalled,
/// and a stalled node holding a **signer** lease for an **elected** role is the
/// expensive case this workspace exists to catch. One badge cannot say that.
///
/// They travel as a struct rather than six positional arguments, because six
/// `Option`s in a row is a call site nobody can read and two of them are
/// interchangeable by type.
#[derive(Clone, Copy)]
pub struct HealthTabFacts<'a> {
    pub role: Option<NodeRole>,
    pub signer: Option<&'a SignerKeyRef>,
    pub view: Option<&'a NodeChainView>,
    pub timeline: &'a [HealthTransition],
    pub assoc: &'a HermesAgentAssociation,
    pub now: i64,
}

pub fn render_tab_health(
    state: &WebState,
    node: &NodeConfig,
    facts: &HealthTabFacts<'_>,
) -> String {
    let HealthTabFacts {
        role,
        signer,
        view,
        timeline,
        assoc,
        now,
    } = *facts;
    let now_unix = now.max(0) as u64;
    let verdict = view
        .map(|view| chain_view::verdict_panel(view, now_unix))
        .unwrap_or_else(|| {
            html::notice(
                "info",
                "This node has not been judged yet. The observation loop writes a \
                 verdict on its next pass.",
            )
        });

    let requires_signer = role.is_some_and(|role| role.requires_signer());
    let signer_line = match signer {
        Some(key) => format!(
            "leased to {}/{}",
            html::escape(&key.backend_id),
            html::escape(&key.key_id)
        ),
        None if requires_signer => format!(
            "none bound, and {} cannot do its job without one",
            role.map_or("this role", NodeRole::label)
        ),
        None => "none bound; this node signs nothing".to_string(),
    };
    let agent_line = if assoc.is_alive(now) {
        "responding to heartbeats".to_string()
    } else if assoc.enabled {
        "enrolled, no recent heartbeat".to_string()
    } else {
        "not enrolled".to_string()
    };

    let enc_id = html::urlencoding_lite(&node.id);
    format!(
        r#"<div style="margin-top: 12px;">
{verdict}
<div class="grid" style="grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px; margin-top: 16px;">
  <div style="padding: 10px 12px; background: var(--bg-subtle); border-radius: 6px; border: 1px solid var(--line);">
    <div class="muted" style="font-size: 11px; margin-bottom: 4px;">PROCESS</div>
    <div style="display: flex; align-items: center; gap: 6px;">{process_badge}<span style="font-size: 12px;">{process_line}</span></div>
  </div>
  <div style="padding: 10px 12px; background: var(--bg-subtle); border-radius: 6px; border: 1px solid var(--line);">
    <div class="muted" style="font-size: 11px; margin-bottom: 4px;">SIGNING KEY</div>
    <div style="font-size: 12px;">{signer_line}</div>
  </div>
  <div style="padding: 10px 12px; background: var(--bg-subtle); border-radius: 6px; border: 1px solid var(--line);">
    <div class="muted" style="font-size: 11px; margin-bottom: 4px;">HERMES AGENT</div>
    <div style="font-size: 12px;">{agent_line}</div>
  </div>
</div>
<h3 style="margin-top: 20px;">How it got here</h3>
<p class="muted" style="font-size: 12px; margin-top: 0;">Every change of state, newest first. A quiet timeline means the node has held one state since this workspace started watching it.</p>
{timeline}
{quarantine}
{restart_hold}
<div style="margin-top: 16px; display: flex; gap: 8px; flex-wrap: wrap;">
  <form method="post" action="/nodes/{enc_id}/smoke" style="display: inline;"><button type="submit" class="btn small primary" title="Check the binary and the RPC socket now, without waiting for the next pass">Check now</button></form>
  <form method="post" action="/nodes/{enc_id}/restart" style="display: inline;"><button type="submit" class="btn small">Restart</button></form>
  <a href="/logs?node={enc_id}" class="btn small">Read the log</a>
</div>
</div>"#,
        quarantine = quarantine_panel(state, node),
        restart_hold = restart_hold_panel(state, node),
        process_badge = html::status_badge(node.status.label()),
        process_line = html::escape(&process_line(node)),
        timeline = chain_view::timeline(timeline, now_unix),
    )
}

/// The launch material a restored node arrived with, and what to do about it.
///
/// A restore writes an empty binary path deliberately — a backup must not make
/// this host execute a path chosen on another — and stashes the original in
/// `node_runtime_quarantine`. But the stash was `pub(crate)`, absent from
/// `WorkspaceQueries`, and read only by the backup exporter, so the operator
/// retyped argv the database was already holding. The word "quarantine"
/// appeared nowhere in `src/web/`, and the preflight message named a CLI flag.
fn quarantine_panel(state: &WebState, node: &NodeConfig) -> String {
    let Ok(Some(held)) = state.workspace.quarantined_runtime_spec(&node.id) else {
        return String::new();
    };
    let enc_id = html::urlencoding_lite(&node.id);
    format!(
        r#"<div class="notice warn" style="margin-top: 16px;">
  <strong>This node was restored from a backup and cannot start yet.</strong>
  Its launch command was deliberately not trusted: a backup names a path on
  whatever host produced it, and running that here without a decision would let
  an archive choose what this machine executes. This is what the backup
  recorded — check it, then save it on the edit page to accept it.
  <div class="mono" style="margin-top: 8px; font-size: 12px;">{binary}{args}</div>
  <div style="margin-top: 8px;"><a class="btn small primary" href="/nodes/{enc_id}/edit">Review and accept it</a></div>
</div>"#,
        binary = html::escape(&held.binary_path.display().to_string()),
        args = if held.args.is_empty() {
            String::new()
        } else {
            format!(" {}", html::escape(&held.args.join(" ")))
        },
    )
}

/// Whether the watchdog has been told to leave this node alone.
///
/// There is one workspace watchdog policy, read for every node, so stopping the
/// watchdog relaunching *this* node meant disabling automatic restart for the
/// whole fleet — on a page this one does not link to, and with every other node
/// left unsupervised until someone remembered to undo it.
fn restart_hold_panel(state: &WebState, node: &NodeConfig) -> String {
    let held = state.workspace.node_restart_hold(&node.id).ok().flatten();
    let enc_id = html::urlencoding_lite(&node.id);
    match held {
        Some((at, reason)) => format!(
            r#"<div class="notice warn" style="margin-top: 16px;">
  <strong>Automatic restart is held for this node.</strong>
  If it exits, NeoNexus will mark it stopped and leave it alone. The rest of the fleet is
  unaffected. Held {when}{because}.
  <form method="post" action="/nodes/{enc_id}/restart-hold" style="margin-top: 8px;"><button type="submit" class="btn small">Let the watchdog manage it again</button></form>
</div>"#,
            when = html::escape(&crate::web::time::relative(
                at,
                crate::web::time::now_unix()
            )),
            because = if reason.trim().is_empty() {
                String::new()
            } else {
                format!(" — {}", html::escape(reason.trim()))
            },
        ),
        None => format!(
            r#"<div style="margin-top: 16px;">
  <form method="post" action="/nodes/{enc_id}/restart-hold" style="display: inline;"><button type="submit" class="btn small" title="Stop the watchdog relaunching this node while you work on it. The rest of the fleet keeps its policy.">Hold automatic restart for this node</button></form>
</div>"#
        ),
    }
}

/// What the supervisor sees, said plainly.
///
/// Deliberately not a verdict. This axis answers "is there a process", and
/// fusing it with what the chain says is the mistake the tab above corrects.
fn process_line(node: &NodeConfig) -> String {
    match node.status {
        crate::types::NodeStatus::Running => node.pid.map_or_else(
            || "running; this workspace does not hold its handle".to_string(),
            |pid| format!("running as PID {pid}"),
        ),
        crate::types::NodeStatus::Starting => "starting".to_string(),
        crate::types::NodeStatus::Stopped => "stopped by an operator".to_string(),
        crate::types::NodeStatus::Error => "exited without being asked to".to_string(),
    }
}
