use std::{io::Write, time::Duration};

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

use super::{
    environment::EnvironmentInput, SignerConfig, CALLER_ID_ENV, DEFAULT_TIMEOUT, LEGACY_TOKEN_ENV,
    LEGACY_URL_ENV, ORIGIN_ENV, TIMEOUT_ENV, TOKEN_ENV, TOKEN_FILE_ENV, URL_ENV,
    WORKLOAD_KEY_FILE_ENV,
};

const URL: &str = "http://127.0.0.1:8081";
const ADMIN: &str = "admin-token-cantus-9f31";
const FILE_ADMIN: &str = "nsk1_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn no_url_means_custody_is_not_configured_here() {
    assert!(SignerConfig::resolve(None, None, None, None)
        .unwrap()
        .is_none());
    // All blank is the same deliberate state as all absent.
    assert!(SignerConfig::resolve(
        Some("   ".to_string()),
        Some(" ".to_string()),
        Some(String::new()),
        Some(" ".to_string()),
    )
    .unwrap()
    .is_none());
}

#[test]
fn partial_configuration_fails_closed_before_startup() {
    for (url, token, origin, timeout, missing) in [
        (Some(URL), None, None, None, TOKEN_ENV),
        (None, Some(ADMIN), None, None, URL_ENV),
        (None, None, Some("https://nexus.example"), None, URL_ENV),
        (None, None, None, Some("30"), URL_ENV),
    ] {
        let error = SignerConfig::resolve(
            url.map(str::to_string),
            token.map(str::to_string),
            origin.map(str::to_string),
            timeout.map(str::to_string),
        )
        .expect_err("a partial custody identity must not start");
        assert!(error.to_string().contains(missing), "{error}");
    }
}

#[test]
fn a_usable_url_is_the_one_state_that_carries_a_config() {
    let config = SignerConfig::resolve(
        Some(format!("  {URL}  ")),
        Some(ADMIN.to_string()),
        None,
        Some("45".to_string()),
    )
    .unwrap()
    .expect("configured");
    assert_eq!(config.base_url(), URL);
    assert_eq!(config.timeout(), Duration::from_secs(45));
    let admin = config.admin().expect("a configured credential");
    assert_eq!(admin.token(), ADMIN);
}

#[test]
fn a_set_but_unusable_url_is_an_error_rather_than_a_default() {
    // Each of these is a typo that would otherwise read as a decision to run
    // without custody — and one of them is a decision to send a private key
    // somewhere the operator did not mean.
    for candidate in [
        "signer.internal:8081",
        "ftp://signer.internal",
        "http://operator:hunter2@127.0.0.1:8081",
        "http://127.0.0.1:8081#keys",
        "http://127.0.0.1:8081?tenant=1",
        // The prefix is the client's to add. A URL that already carries it
        // produces /signer/api/v1/signer/api/v1/… and a 404 nobody can explain.
        "http://127.0.0.1:8081/signer/api/v1",
    ] {
        let error = SignerConfig::resolve(
            Some(candidate.to_string()),
            Some(ADMIN.to_string()),
            None,
            None,
        )
        .expect_err("{candidate} must not be accepted");
        let text = error.to_string();
        // The message has to name the variable, because "invalid URL" on its own
        // does not tell an operator which of four they mis-typed.
        assert!(text.contains(URL_ENV), "{text}");
        assert!(text.contains("signer service"), "{text}");
    }
}

#[test]
fn the_origin_and_the_mount_path_are_kept_apart() {
    // A proxy that mounts the service under a path is a deployment that exists.
    // The operator names the mount; the client owns §5's prefix.
    let config = SignerConfig::new(
        "https://signer.internal:9443/custody/",
        None,
        DEFAULT_TIMEOUT,
    )
    .unwrap();
    assert_eq!(config.base_url(), "https://signer.internal:9443/custody");
    assert_eq!(
        config.url("/keys", Some("limit=5")),
        "https://signer.internal:9443/custody/signer/api/v1/keys?limit=5"
    );
}

#[test]
fn a_bare_origin_needs_no_trailing_slash_to_join() {
    for candidate in ["http://127.0.0.1:8081", "http://127.0.0.1:8081/"] {
        let config = SignerConfig::new(candidate, None, DEFAULT_TIMEOUT).unwrap();
        assert_eq!(
            config.url("/keys/key-1", None),
            "http://127.0.0.1:8081/signer/api/v1/keys/key-1"
        );
    }
}

#[test]
fn an_ipv6_origin_keeps_the_brackets_the_parser_added() {
    let config = SignerConfig::new("http://[::1]:8081", None, DEFAULT_TIMEOUT).unwrap();
    assert_eq!(config.base_url(), "http://[::1]:8081");
}

