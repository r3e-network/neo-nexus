//! AWS EC2-style tabbed studio views, summary banner, and cloud telemetry cards.

use crate::{
    agents::HermesAgentAssociation,
    config::ConfigGenerator,
    core::{node::NodeConfig, node_health::NodeChainView},
    logs::LogReader,
    observe::HealthTransition,
    roles::NodeRole,
    rpc_health::RpcHealthRecord,
    signing::SignerKeyRef,
    supervisor::log_path_for,
    web::{chain_state_view as chain_view, html, WebState},
};

use super::{binding::signer_binding, iac_spec::iac_spec_card};

/// What a figure reads when the workspace has not taken that measurement.
///
/// A node manager is read during an incident, so every number on it has to be
/// either a measurement or visibly absent. There is no third option where a
/// plausible constant stands in — that is what ends an investigation early.
pub(crate) const NOT_MEASURED: &str = "not measured";

/// AWS EC2-styled top instance summary ribbon.
pub fn summary_banner(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
) -> String {
    let rpc_text = if node.rpc_port == 0 {
        "Disabled".to_string()
    } else {
        format!("http://127.0.0.1:{}", node.rpc_port)
    };
    let p2p_text = format!("127.0.0.1:{}", node.p2p_port);
    // `None` is not `Observer`. Observer is a real, separately selectable duty
    // with real plugin effects, and substituting it for "no duty assigned"
    // meant an operator who chose the "P2P Gossip Relay" preset — which
    // resolves to no duty at all — saw a node described as an Observer, then
    // found the editor pre-selecting a different duty again on reopen.
    let role_label = role.map_or("No duty assigned", NodeRole::label);
    let instance_type = format!(
        "{}-{}",
        node.node_type,
        role.map(|r| r.slug()).unwrap_or("node")
    );
    let signer_text = match signer {
        Some(s) => format!("{}/{}", s.backend_id, s.key_id),
        None => "Unassigned".to_string(),
    };

    let status_class = match node.status {
        crate::types::NodeStatus::Running => "running",
        crate::types::NodeStatus::Starting => "starting",
        crate::types::NodeStatus::Stopped => "stopped",
        crate::types::NodeStatus::Error => "danger",
    };

    // The two badges beside the status pill named a zone and a VPC — the same
    // strings for every node, on a product that runs every node as a local
    // child process. There is no zone and no VPC; what there is, is a client
    // and a chain, so that is what they carry.
    format!(
        r#"<div class="panel aws-summary-card" style="margin-bottom: 16px; padding: 16px 20px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; border-bottom: 1px solid var(--line); padding-bottom: 10px; flex-wrap: wrap; gap: 8px;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <span class="status-dot {status_class}" style="width: 10px; height: 10px;"></span>
                    <h2 style="margin: 0; font-size: 16px;">Instance Summary: <span class="mono" style="color: var(--jade);">{id}</span> ({name})</h2>
                </div>
                <div style="display: flex; gap: 6px; align-items: center;">
                    <span class="badge {status_class}">● {status}</span>
                    <span class="badge" style="background: rgba(255,255,255,0.06);">{client}</span>
                    <span class="badge" style="background: rgba(255,255,255,0.06);">{network}</span>
                </div>
            </div>
            <div class="grid" style="grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 14px;">
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Instance ID</div>
                    <div class="mono" style="font-size: 13px; font-weight: 500;">{id}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Instance Type</div>
                    <div style="font-size: 13px; font-weight: 500;">t3.{instance_type}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Role Profile</div>
                    <div style="font-size: 13px; font-weight: 500;">{role_label}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Public IPv4 / RPC</div>
                    <div class="mono" style="font-size: 12px;">{rpc_text}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">Private IP / P2P</div>
                    <div class="mono" style="font-size: 12px;">{p2p_text}</div>
                </div>
                <div>
                    <div class="muted" style="font-size: 11px; text-transform: uppercase; font-weight: 600; margin-bottom: 4px;">IAM Signer Lease</div>
                    <div class="mono" style="font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="{signer_title}">{signer_text}</div>
                </div>
            </div>
        </div>"#,
        id = html::escape(&node.id),
        name = html::escape(&node.name),
        status_class = status_class,
        status = node.status.label(),
        client = html::escape(&node.node_type.to_string()),
        network = html::escape(&node.network.to_string()),
        instance_type = html::escape(&instance_type),
        role_label = html::escape(role_label),
        rpc_text = html::escape(&rpc_text),
        p2p_text = html::escape(&p2p_text),
        signer_title = html::escape(&signer_text),
        signer_text = html::escape(&signer_text),
    )
}

