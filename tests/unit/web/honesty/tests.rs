//! A source-level guard against the console asserting state it never measured.
//!
//! The product shipped a page of four alarms permanently reading `● OK` —
//! including "block height stall" and "peer count low" — over metrics nothing
//! emitted; a node telemetry tab printing `1.2% (Active)`, `64.5 MB` and
//! `3.2 ms` for every running node; a fixed SVG path captioned as a 60-minute
//! chart; and a "Watchdog Armed (5 retries/60m)" badge that disagreed with the
//! policy the operator had saved.
//!
//! Each was individually plausible when written. Together they meant an
//! operator could open this console during an incident, read four green
//! alarms and a healthy instance, and stop looking. That is worse than showing
//! nothing at all, so the rule is mechanical rather than a matter of review
//! judgement: **a page may not state a verdict, a measurement or a threshold
//! that it did not compute.**
//!
//! This test greps the page sources. It cannot prove a value is bound to state,
//! but it catches the shape the regressions took — a status word or a unit
//! suffix sitting in a string literal.

use std::{fs, path::Path};

/// Status words that assert a verdict. Rendering one from a literal means the
/// verdict was decided at compile time.
const VERDICT_WORDS: [&str; 15] = [
    "● OK",
    "In Sync",
    "passed</span>",
    "Lease Valid",
    "Healthy response",
    "PID Supervised",
    "Watchdog Armed",
    "In alarm",
    // The watchdog tile on the Health page: a constant that never read
    // `WatchdogStatus`, so it stayed green with automatic restart switched off
    // and stayed green while a node sat `WatchdogExhausted` in the journal.
    "● Healthy",
    // The landing page's "AWS Health Dashboard" panel, over nothing measurable.
    "● Operational",
    // Encryption this product does not perform: no AWS SDK is in the
    // dependency tree and nothing under src/config/ encrypts anything. The
    // files *are* 0600, which is a real and different guarantee.
    "AWS-KMS",
    "AWS KMS",
    // A fused check score derived from `is_running()` alone, over four
    // genuinely independent facts.
    "Checks Passed",
    // A firewall ruleset on a product with no firewall capability whatsoever:
    // iptables, pfctl, ufw and nftables appear in this repository only inside
    // UI captions.
    "Rule Status",
    // An availability zone on a product that runs every node as a local child
    // process.
    "nexus-az-",
];

/// A page that names one of these has invented a measurement: no code path in
/// this workspace produces a CPU percentage, a byte count, a millisecond figure
/// or a provisioned-IOPS rating.
///
/// Note what is deliberately absent from this list: the duty presets still
/// carry strings like "8 vCPU · 32 GB RAM". Those are guidance about the host
/// an operator should provide, labelled "Typical host", not a claim about what
/// NeoNexus allocated — and a recommendation is not a measurement.
const INVENTED_MEASUREMENTS: [&str; 8] = [
    "1.2% (Active)",
    "64.5 MB",
    "3.2 ms",
    "3000 IOPS",
    "5 retries/60m",
    // A fixed SVG path captioned as an hour of history, on a product that
    // persisted no host metric at all. Readings are kept now and the chart is
    // drawn from them; the literal must not return.
    "M0,105 Q120,95",
    // The axis that labelled it.
    "1h Window",
    // An account number, on a product with no account.
    "0123-4567-8901",
];

fn page_sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(text) = fs::read_to_string(&path) {
                    out.push((path.display().to_string(), text));
                }
            }
        }
    }
    let mut sources = Vec::new();
    walk(Path::new("src/web/pages"), &mut sources);
    assert!(
        !sources.is_empty(),
        "no page sources found; this test is looking in the wrong place"
    );
    sources
}

/// An actor the journal does not record.
///
/// The events page derived one by grepping the message text — `if
/// message.contains("Hermes") || message.contains("probe")` — over a
/// `RuntimeEvent` that has no actor field, and printed the result in a column
/// headed "User Identity". A human action whose message happened to contain
/// the word "probe" was attributed to the AI agent; every agent action whose
/// message did not was attributed to the operator.
const INVENTED_IDENTITIES: [&str; 2] = ["arn:neo:iam::nexus:operator", "arn:neo:agent::hermes-ai"];

/// Comment lines are how a fix records what it removed, so they are not
/// evidence of the defect returning.
///
/// Both flavours: a Rust `//` line, and an HTML `<!-- -->` line, because these
/// files are mostly markup and a note left inside a template is still a note.
fn offending_lines<'a>(text: &'a str, needle: &str) -> Vec<(usize, &'a str)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("//") && !trimmed.starts_with("<!--")
        })
        .filter(|(_, line)| line.contains(needle))
        .map(|(index, line)| (index + 1, line.trim()))
        .collect()
}

#[test]
fn no_page_states_a_verdict_it_did_not_compute() {
    let mut violations = Vec::new();
    for (path, text) in page_sources() {
        for word in VERDICT_WORDS {
            for (line_number, line) in offending_lines(&text, word) {
                violations.push(format!("{path}:{line_number} — {word:?} in: {line}"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "a page asserts a verdict from a string literal. Derive it from state, \
         or render the unevaluated case explicitly — it must not read as OK:\n{}",
        violations.join("\n")
    );
}

#[test]
fn no_page_invents_a_measurement() {
    let mut violations = Vec::new();
    for (path, text) in page_sources() {
        for measurement in INVENTED_MEASUREMENTS {
            for (line_number, line) in offending_lines(&text, measurement) {
                violations.push(format!("{path}:{line_number} — {measurement:?} in: {line}"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "a page prints a measurement no code in this workspace takes. Measure it, \
         or say it is not measured:\n{}",
        violations.join("\n")
    );
}

/// The journal records no actor, so no page may name one.
#[test]
fn no_page_attributes_an_action_to_an_actor_the_journal_never_recorded() {
    let mut violations = Vec::new();
    for (path, text) in page_sources() {
        for identity in INVENTED_IDENTITIES {
            for (line_number, line) in offending_lines(&text, identity) {
                violations.push(format!("{path}:{line_number} — {identity:?} in: {line}"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "a page names an actor for an event. `RuntimeEvent` has no actor field: \
         record one, or say nothing:\n{}",
        violations.join("\n")
    );
}

/// A duty the operator never assigned must not be substituted for the absence
/// of one.
///
/// `None` rendered as "Observer" on the detail banner and the Tags tab, "Node"
/// in the fleet list and over MCP, `"observer"` in all five IaC generators, and
/// `"standard"` in the JSON API — five inventions of the same absence, one of
/// them naming a real, separately selectable duty that enables real plugins.
#[test]
fn no_page_substitutes_a_real_duty_for_no_duty() {
    let mut violations = Vec::new();
    for (path, text) in page_sources() {
        for pattern in [
            r#"unwrap_or("Observer")"#,
            r#"unwrap_or_else(|| "observer".to_string())"#,
            r#"unwrap_or_else(|| "Node".to_string())"#,
            r#"unwrap_or_else(|| "standard".to_string())"#,
        ] {
            for (line_number, line) in offending_lines(&text, pattern) {
                violations.push(format!("{path}:{line_number} — {pattern:?} in: {line}"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "a page renders `None` as a duty. A node with no duty assigned is a fact; \
         name it, do not substitute a real duty for it:\n{}",
        violations.join("\n")
    );
}
