//! Add one scoped NeoNexus MCP connection to an existing Hermes profile.
//! Channel and provider credentials remain owned by Hermes.
use anyhow::{bail, Context, Result};
use serde_yaml::{Mapping, Value};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct InjectionReport {
    pub config_backup: PathBuf,
    pub environment_backup: Option<PathBuf>,
    config_path: PathBuf,
    config_digest: String,
    environment_digest: String,
}

/// Validate before issuing a new grant. Never includes config values in errors.
pub fn validate(config_path: &Path, endpoint: &str) -> Result<()> {
    let url = url::Url::parse(endpoint).map_err(|_| anyhow::anyhow!("invalid NeoNexus MCP URL"))?;
    let loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host == "[::1]"
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if !(url.scheme() == "https" && url.host_str().is_some() || url.scheme() == "http" && loopback)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/mcp"
    {
        bail!("MCP URL must use HTTPS or loopback HTTP, end in /mcp, and contain no credentials, query or fragment");
    }
    if !config_path.is_absolute()
        || config_path.file_name().and_then(|p| p.to_str()) != Some("config.yaml")
    {
        bail!("Hermes profile must reference an absolute config.yaml path");
    }
    reject_links(config_path)?;
    let config = parse_config(&read_limited(config_path, 4 * 1024 * 1024)?)?;
    if config
        .get("mcp_servers")
        .is_some_and(|servers| !servers.is_mapping() && !servers.is_null())
    {
        bail!("Hermes mcp_servers must be a mapping");
    }
    let environment = config_path.with_file_name(".env");
    reject_links(&environment)?;
    if environment.exists() {
        read_limited(&environment, 1024 * 1024)?;
    }
    Ok(())
}

pub fn validate_entry(config_path: &Path, endpoint: &str, entry_id: &str) -> Result<()> {
    validate(config_path, endpoint)?;
    if !entry_id.starts_with("neonexus_")
        || !entry_id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
    {
        bail!("invalid managed MCP entry name");
    }
    let config = parse_config(&read_limited(config_path, 4 * 1024 * 1024)?)?;
    if let Some(server) = config
        .get("mcp_servers")
        .and_then(|servers| servers.get(entry_id))
    {
        if !server.is_mapping() {
            bail!(
                "managed MCP entry conflicts with a non-mapping value; review Hermes config.yaml"
            );
        }
        if server.get("command").is_some()
            || server
                .get("transport")
                .is_some_and(|value| value.as_str() != Some("http"))
        {
            bail!("managed MCP entry uses a different transport; review Hermes config.yaml before reconnecting");
        }
        if server
            .get("headers")
            .is_some_and(|headers| !headers.is_mapping())
        {
            bail!("managed MCP headers must be a mapping");
        }
    }
    Ok(())
}