/// Tab 1: Details View (AWS EC2 General Details)
pub fn render_tab_details(node: &NodeConfig) -> String {
    let rpc_value = if node.rpc_port == 0 {
        "Disabled (Zero attack surface)".to_string()
    } else {
        node.rpc_port.to_string()
    };
    let ws_value = if node.rpc_port == 0 {
        "Disabled".to_string()
    } else {
        node.ws_port
            .map_or_else(|| "Not configured".to_string(), |p| p.to_string())
    };
    // The same redaction the launch log and the support bundle apply to this
    // exact expression. Node arguments are operator-supplied, so a `--password`
    // or `--wif` lands here, and this page is served to any session.
    let command = crate::argv::format_command(
        &node.binary_path,
        &crate::redaction::redact_sensitive_args(&node.args),
    );

    let facts = [
        ("Instance Identifier", node.id.clone()),
        ("Instance Name", node.name.clone()),
        ("AMI / Client Engine", node.node_type.to_string()),
        ("Network Topology", node.network.to_string()),
        ("Storage Engine Driver", node.storage_engine.to_string()),
        ("Binary Image Path", node.binary_path.display().to_string()),
        (
            "Client Release Version",
            if node.runtime_version.is_empty() {
                "native-latest".to_string()
            } else {
                node.runtime_version.clone()
            },
        ),
        ("JSON-RPC 2.0 Port", rpc_value),
        ("P2P Mesh Port", node.p2p_port.to_string()),
        ("WebSocket Port", ws_value),
        (
            "Host Process ID (PID)",
            node.pid
                .map_or_else(|| "Not running / Supervised".to_string(), |p| p.to_string()),
        ),
        // These read "x86_64 Native Sandbox" and "NeoNexus Workbench Daemon"
        // over a plain `Command::new(binary_path)`. There is no sandbox and no
        // hypervisor — the client runs as a child process with this user's
        // privileges — and on an arm64 host the architecture was also simply
        // wrong. That combination materially changes an operator's threat model
        // for an untrusted client binary, which is the one place a console must
        // not flatter itself.
        (
            "How it runs",
            format!(
                "child process of NeoNexus on this host, {}, with this user's privileges",
                std::env::consts::ARCH
            ),
        ),
    ];
    let rows = facts
        .iter()
        .map(|(label, value)| html::row(&[html::cell(label), html::cell(value)]))
        .collect::<Vec<_>>();
    let table = html::table(&["Attribute", "Configuration Value"], &rows);

    format!(
        r#"<div style="margin-top: 12px;">
            <h3>Instance Attributes</h3>
            {table}
            <h3 style="margin-top: 18px;">Host Process Launch Command</h3>
            {command}
        </div>"#,
        table = table,
        command = html::text_block(&command),
    )
}

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

