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
const VERDICT_WORDS: [&str; 8] = [
    "● OK",
    "In Sync",
    "passed</span>",
    "Lease Valid",
    "Healthy response",
    "PID Supervised",
    "Watchdog Armed",
    "In alarm",
];

/// A page that names one of these has invented a measurement: no code path in
/// this workspace produces a CPU percentage, a byte count, a millisecond figure
/// or a provisioned-IOPS rating.
///
/// Note what is deliberately absent from this list: the duty presets still
/// carry strings like "8 vCPU · 32 GB RAM". Those are guidance about the host
/// an operator should provide, labelled "Typical host", not a claim about what
/// NeoNexus allocated — and a recommendation is not a measurement.
const INVENTED_MEASUREMENTS: [&str; 5] = [
    "1.2% (Active)",
    "64.5 MB",
    "3.2 ms",
    "3000 IOPS",
    "5 retries/60m",
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

/// Comment lines are how a fix records what it removed, so they are not
/// evidence of the defect returning.
fn offending_lines<'a>(text: &'a str, needle: &str) -> Vec<(usize, &'a str)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim_start().starts_with("//"))
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
