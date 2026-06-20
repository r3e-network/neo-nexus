use super::*;

#[test]
fn runtime_signer_profile_requires_valid_ed25519_public_key() {
    let signing_key = SigningKey::from_bytes(&[23u8; 32]);
    let profile = RuntimeSignerProfile {
        id: "official-neo-rs".to_string(),
        label: "Official neo-rs signer".to_string(),
        ed25519_public_key: BASE64_STANDARD.encode(signing_key.verifying_key().to_bytes()),
        enabled: true,
        created_at_unix: 1_800_000_010,
        last_used_at_unix: None,
    };
    let invalid = RuntimeSignerProfile {
        ed25519_public_key: BASE64_STANDARD.encode([0u8; 31]),
        ..profile.clone()
    };

    validate_runtime_signer_profile(&profile).unwrap();
    let error = validate_runtime_signer_profile(&invalid).unwrap_err();

    assert!(error.to_string().contains("must be 32 bytes"));
}

#[test]
fn runtime_upgrade_policy_validates_safe_scheduling_bounds() {
    let policy = RuntimeUpgradePolicy {
        enabled: true,
        catalog_profile_id: Some("official-neo-rs".to_string()),
        interval_minutes: RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES,
        require_signed_catalog: true,
        max_nodes_per_run: 2,
        maintenance_window_enabled: true,
        maintenance_window_start_minute_utc: 60,
        maintenance_window_end_minute_utc: 240,
        wave_delay_minutes: 120,
        last_checked_at_unix: None,
        last_applied_at_unix: None,
    };
    let missing_profile = RuntimeUpgradePolicy {
        catalog_profile_id: None,
        ..policy.clone()
    };
    let too_fast = RuntimeUpgradePolicy {
        interval_minutes: RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES - 1,
        ..policy.clone()
    };
    let invalid_window = RuntimeUpgradePolicy {
        maintenance_window_start_minute_utc: 60,
        maintenance_window_end_minute_utc: 60,
        ..policy.clone()
    };
    let invalid_wave_delay = RuntimeUpgradePolicy {
        wave_delay_minutes: RuntimeUpgradePolicy::MAX_WAVE_DELAY_MINUTES + 1,
        ..policy.clone()
    };

    validate_runtime_upgrade_policy(&policy).unwrap();
    assert!(missing_profile.is_due(90 * 60));
    assert!(validate_runtime_upgrade_policy(&missing_profile)
        .unwrap_err()
        .to_string()
        .contains("requires a catalog profile"));
    assert!(validate_runtime_upgrade_policy(&too_fast)
        .unwrap_err()
        .to_string()
        .contains("between"));
    assert!(validate_runtime_upgrade_policy(&invalid_window)
        .unwrap_err()
        .to_string()
        .contains("non-zero duration"));
    assert!(validate_runtime_upgrade_policy(&invalid_wave_delay)
        .unwrap_err()
        .to_string()
        .contains("wave delay"));

    let checked = policy.with_checked_at(90 * 60);
    assert!(!checked.is_due(100 * 60));
    assert!(checked.is_due((90 + policy.interval_minutes) * 60));
    assert!(!policy.is_due(30 * 60));
    assert!(policy.is_due(90 * 60));
    assert!(!RuntimeUpgradePolicy {
        last_applied_at_unix: Some(90 * 60),
        ..policy.clone()
    }
    .is_due((90 + 30) * 60));
    assert!(RuntimeUpgradePolicy {
        last_applied_at_unix: Some(90 * 60),
        ..policy.clone()
    }
    .is_due((90 + policy.wave_delay_minutes) * 60));

    let overnight = RuntimeUpgradePolicy {
        maintenance_window_start_minute_utc: 23 * 60,
        maintenance_window_end_minute_utc: 2 * 60,
        ..policy
    };
    assert!(overnight.is_in_maintenance_window((23 * 60 + 30) * 60));
    assert!(overnight.is_in_maintenance_window(90 * 60));
    assert!(!overnight.is_in_maintenance_window(3 * 60 * 60));
}
