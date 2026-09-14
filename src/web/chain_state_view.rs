//! Rendering a node's chain state, in one place, for every surface that shows
//! it.
//!
//! The console previously reported a node's condition in at least four
//! different vocabularies — `2/2 passed` on the home table, `In Sync` on the
//! detail page, `● OK` on the alarms page, `healthy · block 8421` in the fleet
//! row — none of which were computed from the same thing and three of which
//! were computed from nothing. Sharing the renderer is what makes a list row
//! and a detail page agree by construction rather than by review.
//!
//! Two rules run through all of it. **A number that was not measured is not
//! rendered as a number**: every absent value prints the sentence explaining
//! which kind of absence it is. And **a verdict carries its age**: "stalled for
//! twelve minutes" and "stalled, judged four seconds ago" are different claims,
//! and a surface that shows only the second invites an operator to read
//! freshness as reassurance.

use crate::{
    core::node_health::NodeChainView,
    observe::{HealthState, HealthTone, HealthTransition, NextStep, NodeHealth, ReferenceHead},
};

use super::{html, time};

/// A health badge, coloured by tone rather than by state.
///
/// `Unknown` is grey and never green: the whole point of the state machine
/// above is that "we have not looked" cannot read as a pass.
pub fn health_badge(state: HealthState) -> String {
    format!(
        r#"<span class="badge {class}" data-node-health="{key}">{label}</span>"#,
        class = tone_class(state.tone()),
        key = state.persist_key(),
        label = html::escape(state.label()),
    )
}

fn tone_class(tone: HealthTone) -> &'static str {
    match tone {
        HealthTone::Good => "health-good",
        HealthTone::Working => "health-working",
        HealthTone::Warning => "health-warning",
        HealthTone::Bad => "health-bad",
        HealthTone::Neutral => "health-neutral",
    }
}

/// The badge a node that has never been judged gets.
///
/// Deliberately its own sentence rather than a blank cell. A fleet that has
/// just started has no verdicts, and an empty column there reads as "nothing
/// wrong" when it means "nothing known".
pub fn not_judged_badge() -> String {
    format!(
        r#"{badge} <span class="muted" style="font-size: 11px;">no verdict yet</span>"#,
        badge = health_badge(HealthState::Unknown),
    )
}

/// One table cell: what state the node is in, and how long it has been there.
pub fn health_cell(view: &NodeChainView, now_unix: u64) -> String {
    let Some(health) = &view.health else {
        return not_judged_badge();
    };
    format!(
        r#"{badge}<div class="muted" style="font-size: 11px;">{held}</div>"#,
        badge = health_badge(health.state),
        held = html::escape(&held_for(health, now_unix)),
    )
}

/// How long the current state has held, and how fresh the judgement is.
fn held_for(health: &NodeHealth, now_unix: u64) -> String {
    let held = duration_label(health.held_for_seconds(now_unix));
    let age = health.evaluated_seconds_ago(now_unix);
    // Only worth saying when the judgement is old enough that an operator
    // should discount it; on a healthy loop it is a handful of seconds and
    // printing it every row is noise.
    if age > STALE_VERDICT_SECONDS {
        format!("for {held} · judged {} ago", duration_label(age))
    } else {
        format!("for {held}")
    }
}

/// Beyond this, a verdict's own age is worth showing beside it.
///
/// Twice the longest default monitoring interval an operator is likely to have
/// set without meaning "watch this loosely".
const STALE_VERDICT_SECONDS: u64 = 120;

/// One table cell: where the node is on its chain.
///
/// Shows the height and — only where there is something to compare against —
/// how far behind the chain's head it is. A node alone on its chain shows no
/// lag at all rather than `0 behind`, which would be a number that means
/// nothing while reading as reassurance.
pub fn chain_cell(view: &NodeChainView) -> String {
    let Some(latest) = &view.latest else {
        return r#"<span class="muted">not checked yet</span>"#.to_string();
    };
    let height = latest.block_height.render(|height| format!("{height}"));
    let detail = match (&view.reference, view.derived.head_lag) {
        (ReferenceHead::Known { source, .. }, Some(0)) => {
            format!("at the head of {}", html::escape(source))
        }
        (ReferenceHead::Known { source, .. }, Some(lag)) => {
            format!("{lag} behind {}", html::escape(source))
        }
        _ => match view.derived.header_gap {
            // With nothing to compare against, the node's own header chain is
            // still a witness: headers run ahead of blocks while it catches up.
            Some(gap) if gap > 0 => format!("{gap} behind its own headers"),
            _ => "nothing to compare against".to_string(),
        },
    };
    let measured = latest.block_height.is_known();
    format!(
        r#"<div class="{class}">{height}</div><div class="muted" style="font-size: 11px;">{detail}</div>"#,
        class = if measured { "mono" } else { "muted" },
        height = html::escape(&height),
    )
}