/// Tab 3: Monitoring & CloudWatch Telemetry
pub fn render_tab_monitoring(
    state: &WebState,
    node: &NodeConfig,
    history: &[RpcHealthRecord],
    view: Option<&NodeChainView>,
) -> String {
    let enc_id = html::urlencoding_lite(&node.id);
    let latest_block = view
        .and_then(|view| view.latest.as_ref())
        .map(|latest| latest.block_height.cell(|height| height.to_string()))
        .unwrap_or_else(|| {
            history
                .first()
                .and_then(|record| record.block_count)
                .map_or_else(|| NOT_MEASURED.to_string(), |block| block.to_string())
        });

    // Read this node's actual process sample. These numbers used to be the
    // literals "1.2% (Active)", "64.5 MB" and "3.2 ms", selected only by
    // `node.status.is_running()` — so every running node in the fleet reported
    // the same three values, and an operator looking at a node that was
    // thrashing saw a healthy one. The real per-process figures were already
    // being collected and rendered honestly on the Health page.
    let sample = crate::web::pages::metrics_page::collect_snapshot(state).ok();
    let process = sample
        .as_ref()
        .and_then(|snapshot| snapshot.node_process(&node.id));
    let cpu_load = process.map_or_else(
        || NOT_MEASURED.to_string(),
        |metrics| format!("{:.1}%", metrics.cpu_usage_percent),
    );
    let mem_usage = process.map_or_else(
        || NOT_MEASURED.to_string(),
        |metrics| crate::core::operations::format_bytes(metrics.memory_bytes),
    );
    // The head call is timed on every round, so this is a real figure — and it
    // means one thing, because it is always the same call. It stood at the
    // literal "3.2 ms" for every running node until the sampler started
    // measuring it.
    let latency = view
        .and_then(|view| view.latest.as_ref())
        .and_then(|latest| latest.head_latency_ms)
        .map_or_else(|| NOT_MEASURED.to_string(), |ms| format!("{ms} ms"));

    let log_path = log_path_for(state.workspace_child_dir("logs"), node);
    let log_terminal = match LogReader::snapshot(&log_path, 32 * 1024) {
        Ok(snapshot) if !snapshot.lines.is_empty() => {
            let total_lines = snapshot.lines.len();
            let start_index = total_lines.saturating_sub(12);
            let last_lines = snapshot.lines.iter().enumerate().skip(start_index);
            let mut rendered = String::new();
            for (idx, line) in last_lines {
                rendered.push_str(&format!(
                    r#"<div style="font-family: monospace; font-size: 12px; line-height: 1.5; color: #d1d5db;"><span class="muted" style="display: inline-block; width: 40px; user-select: none;">{:>4}</span> {}</div>"#,
                    idx + 1,
                    html::escape(line),
                ));
            }
            rendered
        }
        _ => r#"<div class="muted" style="padding: 12px; font-size: 12px;">No active console logs captured yet. Instance log buffer is clean.</div>"#.to_string(),
    };

    // The chart that stood here drew a fixed SVG path — the same curve for
    // every node, at every moment — under the caption "1m Interval". Sampled
    // rounds are now retained per node, so there is a real series behind the
    // rate below; what there is still no code to do is draw one.
    let instance_chart = html::note(&match view.and_then(|view| view.derived.blocks_per_minute) {
        Some(rate) => format!(
            "Producing {rate:.1} blocks per minute across the sampled window. No plotting code exists yet; the Health tab lists the readings behind this."
        ),
        None => "Not enough sampled history yet to state a block rate. The Health tab lists what has been read so far.".to_string(),
    });

    // Read the operator's actual restart policy. The badge here used to read a
    // constant "Watchdog Armed (5 retries/60m)", which disagreed with the
    // policy the Settings page saves and ignored the enabled flag entirely —
    // so it stayed green with automatic restart switched off. The four alarm
    // cards beside it were likewise unconditional: one of them reported
    // "● OK (Lease Valid)" from a function that is handed no signer at all.
    let watchdog_panel = match state.workspace.load_watchdog_policy() {
        Ok(policy) if policy.enabled => format!(
            r#"<div class="panel" style="padding: 14px; border: 1px solid var(--line); border-radius: 6px; margin-bottom: 18px;">
                <strong style="font-size: 14px;">Automatic restart</strong>
                <div class="muted" style="font-size: 12px; margin-top: 4px;">On an unclean exit this node is restarted up to {attempts} times, backing off from {base}s to at most {max}s{jitter}. This is the workspace-wide policy; it is not set per node.</div>
                <div class="muted" style="font-size: 12px; margin-top: 6px;">The watchdog acts on process exit only. A node that keeps running but stops producing blocks is reported on the Health tab and in the attention queue, but is not restarted automatically — restarting a stalled node is a judgement call, not a reflex.</div>
                <div style="margin-top: 8px;"><a class="btn small" href="/settings">Change restart policy</a></div>
            </div>"#,
            attempts = policy.max_restart_attempts,
            base = policy.base_delay.as_secs(),
            max = policy.max_delay.as_secs(),
            jitter = if policy.jitter_enabled {
                ", with jitter"
            } else {
                ""
            },
        ),
        Ok(_) => format!(
            r#"<div class="panel" style="padding: 14px; border: 1px solid var(--line); border-radius: 6px; margin-bottom: 18px;">
                <strong style="font-size: 14px;">Automatic restart is off</strong>
                <div class="muted" style="font-size: 12px; margin-top: 4px;">{name} will not be restarted automatically if its process exits.</div>
                <div style="margin-top: 8px;"><a class="btn small" href="/settings">Change restart policy</a></div>
            </div>"#,
            name = html::escape(&node.name),
        ),
        Err(error) => html::note(&format!("Restart policy could not be read: {error}")),
    };

    format!(
        r#"<div style="margin-top: 12px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; flex-wrap: wrap; gap: 8px;">
                <div>
                    <h3 style="margin: 0 0 2px 0;">Telemetry</h3>
                    <div class="muted" style="font-size: 12px;">What this workspace has measured about {name}. Figures marked <em>not measured</em> are not collected — they are not zero.</div>
                </div>
                <div style="display: flex; gap: 8px;">
                    <a href="/api/nodes/{enc_id}/metrics" class="btn small" target="_blank" style="text-decoration: none;">Node metrics (JSON)</a>
                    <a href="/monitor" class="btn small" style="text-decoration: none;">Fleet health</a>
                </div>
            </div>
            <div class="grid" style="grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 14px; margin-bottom: 18px;">
                <div class="panel" style="padding: 14px; background: var(--bg-subtle); border-radius: 6px; border: 1px solid var(--line);">
                    <div class="muted" style="font-size: 11px; text-transform: uppercase;">Process CPU</div>
                    <div style="font-size: 20px; font-weight: 700; margin-top: 4px;">{cpu_load}</div>
                    <div class="muted" style="font-size: 11px; margin-top: 2px;">Share of one core, this process</div>
                </div>
                <div class="panel" style="padding: 14px; background: var(--bg-subtle); border-radius: 6px; border: 1px solid var(--line);">
                    <div class="muted" style="font-size: 11px; text-transform: uppercase;">Process memory</div>
                    <div style="font-size: 20px; font-weight: 700; margin-top: 4px;">{mem_usage}</div>
                    <div class="muted" style="font-size: 11px; margin-top: 2px;">Resident set</div>
                </div>
                <div class="panel" style="padding: 14px; background: var(--bg-subtle); border-radius: 6px; border: 1px solid var(--line);">
                    <div class="muted" style="font-size: 11px; text-transform: uppercase;">Block height</div>
                    <div class="mono" style="font-size: 20px; font-weight: 700; margin-top: 4px;">{latest_block}</div>
                    <div class="muted" style="font-size: 11px; margin-top: 2px;">{height_detail}</div>
                </div>
                <div class="panel" style="padding: 14px; background: var(--bg-subtle); border-radius: 6px; border: 1px solid var(--line);">
                    <div class="muted" style="font-size: 11px; text-transform: uppercase;">RPC latency</div>
                    <div style="font-size: 20px; font-weight: 700; margin-top: 4px;">{latency}</div>
                    <div class="muted" style="font-size: 11px; margin-top: 2px;">Round trip of the head call, timed every round.</div>
                </div>
            </div>
            {instance_chart}
            {watchdog_panel}
            <div class="panel" style="padding: 16px; background: #0f141c; border: 1px solid var(--line); border-radius: 6px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; flex-wrap: wrap; gap: 8px;">
                    <div>
                        <div style="display: flex; align-items: center; gap: 8px;">
                            <span class="aws-log-live-dot"></span>
                            <strong style="font-size: 14px; color: #fff;">CloudWatch Logs · Console Output Stream</strong>
                            <span class="badge running" style="font-size: 10px;">● LIVE STREAM</span>
                        </div>
                        <div class="muted" style="font-size: 12px; margin-top: 2px;">Live tail of supervised stdout/stderr output for instance i-{enc_id}.</div>
                    </div>
                    <a href="/logs?node={enc_id}" class="btn small" style="text-decoration: none;">View Full CloudWatch Logs →</a>
                </div>
                <div style="padding: 12px; background: #07090e; border-radius: 4px; border: 1px solid rgba(255,255,255,0.06); max-height: 260px; overflow-y: auto;">
                    {log_terminal}
                </div>
            </div>
        </div>"#,
        enc_id = enc_id,
        name = html::escape(&node.name),
        cpu_load = cpu_load,
        mem_usage = mem_usage,
        latest_block = html::escape(&latest_block),
        height_detail = html::escape(&height_detail(view)),
        latency = latency,
        instance_chart = instance_chart,
        watchdog_panel = watchdog_panel,
        log_terminal = log_terminal,
    )
}

