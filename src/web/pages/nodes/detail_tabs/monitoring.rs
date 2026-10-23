//! The monitoring tab: resource history, RPC health and chain height.

use crate::{
    core::{node::NodeConfig, node_health::NodeChainView},
    logs::LogReader,
    rpc_health::RpcHealthRecord,
    supervisor::log_path_for,
    web::{html, WebState},
};

/// What a figure reads when the workspace has not taken that measurement.
///
/// A node manager is read during an incident, so every number on it has to be
/// either a measurement or visibly absent. There is no third option where a
/// plausible constant stands in — that is what ends an investigation early.
const NOT_MEASURED: &str = "not measured";

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
