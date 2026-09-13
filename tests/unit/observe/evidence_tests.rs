use super::*;

fn sample_evidence() -> Evidence {
    Evidence::recorded(
        "getblockcount",
        "result",
        "6245100",
        "http://127.0.0.1:10332",
        1_770_000_000,
    )
}

/// Evidence is the raw answer, not an interpretation of it. When a derived
/// figure later looks wrong, this is what separates a parsing bug from a node
/// problem.
#[test]
fn evidence_carries_the_call_the_field_and_what_came_back() {
    let evidence = sample_evidence();
    assert_eq!(evidence.method(), "getblockcount");
    assert_eq!(evidence.field(), "result");
    assert_eq!(evidence.value(), "6245100");
    assert_eq!(evidence.endpoint(), "http://127.0.0.1:10332");
    assert_eq!(evidence.sampled_at_unix(), 1_770_000_000);
    assert_eq!(evidence.summary(), "getblockcount.result = 6245100");
}

/// The whole point of the type. Every absent rendering has to say something an
/// operator can act on, and none of them may be blank, zero, or a value.
#[test]
fn no_absent_observation_renders_as_a_value() {
    let absences: Vec<Observation<u64>> = vec![
        Observation::Unknown(NotSampled::NeverSampled),
        Observation::Unknown(NotSampled::SamplingDisabled),
        Observation::Unknown(NotSampled::Stale { age_seconds: 900 }),
        Observation::Unknown(NotSampled::MethodUnsupported {
            method: "getversion",
        }),
        Observation::Unknown(NotSampled::CallFailed {
            method: "getblockcount",
            detail: "connection refused".to_string(),
        }),
        Observation::Unanswerable("neo-go exposes no plugin list"),
    ];
    for absent in absences {
        let rendered = absent.render(|height| height.to_string());
        assert!(
            !rendered.trim().is_empty(),
            "an absent observation rendered as nothing"
        );
        assert!(
            rendered.parse::<f64>().is_err(),
            "an absent observation rendered as the number {rendered}, which is \
             indistinguishable from a measurement"
        );
        // It has to read as prose explaining the gap, not as a value with
        // units. An absence may quote a duration ("900s ago"), so the test is
        // that words are present, not that digits are absent.
        assert!(
            rendered.chars().filter(char::is_ascii_alphabetic).count() >= 8,
            "an absent observation rendered {rendered:?}, which does not \
             explain anything to an operator"
        );
        assert!(absent.value().is_none());
        assert!(absent.evidence().is_none());
        assert!(!absent.is_known());
    }
}

/// "This client cannot be asked" and "we have not asked yet" lead to different
/// actions: one waits, the other never resolves. Rendering both as an em dash
/// tells an operator to keep waiting for one of them forever.
#[test]
fn unanswerable_reads_differently_from_not_yet_checked() {
    let never: Observation<u64> = Observation::Unknown(NotSampled::NeverSampled);
    let cannot: Observation<u64> = Observation::Unanswerable("neo-go exposes no plugin list");
    assert_ne!(
        never.render(|v| v.to_string()),
        cannot.render(|v| v.to_string())
    );
    assert_ne!(
        never.cell(|v| v.to_string()),
        cannot.cell(|v| v.to_string())
    );
}

/// A method this client does not implement is not an outage. Conflating them is
/// what once made a healthy Neo X node — which has no `getversion` — report as
/// unreachable.
#[test]
fn an_unimplemented_method_does_not_read_as_a_failure() {
    let unsupported = NotSampled::MethodUnsupported {
        method: "getversion",
    };
    let failed = NotSampled::CallFailed {
        method: "getversion",
        detail: "connection refused".to_string(),
    };
    assert_ne!(unsupported.to_string(), failed.to_string());
    assert!(unsupported.to_string().contains("does not implement"));
    assert!(failed.to_string().contains("connection refused"));
}

/// Deriving from an observation must not quietly turn an absence into a
/// computed number — the reason has to survive the transformation.
#[test]
fn mapping_preserves_both_evidence_and_the_reason_for_absence() {
    let known = Observation::Known(100u64, sample_evidence());
    let mapped = known.map(|height| height * 2);
    assert_eq!(mapped.value(), Some(&200));
    assert_eq!(
        mapped.evidence().map(Evidence::value),
        Some("6245100"),
        "the evidence must still show what was actually read, not the derived value"
    );

    let absent: Observation<u64> = Observation::Unknown(NotSampled::Stale { age_seconds: 90 });
    let mapped = absent.map(|height| height * 2);
    assert_eq!(
        mapped,
        Observation::Unknown(NotSampled::Stale { age_seconds: 90 })
    );
}

/// A known value renders as itself, and keeps the provenance that proves it.
#[test]
fn a_known_observation_renders_its_value_and_keeps_its_provenance() {
    let known = Observation::Known(6_245_100u64, sample_evidence());
    assert_eq!(known.render(|height| format!("#{height}")), "#6245100");
    assert_eq!(known.cell(|height| height.to_string()), "6245100");
    assert!(known.is_known());
    assert_eq!(
        known.evidence().map(Evidence::method),
        Some("getblockcount")
    );
}
