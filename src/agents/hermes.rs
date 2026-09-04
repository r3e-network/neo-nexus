//! NousResearch/hermes-agent adapter. Pin the home before Python imports the CLI.
use super::AgentProfile;
use anyhow::{bail, Context, Result};

pub(super) fn validate(profile: &AgentProfile) -> Result<()> {
    let config = profile
        .config_path
        .as_deref()
        .context("Hermes requires its profile's config.yaml")?;
    if config.file_name().and_then(|name| name.to_str()) != Some("config.yaml") {
        bail!("Hermes configuration must be named config.yaml; its parent becomes HERMES_HOME");
    }
    let text = std::fs::read_to_string(config).context("cannot read Hermes configuration")?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&text)
        .map_err(|_| anyhow::anyhow!("Hermes config.yaml is not valid YAML"))?;
    if !yaml.is_mapping() {
        bail!("Hermes config.yaml must contain a YAML mapping");
    }
    if !profile.working_dir.join("hermes_cli/main.py").is_file() {
        bail!("Hermes working directory must be the installed hermes-agent source root");
    }
    if !profile
        .binary_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem.to_ascii_lowercase().starts_with("python"))
    {
        bail!("Hermes executable must be the installation's Python interpreter (for example .venv/Scripts/python.exe)");
    }
    if !profile.args.is_empty() {
        bail!("Hermes arguments are managed: leave the array empty to run python -m hermes_cli.main gateway run");
    }
    Ok(())
}

pub(super) fn environment(profile: &AgentProfile) -> Vec<(String, String)> {
    let Some(home) = profile
        .config_path
        .as_deref()
        .and_then(|path| path.parent())
    else {
        return vec![];
    };
    vec![
        ("HERMES_HOME".into(), home.display().to_string()),
        // Prevent sticky active_profile from redirecting this supervised gateway
        // into another profile (and potentially another platform credential).
        ("HERMES_GATEWAY_EXTERNAL_SUPERVISOR".into(), "1".into()),
        ("HERMES_SUPERVISED_CHILD".into(), "1".into()),
        ("PYTHONUNBUFFERED".into(), "1".into()),
    ]
}

pub(super) fn arguments() -> Vec<String> {
    [
        "-m",
        "hermes_cli.main",
        "gateway",
        "run",
        "--external-supervisor",
    ]
    .map(str::to_string)
    .to_vec()
}