#[test]
fn a_default_port_is_not_invented_back_in() {
    let config = SignerConfig::new("https://signer.internal:443", None, DEFAULT_TIMEOUT).unwrap();
    assert_eq!(config.base_url(), "https://signer.internal");
}

#[test]
fn cleartext_is_accepted_only_for_loopback() -> Result<()> {
    for candidate in [
        URL,
        "http://127.0.0.2:8081",
        "http://[::1]:8081",
        "http://[::ffff:127.0.0.1]:8081",
        "http://localhost:8081",
        "http://worker.localhost:8081",
        "http://localhost.:8081",
    ] {
        let config = SignerConfig::new(candidate, None, DEFAULT_TIMEOUT)
            .with_context(|| format!("loopback {candidate} was refused"))?;
        assert!(config.uses_cleartext(), "{candidate}");
        assert!(config.is_loopback(), "{candidate}");
    }

    for candidate in [
        "http://signer.internal:8081",
        "http://192.168.1.8:8081",
        "http://10.0.0.5:8081",
        "http://0.0.0.0:8081",
        "http://[::]:8081",
    ] {
        let error = SignerConfig::new(candidate, None, DEFAULT_TIMEOUT)
            .expect_err("non-loopback HTTP must not carry signer credentials");
        assert!(error.to_string().contains("non-loopback"), "{error:#}");
    }

    for candidate in [
        "https://signer.internal",
        "https://192.168.1.8:8443",
        "https://[2001:db8::5]:8443",
    ] {
        let config = SignerConfig::new(candidate, None, DEFAULT_TIMEOUT)
            .with_context(|| format!("HTTPS {candidate} was refused"))?;
        assert!(!config.uses_cleartext(), "{candidate}");
        assert!(!config.is_loopback(), "{candidate}");
    }
    Ok(())
}

#[test]
fn a_blank_token_is_no_credential() {
    let config = SignerConfig::new(URL, Some("   ".to_string()), DEFAULT_TIMEOUT).unwrap();
    assert!(config.admin().is_none());
}

#[test]
fn a_relay_needs_no_credential_of_its_own() {
    // The two halves of §5.1's caller story are configured apart on purpose: the
    // console needs the admin token, the relay brings its caller's.
    let config = SignerConfig::new(URL, None, DEFAULT_TIMEOUT).unwrap();
    assert!(config.admin().is_none());
    assert!(!config.base_url().is_empty());
}

#[test]
fn a_timeout_that_is_not_a_positive_number_is_refused() {
    for candidate in ["0", "-3", "ten", "1.5"] {
        let error = SignerConfig::resolve(
            Some(URL.to_string()),
            Some(ADMIN.to_string()),
            None,
            Some(candidate.to_string()),
        )
        .expect_err("{candidate} must not become a timeout");
        assert!(error.to_string().contains("seconds"), "{error}");
        assert!(error.to_string().contains(TIMEOUT_ENV), "{error}");
    }
}

#[test]
fn an_absent_timeout_is_the_default_and_a_blank_one_is_absent() {
    for candidate in [None, Some(String::new()), Some("  ".to_string())] {
        let config = SignerConfig::resolve(
            Some(URL.to_string()),
            Some(ADMIN.to_string()),
            None,
            candidate,
        )
        .unwrap()
        .expect("configured");
        assert_eq!(config.timeout(), DEFAULT_TIMEOUT);
    }
}

#[test]
fn the_debug_of_a_config_names_the_credential_without_showing_it() {
    let text = format!(
        "{:?}",
        SignerConfig::new(URL, Some(ADMIN.to_string()), DEFAULT_TIMEOUT).unwrap()
    );
    assert!(!text.contains(ADMIN), "{text}");
    assert!(text.contains("<redacted>"), "{text}");
    let unset = format!(
        "{:?}",
        SignerConfig::new(URL, None, DEFAULT_TIMEOUT).unwrap()
    );
    assert!(unset.contains("<none>"), "{unset}");
}

#[test]
fn the_admin_credential_is_a_bearer_with_no_origin() {
    // A caller record that declares no origins is refused `origin-unexpected` by a
    // request that brings one, so the credential this process uses for itself must
    // not grow headers.
    let config = SignerConfig::new(URL, Some(ADMIN.to_string()), DEFAULT_TIMEOUT).unwrap();
    let admin = config.admin().expect("configured");
    assert_eq!(admin.token(), ADMIN);
    assert!(admin.origin().is_none() && admin.referer().is_none());
}

#[test]
fn a_configured_consumer_origin_is_normalized_and_sent_by_admin() {
    let config = SignerConfig::resolve(
        Some(URL.to_string()),
        Some(ADMIN.to_string()),
        Some(" https://nexus.example:443/ ".to_string()),
        None,
    )
    .unwrap()
    .expect("configured");
    assert_eq!(config.consumer_origin(), Some("https://nexus.example"));
    let admin = config.admin().expect("configured");
    assert_eq!(admin.origin(), Some("https://nexus.example"));
    assert!(admin.referer().is_none());
}