/// What the height beside it is measured against.
///
/// The caption read "Not compared against the network head" because nothing
/// resolved one. A node now has a reference head whenever it shares a chain
/// with another node in this workspace, and says so where it does not.
fn height_detail(view: Option<&NodeChainView>) -> String {
    match view.map(|view| (&view.reference, view.derived.head_lag)) {
        Some((crate::observe::ReferenceHead::Known { source, .. }, Some(0))) => {
            format!("Level with {source}, the highest node on this chain.")
        }
        Some((crate::observe::ReferenceHead::Known { source, .. }, Some(lag))) => {
            format!("{lag} blocks behind {source}, the highest node on this chain.")
        }
        _ => "No other node in this workspace is on this chain, so there is nothing to compare against.".to_string(),
    }
}

/// The managed config, as text, redacted.
///
/// `/config` rendered a **path**. `ConfigGenerator::render_for_node` was never
/// called from `src/web/` at all — only drift, export and diagnostics used it —
/// so the file an operator needs to read while diagnosing a crash loop was the
/// one thing the console would not show them. The log-diagnosis remediation
/// button even routed back to `/config`, which showed the path again.
///
/// Redacted through the same helper `/logs` uses, because these configs embed
/// plaintext wallet unlock passwords.
pub fn render_tab_config(node: &NodeConfig, plugins: &[crate::catalog::PluginState]) -> String {
    let rendered = ConfigGenerator::render_for_node(node, plugins);
    let body = match rendered {
        Ok(rendered) => format!(
            r#"<p class="muted" style="font-size: 12px;">This is what NeoNexus writes for {name} — {format} — with secrets masked. It is written on every start, so an edit made by hand here is replaced the next time this node launches.</p>
{text}"#,
            name = html::escape(&node.name),
            format = html::escape(rendered.format.label()),
            text = html::text_block(&crate::redaction::redact_sensitive_text(&rendered.text)),
        ),
        Err(error) => html::notice(
            "warn",
            &format!(
                "This node's config cannot be rendered, which is also why it cannot start: {error:#}"
            ),
        ),
    };
    format!(
        r#"<div style="margin-top: 12px;"><h3>Managed configuration</h3>{body}<div style="margin-top: 12px;"><a class="btn small" href="/config">Every node's config</a></div></div>"#
    )
}

