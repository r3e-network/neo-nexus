//! Fail-closed production environment resolution.

use std::{env, path::PathBuf};

use anyhow::{bail, Context, Result};
use zeroize::Zeroizing;

use super::{
    secret_file::{read_token, read_workload_seed},
    validation::{validated_consumer_origin, validated_service_url, validated_workload_identity},
    AdminCredential, SignerConfig, WorkloadCredential, CALLER_ID_ENV, DEFAULT_TIMEOUT,
    LEGACY_TOKEN_ENV, LEGACY_URL_ENV, ORIGIN_ENV, TIMEOUT_ENV, TOKEN_FILE_ENV, URL_ENV,
    WORKLOAD_KEY_FILE_ENV, WORKLOAD_SUBJECT_ENV,
};

#[derive(Default)]
pub(crate) struct EnvironmentInput {
    pub canonical_url: Option<String>,
    pub legacy_url: Option<String>,
    /// Presence is significant even when blank: secret-bearing env input is
    /// refused rather than normalized away.
    pub legacy_token: Option<String>,
    pub token_file: Option<String>,
    pub caller_id: Option<String>,
    pub workload_key_file: Option<String>,
    pub workload_subject: Option<String>,
    pub consumer_origin: Option<String>,
    pub timeout: Option<String>,
}

impl SignerConfig {
    pub fn from_env() -> Result<Option<Self>> {
        Self::resolve_environment(EnvironmentInput {
            canonical_url: read_env(URL_ENV)?,
            legacy_url: read_env(LEGACY_URL_ENV)?,
            legacy_token: read_env(LEGACY_TOKEN_ENV)?,
            token_file: read_env(TOKEN_FILE_ENV)?,
            caller_id: read_env(CALLER_ID_ENV)?,
            workload_key_file: read_env(WORKLOAD_KEY_FILE_ENV)?,
            workload_subject: read_env(WORKLOAD_SUBJECT_ENV)?,
            consumer_origin: read_env(ORIGIN_ENV)?,
            timeout: read_env(TIMEOUT_ENV)?,
        })
    }

    pub(crate) fn resolve_environment(input: EnvironmentInput) -> Result<Option<Self>> {
        if input.legacy_token.is_some() {
            bail!(
                "{LEGACY_TOKEN_ENV} is refused because process environments expose signer \
                 credentials; write the same bearer token to a protected file and set \
                 {TOKEN_FILE_ENV}"
            );
        }
        if input.canonical_url.is_some() && input.legacy_url.is_some() {
            bail!(
                "set only {URL_ENV}; {LEGACY_URL_ENV} is a deprecated compatibility alias and \
                 both URL variables were present"
            );
        }

        let canonical_url = configured(input.canonical_url);
        let legacy_url = configured(input.legacy_url);
        let raw_url = canonical_url.or(legacy_url);
        let token_file = configured(input.token_file);
        let caller_id = configured(input.caller_id);
        let workload_key_file = configured(input.workload_key_file);
        let workload_subject = configured(input.workload_subject);
        let consumer_origin = configured(input.consumer_origin);
        let timeout = configured(input.timeout);

        let any_setting = raw_url.is_some()
            || token_file.is_some()
            || caller_id.is_some()
            || workload_key_file.is_some()
            || workload_subject.is_some()
            || consumer_origin.is_some()
            || timeout.is_some();
        if !any_setting {
            return Ok(None);
        }
        let raw_url = raw_url.with_context(|| {
            format!(
                "{URL_ENV} must be set when a signer credential, Origin, or timeout is configured"
            )
        })?;
        let (origin, cleartext, loopback) = validated_service_url(&raw_url).with_context(|| {
            format!(
                "{URL_ENV} is not a native signer origin: use http loopback or https with no \
                 credentials, path, query, or fragment"
            )
        })?;
        let timeout = parse_timeout(timeout)?;

        let has_workload =
            caller_id.is_some() || workload_key_file.is_some() || workload_subject.is_some();
        let admin = match (token_file, has_workload) {
            (Some(_), true) => bail!(
                "configure exactly one signer admin profile: {TOKEN_FILE_ENV}, or \
                 {CALLER_ID_ENV} plus {WORKLOAD_KEY_FILE_ENV}"
            ),
            (Some(path), false) => {
                AdminCredential::Bearer(Zeroizing::new(read_token(&PathBuf::from(path))?))
            }
            (None, true) => {
                let caller_id = caller_id.with_context(|| {
                    format!("{CALLER_ID_ENV} is required with {WORKLOAD_KEY_FILE_ENV}")
                })?;
                let key_file = workload_key_file.with_context(|| {
                    format!("{WORKLOAD_KEY_FILE_ENV} is required with {CALLER_ID_ENV}")
                })?;
                if consumer_origin.is_some() {
                    bail!(
                        "{ORIGIN_ENV} applies only to bearer callers; the configured admin \
                         workload identity is server-to-server and must send no Origin"
                    );
                }
                let (caller_id, subject) =
                    validated_workload_identity(caller_id, workload_subject)?;
                let seed = read_workload_seed(&PathBuf::from(key_file))?;
                AdminCredential::Workload(Box::new(WorkloadCredential::new(
                    caller_id, subject, *seed,
                )))
            }
            (None, false) => bail!(
                "{URL_ENV} requires exactly one signer admin profile: {TOKEN_FILE_ENV}, or \
                 {CALLER_ID_ENV} plus {WORKLOAD_KEY_FILE_ENV}"
            ),
        };
        let consumer_origin = consumer_origin
            .map(|value| {
                validated_consumer_origin(&value)
                    .with_context(|| format!("{ORIGIN_ENV} is not a usable browser Origin"))
            })
            .transpose()?;
        Ok(Some(SignerConfig::production(
            origin,
            admin,
            consumer_origin,
            timeout,
            cleartext,
            loopback,
        )))
    }
}

pub(super) fn configured(raw: Option<String>) -> Option<String> {
    raw.map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_env(name: &str) -> Result<Option<String>> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => bail!("{name} must contain valid Unicode"),
    }
}

pub(super) fn parse_timeout(raw_timeout: Option<String>) -> Result<std::time::Duration> {
    match raw_timeout {
        Some(raw) => raw
            .parse::<u64>()
            .ok()
            .filter(|seconds| *seconds > 0)
            .map(std::time::Duration::from_secs)
            .with_context(|| {
                format!("{TIMEOUT_ENV} must be a whole number of seconds greater than zero")
            }),
        None => Ok(DEFAULT_TIMEOUT),
    }
}