#[test]
fn a_consumer_origin_is_an_origin_not_a_url() {
    for candidate in [
        "nexus.example",
        "ftp://nexus.example",
        "https://operator:secret@nexus.example",
        "https://nexus.example/control",
        "https://nexus.example?tenant=1",
        "https://nexus.example#fragment",
    ] {
        let error = SignerConfig::resolve(
            Some(URL.to_string()),
            Some(ADMIN.to_string()),
            Some(candidate.to_string()),
            None,
        )
        .expect_err("not an Origin");
        let text = format!("{error:#}");
        assert!(text.contains(ORIGIN_ENV), "{text}");
    }
}

// -- production environment profiles -------------------------------------

#[test]
fn production_prefers_one_canonical_url_name_and_fails_on_ambiguity() {
    let error = SignerConfig::resolve_environment(EnvironmentInput {
        canonical_url: Some(URL.to_string()),
        legacy_url: Some(URL.to_string()),
        ..EnvironmentInput::default()
    })
    .expect_err("the canonical URL and deprecated alias must not compete");
    let text = error.to_string();
    assert!(text.contains(URL_ENV), "{text}");
    assert!(text.contains(LEGACY_URL_ENV), "{text}");
}

#[test]
fn production_refuses_the_legacy_plaintext_token_even_when_blank() {
    for exposed in [String::new(), FILE_ADMIN.to_string()] {
        let error = SignerConfig::resolve_environment(EnvironmentInput {
            canonical_url: Some(URL.to_string()),
            legacy_token: Some(exposed),
            ..EnvironmentInput::default()
        })
        .expect_err("secret-bearing environment input must always be refused");
        let text = error.to_string();
        assert!(text.contains(LEGACY_TOKEN_ENV), "{text}");
        assert!(text.contains(TOKEN_FILE_ENV), "{text}");
    }
}

#[test]
fn production_url_is_the_native_origin_not_a_proxy_mount() {
    for candidate in [
        "https://signer.example/custody",
        "https://signer.example//",
        "https://signer.example/.",
        "https://signer.example?tenant=one",
        "https://signer.example#vault",
        "https://operator:secret@signer.example",
        "https://@signer.example",
        "http://signer.example",
    ] {
        let error = SignerConfig::resolve_environment(EnvironmentInput {
            canonical_url: Some(candidate.to_string()),
            token_file: Some("unused-token-file".to_string()),
            ..EnvironmentInput::default()
        })
        .expect_err("a non-native signer origin must fail before credential loading");
        let text = format!("{error:#}");
        assert!(text.contains(URL_ENV), "{candidate}: {text}");
    }
}

#[test]
fn production_reads_one_protected_bearer_file() -> Result<()> {
    let mut file = protected_file()?;
    writeln!(file, "{FILE_ADMIN}")?;
    let config = SignerConfig::resolve_environment(EnvironmentInput {
        canonical_url: Some(URL.to_string()),
        token_file: Some(file.path().display().to_string()),
        ..EnvironmentInput::default()
    })?
    .context("the complete bearer profile was treated as unconfigured")?;
    let credential = config.admin().context("the bearer profile has no admin")?;
    assert_eq!(credential.token(), FILE_ADMIN);
    assert_eq!(config.base_url(), URL);
    Ok(())
}

#[test]
fn deprecated_url_alias_works_only_when_canonical_name_is_absent() -> Result<()> {
    let mut file = protected_file()?;
    writeln!(file, "{FILE_ADMIN}")?;
    let config = SignerConfig::resolve_environment(EnvironmentInput {
        legacy_url: Some(URL.to_string()),
        token_file: Some(file.path().display().to_string()),
        ..EnvironmentInput::default()
    })?
    .context("the deprecated URL alias was treated as unconfigured")?;
    assert_eq!(config.base_url(), URL);
    Ok(())
}