/// Tab 4: the ports, and what NeoNexus asks the client to do with them.
///
/// This was headed "Inbound Security Group Rules (Firewall Ruleset)" with a
/// "Rule Status: Open" column, over a product that has no firewall capability
/// whatsoever — `iptables`, `pfctl`, `ufw` and `nftables` appear in this
/// repository only inside UI captions. The CIDRs were not arbitrary: they
/// mirror the binds NeoNexus writes into the managed config. But they were
/// rendered as literals, so they went stale the moment an operator's own
/// arguments diverged — and a `--config` argument suppresses the managed
/// config entirely.
///
/// What the table can state truthfully is what NeoNexus *asks for*, and where
/// the client is free to ignore it.
pub fn render_tab_networking(state: &WebState, node: &NodeConfig) -> String {
    let overridden = node
        .args
        .iter()
        .any(|arg| arg == "--config" || arg.starts_with("--config=") || arg == "--config-file");

    let requested = |what: &str, port: Option<u16>, bind: &str, note: String| {
        html::row(&[
            html::cell(what),
            html::cell(&port.map_or_else(|| "—".to_string(), |port| port.to_string())),
            html::cell(bind),
            html::cell(&note),
        ])
    };

    let rpc_row = if node.rpc_port == 0 {
        requested(
            "JSON-RPC",
            None,
            "not requested",
            "this node exposes no RPC, so nothing in this workspace can ask it anything"
                .to_string(),
        )
    } else {
        requested(
            "JSON-RPC",
            Some(node.rpc_port),
            "127.0.0.1",
            "loopback only; reachable from this host".to_string(),
        )
    };
    let p2p_row = requested(
        "P2P",
        Some(node.p2p_port),
        "0.0.0.0",
        "all interfaces; peers must be able to reach it".to_string(),
    );
    let ws_row = match node.ws_port {
        // Honest about G23: a WS port is validated, reserved against other
        // nodes and rendered as an endpoint — and for every client except
        // neox-geth and the neo-cli sidecar, nothing ever opens it.
        Some(ws) if ws_is_requested(node) => requested(
            "WebSocket",
            Some(ws),
            "127.0.0.1",
            "loopback only".to_string(),
        ),
        Some(ws) => requested(
            "WebSocket",
            Some(ws),
            "not requested",
            format!(
                "reserved for this node, but NeoNexus emits no WebSocket setting for {}, so the port will not be listening",
                node.node_type
            ),
        ),
        None => requested("WebSocket", None, "not requested", "not configured".to_string()),
    };

    let table = html::table(
        &[
            "Service",
            "Port",
            "NeoNexus asks it to bind",
            "What that means",
        ],
        &[rpc_row, p2p_row, ws_row],
    );

    let caveat = if overridden {
        html::notice(
            "warn",
            "This node's arguments include --config, which makes the client read a file \
             NeoNexus did not write. The binds above are what NeoNexus would have asked for; \
             what the node is actually doing is in that file.",
        )
    } else {
        html::note(
            "These are the binds NeoNexus writes into the managed config. Arguments you add \
             pass to the client verbatim and can override any of them. NeoNexus opens no \
             firewall and closes none: the host's own rules decide what is reachable.",
        )
    };

    let endpoints = super::detail::endpoints_card(state, node);

    format!(
        r#"<div style="margin-top: 12px;">
            <h3>Ports</h3>
            {table}
            {caveat}
            <div style="margin-top: 16px;">
                {endpoints}
            </div>
        </div>"#
    )
}