/// Caller holds the lifecycle lock and guarantees the Hermes process is stopped.
/// Both originals are backed up before a replacement. If config replacement
/// fails after .env changes, restore .env before returning the failure.
pub fn inject(
    config_path: &Path,
    endpoint: &str,
    entry_id: &str,
    env_name: &str,
    secret: &str,
) -> Result<InjectionReport> {
    validate_entry(config_path, endpoint, entry_id)?;
    if !env_name.starts_with("NEONEXUS_ASSISTANT_")
        || !env_name.ends_with("_TOKEN")
        || !env_name
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
    {
        bail!("invalid assistant token environment name");
    }
    if secret.is_empty()
        || secret.len() > 4096
        || !secret
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
    {
        bail!("invalid assistant credential encoding");
    }
    let old_config = read_limited(config_path, 4 * 1024 * 1024)?;
    let mut config = parse_config(&old_config)?;
    let servers = config
        .as_mapping_mut()
        .context("Hermes config.yaml must contain a mapping")?
        .entry(Value::String("mcp_servers".into()))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    if servers.is_null() {
        *servers = Value::Mapping(Mapping::new());
    }
    let server = servers
        .as_mapping_mut()
        .context("Hermes mcp_servers must be a mapping")?
        .entry(Value::String(entry_id.into()))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    let server = server.as_mapping_mut().context(
        "managed MCP entry conflicts with a non-mapping value; review Hermes config.yaml",
    )?;
    if server.contains_key(Value::String("command".into()))
        || server
            .get(Value::String("transport".into()))
            .is_some_and(|value| value.as_str() != Some("http"))
    {
        bail!("managed MCP entry uses a different transport; review Hermes config.yaml before reconnecting");
    }
    server.insert(
        Value::String("transport".into()),
        Value::String("http".into()),
    );
    server.insert(Value::String("url".into()), Value::String(endpoint.into()));
    let headers = server
        .entry(Value::String("headers".into()))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    let headers = headers
        .as_mapping_mut()
        .context("managed MCP headers must be a mapping")?;
    // HTTP header names are case insensitive; avoid leaving an older lowercase
    // authorization header beside the new credential reference.
    headers.retain(|key, _| {
        !key.as_str()
            .is_some_and(|key| key.eq_ignore_ascii_case("authorization"))
    });
    headers.insert(
        Value::String("Authorization".into()),
        Value::String(format!("Bearer ${{{env_name}}}")),
    );
    server
        .entry(Value::String("connect_timeout".into()))
        .or_insert(Value::Number(10.into()));
    server
        .entry(Value::String("tool_timeout".into()))
        .or_insert(Value::Number(30.into()));
    let new_config =
        serde_yaml::to_string(&config).context("cannot serialize Hermes configuration")?;
    let environment = config_path.with_file_name(".env");
    let old_environment = if environment.exists() {
        Some(read_limited(&environment, 1024 * 1024)?)
    } else {
        None
    };
    let new_environment = update_env(old_environment.as_deref().unwrap_or(""), env_name, secret);
    let config_backup = backup_path(config_path);
    private_write(&config_backup, old_config.as_bytes())?;
    let environment_backup = if let Some(old) = &old_environment {
        let path = backup_path(&environment);
        private_write(&path, old.as_bytes())?;
        Some(path)
    } else {
        None
    };
    // Detect edits made while preflighting/backing up before replacing either file.
    if read_limited(config_path, 4 * 1024 * 1024)? != old_config
        || read_optional(&environment)? != old_environment
    {
        bail!("Hermes configuration changed during connection setup; retry after reviewing it");
    }
    private_write(&environment, new_environment.as_bytes())?;
    if let Err(error) = private_write(config_path, new_config.as_bytes()) {
        let rollback = match &old_environment {
            Some(old) => private_write(&environment, old.as_bytes()),
            None => fs::remove_file(&environment).map_err(Into::into),
        };
        if rollback.is_err() {
            bail!("Hermes config update and environment rollback failed; restore the retained configuration backups before reconnecting");
        }
        return Err(error).context("Hermes config update failed; environment restored");
    }
    Ok(InjectionReport {
        config_backup,
        environment_backup,
        config_path: config_path.into(),
        config_digest: digest(&new_config),
        environment_digest: digest(&new_environment),
    })
}

pub fn configured_endpoint(config_path: &Path, entry_id: &str) -> Result<Option<String>> {
    let config = parse_config(&read_limited(config_path, 4 * 1024 * 1024)?)?;
    let endpoint = config
        .get("mcp_servers")
        .and_then(|servers| servers.get(entry_id))
        .and_then(|server| server.get("url"))
        .and_then(Value::as_str);
    if let Some(endpoint) = endpoint {
        validate(config_path, endpoint)?;
    }
    Ok(endpoint.map(str::to_string))
}