#[test]
fn bearer_file_is_bounded_and_exactly_one_unpadded_line() -> Result<()> {
    for invalid in [
        "too-short".to_string(),
        format!("{FILE_ADMIN}\nsecond-token"),
        "x".repeat(4 * 1024 + 1),
    ] {
        let mut token = protected_file()?;
        write!(token, "{invalid}")?;
        let error = SignerConfig::resolve_environment(EnvironmentInput {
            canonical_url: Some(URL.to_string()),
            token_file: Some(token.path().display().to_string()),
            ..EnvironmentInput::default()
        })
        .expect_err("an unsafe token file must be refused");
        assert!(format!("{error:#}").contains("admin token"), "{error:#}");
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn credential_files_reject_group_or_other_permissions() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut token = protected_file()?;
    writeln!(token, "{FILE_ADMIN}")?;
    std::fs::set_permissions(token.path(), std::fs::Permissions::from_mode(0o640))?;
    let error = SignerConfig::resolve_environment(EnvironmentInput {
        canonical_url: Some(URL.to_string()),
        token_file: Some(token.path().display().to_string()),
        ..EnvironmentInput::default()
    })
    .expect_err("group-readable signer credentials must be refused");
    assert!(format!("{error:#}").contains("group or other"), "{error:#}");
    Ok(())
}

#[test]
fn production_requires_exactly_one_complete_admin_profile() -> Result<()> {
    let mut token = protected_file()?;
    writeln!(token, "{FILE_ADMIN}")?;
    let mut seed = protected_file()?;
    writeln!(seed, "{}", "11".repeat(32))?;

    for input in [
        EnvironmentInput {
            canonical_url: Some(URL.to_string()),
            ..EnvironmentInput::default()
        },
        EnvironmentInput {
            canonical_url: Some(URL.to_string()),
            caller_id: Some("nexus-admin".to_string()),
            ..EnvironmentInput::default()
        },
        EnvironmentInput {
            canonical_url: Some(URL.to_string()),
            workload_key_file: Some(seed.path().display().to_string()),
            ..EnvironmentInput::default()
        },
        EnvironmentInput {
            canonical_url: Some(URL.to_string()),
            token_file: Some(token.path().display().to_string()),
            caller_id: Some("nexus-admin".to_string()),
            workload_key_file: Some(seed.path().display().to_string()),
            ..EnvironmentInput::default()
        },
    ] {
        let error = SignerConfig::resolve_environment(input)
            .expect_err("a missing or ambiguous admin profile must fail closed");
        let text = error.to_string();
        assert!(
            text.contains(TOKEN_FILE_ENV)
                || text.contains(CALLER_ID_ENV)
                || text.contains(WORKLOAD_KEY_FILE_ENV),
            "{text}"
        );
    }
    Ok(())
}

#[test]
fn workload_seed_is_exactly_one_lowercase_hex_line() -> Result<()> {
    for invalid in [
        "AA".repeat(32),
        format!("{}\n{}", "11".repeat(32), "22".repeat(32)),
        "1".repeat(1025),
    ] {
        let mut seed = protected_file()?;
        write!(seed, "{invalid}")?;
        let error = SignerConfig::resolve_environment(EnvironmentInput {
            canonical_url: Some(URL.to_string()),
            caller_id: Some("nexus-admin".to_string()),
            workload_key_file: Some(seed.path().display().to_string()),
            ..EnvironmentInput::default()
        })
        .expect_err("a malformed Ed25519 seed must be refused");
        let text = format!("{error:#}");
        assert!(text.contains("workload key"), "{text}");
    }
    Ok(())
}

#[test]
fn production_accepts_a_complete_workload_identity() -> Result<()> {
    let mut seed = protected_file()?;
    writeln!(seed, "{}", "42".repeat(32))?;
    let config = SignerConfig::resolve_environment(EnvironmentInput {
        canonical_url: Some(URL.to_string()),
        caller_id: Some("nexus-admin-1".to_string()),
        workload_key_file: Some(seed.path().display().to_string()),
        workload_subject: Some("neo-nexus-production".to_string()),
        ..EnvironmentInput::default()
    })?
    .context("the complete workload profile was treated as unconfigured")?;
    let credential = config
        .admin()
        .context("the workload profile has no admin identity")?;
    assert_eq!(credential.token(), "");
    assert!(credential.origin().is_none());
    assert!(format!("{config:?}").contains("workload-ed25519:<redacted>"));
    Ok(())
}

#[test]
fn workload_admin_is_server_to_server_and_rejects_consumer_origin() -> Result<()> {
    let mut seed = protected_file()?;
    writeln!(seed, "{}", "42".repeat(32))?;
    let error = SignerConfig::resolve_environment(EnvironmentInput {
        canonical_url: Some(URL.to_string()),
        caller_id: Some("nexus-admin-1".to_string()),
        workload_key_file: Some(seed.path().display().to_string()),
        consumer_origin: Some("https://nexus.example".to_string()),
        ..EnvironmentInput::default()
    })
    .expect_err("a workload identity must not emit browser Origin");
    assert!(format!("{error:#}").contains(ORIGIN_ENV), "{error:#}");
    Ok(())
}

fn protected_file() -> Result<NamedTempFile> {
    let file = NamedTempFile::new().context("create a temporary credential file")?;
    crate::secret_file::protect_for_test(file.path())?;
    Ok(file)
}