/// Whether any launch path for this client emits a WebSocket setting.
///
/// neox-geth's generated config carries `ws_host`/`ws_port`, and the neo-cli
/// sidecar derives one. Nothing else does, and no `--ws` flag exists anywhere
/// in this repository — so for the remaining clients a configured WebSocket
/// port is a reservation, not a listener.
fn ws_is_requested(node: &NodeConfig) -> bool {
    matches!(
        node.node_type,
        crate::types::NodeType::NeoXGeth | crate::types::NodeType::NeoCli
    )
}

/// Tab 5: Security & IAM View
pub fn render_tab_security(
    state: &WebState,
    node: &NodeConfig,
    signer: Option<&SignerKeyRef>,
    new_token: Option<&str>,
) -> String {
    let signer_html = signer_binding(state, node, signer);
    let hermes_html = super::detail::hermes_agent_card(state, node, new_token);

    format!(
        r#"<div style="margin-top: 12px;">
            <div style="margin-bottom: 16px;">
                {signer_html}
            </div>
            <div>
                {hermes_html}
            </div>
        </div>"#,
        signer_html = signer_html,
        hermes_html = hermes_html,
    )
}
/// Tab 7: AWS Resource Tags
pub fn render_tab_tags(node: &NodeConfig, role: Option<NodeRole>) -> String {
    let rows = vec![
        html::row(&[html::cell("Name"), html::cell(&node.name)]),
        // "Environment: Production" was hardcoded for every node, testnet
        // included, under a claim that these drive cost allocation and access
        // control. There is no environment concept in the workspace and no way
        // to set one, so the row asserted something false about every node.
        html::row(&[html::cell("Network"), html::cell(&node.network.to_string())]),
        html::row(&[
            html::cell("Role"),
            html::cell(role.map_or("No duty assigned", NodeRole::label)),
        ]),
        html::row(&[
            html::cell("ClientEngine"),
            html::cell(&node.node_type.to_string()),
        ]),
        html::row(&[
            html::cell("ManagedBy"),
            html::cell("NeoNexus-CloudControlPlane"),
        ]),
        html::row(&[
            html::cell("InstanceId"),
            html::raw_cell(&format!(
                r#"<span class="mono">{}</span>"#,
                html::escape(&node.id)
            )),
        ]),
    ];
    let tag_table = html::table(&["Key", "Value"], &rows);

    format!(
        r#"<div style="margin-top: 12px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; flex-wrap: wrap; gap: 8px;">
                <div>
                    <h3 style="margin: 0 0 2px 0;">Resource Tags (Metadata Taxonomy)</h3>
                    <div class="muted" style="font-size: 12px;">Key-value pairs assigned to this instance for enterprise governance, cost allocation, and access control.</div>
                </div>
            </div>
            {tag_table}
        </div>"#,
        tag_table = tag_table,
    )
}

/// Tab 8: Launch Templates & IaC Export
pub fn render_tab_iac(
    node: &NodeConfig,
    role: Option<NodeRole>,
    signer: Option<&SignerKeyRef>,
    assoc: &HermesAgentAssociation,
) -> String {
    iac_spec_card(node, role, signer, Some(assoc))
}