/// Restore originals if the operation's later database update fails. Refuse to
/// overwrite unrelated edits made since injection; the backups remain available.
pub fn rollback(report: &InjectionReport) -> Result<()> {
    let environment = report.config_path.with_file_name(".env");
    if digest(&read_limited(&report.config_path, 4 * 1024 * 1024)?) != report.config_digest
        || digest(&read_limited(&environment, 1024 * 1024)?) != report.environment_digest
    {
        bail!(
            "Hermes files changed after connection setup; original backups retained for recovery"
        );
    }
    private_write(
        &report.config_path,
        read_limited(&report.config_backup, 4 * 1024 * 1024)?.as_bytes(),
    )?;
    match &report.environment_backup {
        Some(backup) => private_write(&environment, read_limited(backup, 1024 * 1024)?.as_bytes())?,
        None => fs::remove_file(environment)?,
    }
    Ok(())
}

fn digest(value: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn parse_config(text: &str) -> Result<Value> {
    let value: Value = serde_yaml::from_str(text)
        .map_err(|_| anyhow::anyhow!("Hermes config.yaml is not valid YAML"))?;
    if !value.is_mapping() {
        bail!("Hermes config.yaml must contain a mapping");
    }
    Ok(value)
}

fn read_limited(path: &Path, limit: u64) -> Result<String> {
    let file = fs::File::open(path).context("cannot open Hermes profile file")?;
    let mut text = String::new();
    file.take(limit + 1)
        .read_to_string(&mut text)
        .context("cannot read Hermes profile as UTF-8")?;
    if text.len() as u64 > limit {
        bail!("Hermes profile file exceeds the supported size");
    }
    Ok(text)
}

fn read_optional(path: &Path) -> Result<Option<String>> {
    if path.exists() {
        Ok(Some(read_limited(path, 1024 * 1024)?))
    } else {
        Ok(None)
    }
}

fn update_env(original: &str, name: &str, value: &str) -> String {
    let newline = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut updated = String::new();
    for line in original.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let key = trimmed
            .strip_prefix("export ")
            .unwrap_or(trimmed)
            .split_once('=')
            .map(|(key, _)| key.trim());
        if key != Some(name) {
            updated.push_str(line);
        }
    }
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push_str(newline);
    }
    updated.push_str(&format!("{name}={value}{newline}"));
    updated
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_file_name(format!(
        "{}.neonexus-backup-{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        uuid::Uuid::new_v4()
    ))
}

fn reject_links(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                bail!("Hermes profile paths must not contain symbolic links")
            }
            Ok(_) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error).context("cannot inspect Hermes profile path"),
        }
    }
    Ok(())
}

fn private_write(path: &Path, bytes: &[u8]) -> Result<()> {
    reject_links(path)?;
    let temporary = path.with_file_name(format!(".neonexus-writing-{}", uuid::Uuid::new_v4()));
    let result = (|| -> Result<()> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temporary)
            .context("cannot stage Hermes profile update")?;
        #[cfg(windows)]
        restrict_windows_file(&temporary)?;
        file.write_all(bytes)
            .context("cannot write Hermes profile update")?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path).context("cannot publish Hermes profile update")?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(windows)]
fn restrict_windows_file(path: &Path) -> Result<()> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt};
    #[link(name = "advapi32")]
    extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            descriptor: *mut *mut c_void,
            size: *mut u32,
        ) -> i32;
        fn SetFileSecurityW(path: *const u16, information: u32, descriptor: *const c_void) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
    }
    // Protected DACL: owner, administrators and SYSTEM only. Applied to the
    // empty staging file before any configuration/credential bytes are written.
    let sddl: Vec<u16> = "D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FA;;;OW)"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut descriptor = std::ptr::null_mut();
    unsafe {
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            std::ptr::null_mut(),
        ) == 0
        {
            return Err(std::io::Error::last_os_error())
                .context("cannot create private Hermes file permissions");
        }
        let applied = SetFileSecurityW(name.as_ptr(), 0x80000004, descriptor);
        let error = std::io::Error::last_os_error();
        LocalFree(descriptor);
        if applied == 0 {
            return Err(error).context("cannot restrict Hermes file permissions");
        }
    }
    Ok(())
}