/// The full verdict, with its reason, its suspected cause and one way forward.
pub fn verdict_panel(view: &NodeChainView, now_unix: u64) -> String {
    let Some(health) = &view.health else {
        return html::notice(
            "info",
            "This node has not been judged yet. The observation loop writes a \
             verdict on its next pass.",
        );
    };
    let cause = health
        .cause
        .as_deref()
        .map(|cause| {
            format!(
                r#"<p class="muted" style="margin: 6px 0 0;">{}</p>"#,
                html::escape(cause)
            )
        })
        .unwrap_or_default();
    let scope = health
        .scope
        .map(|scope| {
            format!(
                r#" <span class="badge">affects {}</span>"#,
                html::escape(scope.label())
            )
        })
        .unwrap_or_default();
    format!(
        r#"<section class="surface">
<div class="section-head"><h2>Chain health</h2><span class="muted" style="font-size: 11px;">judged {judged}</span></div>
<div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">{badge}{scope}<span class="muted" style="font-size: 12px;">held for {held}</span></div>
<p style="margin: 10px 0 0;">{reason}</p>
{cause}
<div style="margin-top: 12px;">{next}</div>
{numbers}
</section>"#,
        judged = time::relative(health.evaluated_at_unix, now_unix),
        badge = health_badge(health.state),
        held = html::escape(&duration_label(health.held_for_seconds(now_unix))),
        reason = html::escape(&health.reason),
        next = next_step(&health.next),
        numbers = measurements(view),
    )
}

/// The one thing to do about it.
///
/// A step this console can take is a button. One it cannot is a sentence that
/// says what can — never a link that goes nowhere, which is how advice becomes
/// a dead end at three in the morning.
fn next_step(next: &NextStep) -> String {
    match next {
        NextStep::Here { label, href } => format!(
            r#"<a class="btn small primary" href="{href}">{label}</a>"#,
            href = html::escape(href),
            label = html::escape(label),
        ),
        NextStep::External { text } => format!(
            r#"<p class="muted" style="margin: 0;">{}</p>"#,
            html::escape(text)
        ),
    }
}

/// The readings the verdict was drawn from.
///
/// Every one of them is an `Observation` or an `Option`, and the absent case
/// prints why it is absent. A node whose client does not implement
/// `getconnectioncount` shows "this client does not implement
/// getconnectioncount", not a peer count of zero — which is a state that pages
/// someone.
fn measurements(view: &NodeChainView) -> String {
    let Some(latest) = &view.latest else {
        return String::new();
    };
    let rows = [
        ("Block height", latest.block_height.cell(|h| h.to_string())),
        (
            "Header height",
            latest.header_height.cell(|h| h.to_string()),
        ),
        (
            "Peers",
            latest.peers_connected.cell(|peers| peers.to_string()),
        ),
        (
            "RPC round trip",
            latest
                .head_latency_ms
                .map(|ms| format!("{ms} ms"))
                .unwrap_or_else(|| "not measured".to_string()),
        ),
        (
            "Blocks per minute",
            optional(view.derived.blocks_per_minute.map(|rate| format!("{rate:.1}"))),
        ),
        (
            "Newest block age",
            if view.derived.clock_suspect {
                "dated in the future; a clock is wrong".to_string()
            } else {
                optional(
                    view.derived
                        .chain_lag_seconds
                        .map(|lag| duration_label(lag.max(0) as u64)),
                )
            },
        ),
        (
            "Height last moved",
            optional(
                view.derived
                    .height_unchanged_seconds
                    .map(|seconds| format!("{} ago", duration_label(seconds))),
            ),
        ),
        (
            "Network magic",
            latest.observed_magic.cell(|magic| magic.to_string()),
        ),
        (
            "Client",
            latest.client_version.cell(|version| version.clone()),
        ),
        (
            "Mempool",
            match (
                latest.mempool_verified.value(),
                view.derived.mempool_utilisation,
            ) {
                (Some(verified), Some(share)) => {
                    format!("{verified} queued · {:.0}% of capacity", share * 100.0)
                }
                (Some(verified), None) => format!("{verified} queued"),
                (None, _) => latest.mempool_verified.cell(|depth| depth.to_string()),
            },
        ),
    ]
    .into_iter()
    .map(|(label, value)| {
        format!(
            r#"<div><div class="muted" style="font-size: 11px;">{label}</div><div class="mono">{value}</div></div>"#,
            label = html::escape(label),
            value = html::escape(&value),
        )
    })
    .collect::<String>();
    format!(
        r#"<div class="grid" style="grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 10px; margin-top: 14px;">{rows}</div>"#
    )
}

/// A value that may not have been derivable, in the same voice as an
/// `Observation` — never an em dash on its own and never a zero.
fn optional(value: Option<String>) -> String {
    value.unwrap_or_else(|| "not enough history".to_string())
}

/// The record of how a node got here.
pub fn timeline(transitions: &[HealthTransition], now_unix: u64) -> String {
    if transitions.is_empty() {
        return html::empty_state(
            "No changes recorded",
            "This node's state has not changed since the workspace started watching it.",
            "",
        );
    }
    let rows = transitions
        .iter()
        .map(|transition| {
            format!(
                r#"<li><div><strong>{to}</strong><p>{summary}</p></div><span class="muted">{when}</span></li>"#,
                to = html::escape(transition.to.label()),
                summary = html::escape(&transition.summary()),
                when = html::escape(&time::relative(transition.at_unix, now_unix)),
            )
        })
        .collect::<String>();
    format!(r#"<ul class="activity-list">{rows}</ul>"#)
}

/// Seconds as a person would say them. Defined in the core facade so the
/// console and the CLI cannot round a duration two different ways.
pub use crate::core::node_health::duration_label;

#[cfg(test)]
#[path = "../../tests/unit/web/chain_state_view/tests.rs"]
mod tests;
