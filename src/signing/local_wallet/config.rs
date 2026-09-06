use std::{env, path::PathBuf, str::FromStr};

use anyhow::{bail, Context, Result};

use crate::{config::network_magic, types::Network};

pub const LOCAL_WALLET_PATH_ENV: &str = "NEONEXUS_SIGNER_LOCAL_WALLET_PATH";
pub const LOCAL_WALLET_PASSWORD_FILE_ENV: &str = "NEONEXUS_SIGNER_LOCAL_WALLET_PASSWORD_FILE";
pub const LOCAL_WALLET_ACCOUNT_ENV: &str = "NEONEXUS_SIGNER_LOCAL_WALLET_ACCOUNT";
pub const LOCAL_WALLET_NETWORK_ENV: &str = "NEONEXUS_SIGNER_LOCAL_WALLET_NETWORK";
pub const LOCAL_WALLET_NETWORK_MAGIC_ENV: &str = "NEONEXUS_SIGNER_LOCAL_WALLET_NETWORK_MAGIC";
pub const LOCAL_WALLET_ALLOW_TRANSACTION_ENV: &str =
    "NEONEXUS_SIGNER_LOCAL_WALLET_ALLOW_TRANSACTION";
pub const LOCAL_WALLET_ALLOW_CONSENSUS_ENV: &str = "NEONEXUS_SIGNER_LOCAL_WALLET_ALLOW_CONSENSUS";
pub const LOCAL_WALLET_ALLOW_RAW_ENV: &str = "NEONEXUS_SIGNER_LOCAL_WALLET_ALLOW_RAW";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalWalletConfig {
    pub wallet_path: PathBuf,
    pub password_file: PathBuf,
    pub account: Option<String>,
    pub network: String,
    pub network_magic: u32,
    pub allow_transaction: bool,
    pub allow_consensus: bool,
    pub allow_raw: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LocalWalletEnvironment {
    pub wallet_path: Option<String>,
    pub password_file: Option<String>,
    pub account: Option<String>,
    pub network: Option<String>,
    pub network_magic: Option<String>,
    pub allow_transaction: Option<String>,
    pub allow_consensus: Option<String>,
    pub allow_raw: Option<String>,
}

impl LocalWalletConfig {
    pub(super) fn validate(&self) -> Result<()> {
        if self.wallet_path.as_os_str().is_empty() || self.password_file.as_os_str().is_empty() {
            bail!("local-wallet paths must not be empty");
        }
        let network = Network::from_str(self.network.trim())
            .with_context(|| format!("unsupported local-wallet network {}", self.network))?;
        if self.network_magic == 0 {
            bail!("{LOCAL_WALLET_NETWORK_MAGIC_ENV} must be greater than zero");
        }
        if matches!(network, Network::Mainnet | Network::Testnet)
            && self.network_magic != network_magic(network)
        {
            bail!(
                "local-wallet {} magic must be {}, not {}",
                network,
                network_magic(network),
                self.network_magic
            );
        }
        if self.account.as_deref().is_some_and(|value| {
            value.trim().is_empty() || value.len() > 128 || value.chars().any(char::is_control)
        }) {
            bail!("{LOCAL_WALLET_ACCOUNT_ENV} is invalid");
        }
        if self.allow_consensus {
            bail!(
                "{LOCAL_WALLET_ALLOW_CONSENSUS_ENV}=true is unsafe without typed consensus \
                 parsing and durable anti-equivocation; configure a signer service"
            );
        }
        Ok(())
    }

    pub fn from_env() -> Result<Option<Self>> {
        Self::resolve_environment(LocalWalletEnvironment {
            wallet_path: read_env(LOCAL_WALLET_PATH_ENV)?,
            password_file: read_env(LOCAL_WALLET_PASSWORD_FILE_ENV)?,
            account: read_env(LOCAL_WALLET_ACCOUNT_ENV)?,
            network: read_env(LOCAL_WALLET_NETWORK_ENV)?,
            network_magic: read_env(LOCAL_WALLET_NETWORK_MAGIC_ENV)?,
            allow_transaction: read_env(LOCAL_WALLET_ALLOW_TRANSACTION_ENV)?,
            allow_consensus: read_env(LOCAL_WALLET_ALLOW_CONSENSUS_ENV)?,
            allow_raw: read_env(LOCAL_WALLET_ALLOW_RAW_ENV)?,
        })
    }

    pub fn resolve_environment(input: LocalWalletEnvironment) -> Result<Option<Self>> {
        let any = [
            input.wallet_path.as_ref(),
            input.password_file.as_ref(),
            input.account.as_ref(),
            input.network.as_ref(),
            input.network_magic.as_ref(),
            input.allow_transaction.as_ref(),
            input.allow_consensus.as_ref(),
            input.allow_raw.as_ref(),
        ]
        .into_iter()
        .any(|value| value.is_some());
        if !any {
            return Ok(None);
        }
        let wallet_path = required(input.wallet_path, LOCAL_WALLET_PATH_ENV)?;
        let password_file = required(input.password_file, LOCAL_WALLET_PASSWORD_FILE_ENV)?;
        let network = required(input.network, LOCAL_WALLET_NETWORK_ENV)?;
        let network_magic = required(input.network_magic, LOCAL_WALLET_NETWORK_MAGIC_ENV)?
            .parse::<u32>()
            .with_context(|| format!("{LOCAL_WALLET_NETWORK_MAGIC_ENV} must be a u32"))?;
        let account = configured(input.account);
        if account.as_deref().is_some_and(|value| value.len() > 128) {
            bail!("{LOCAL_WALLET_ACCOUNT_ENV} is too long");
        }
        let allow_consensus = flag(
            input.allow_consensus,
            LOCAL_WALLET_ALLOW_CONSENSUS_ENV,
            false,
        )?;
        if allow_consensus {
            bail!(
                "{LOCAL_WALLET_ALLOW_CONSENSUS_ENV}=true is unsafe without typed consensus \
                 parsing and durable anti-equivocation; configure a signer service"
            );
        }
        let config = Self {
            wallet_path: PathBuf::from(wallet_path),
            password_file: PathBuf::from(password_file),
            account,
            network,
            network_magic,
            allow_transaction: flag(
                input.allow_transaction,
                LOCAL_WALLET_ALLOW_TRANSACTION_ENV,
                true,
            )?,
            allow_consensus,
            allow_raw: flag(input.allow_raw, LOCAL_WALLET_ALLOW_RAW_ENV, false)?,
        };
        config.validate()?;
        Ok(Some(config))
    }
}

fn flag(raw: Option<String>, name: &str, default: bool) -> Result<bool> {
    match configured(raw).as_deref() {
        None => Ok(default),
        Some("true" | "1" | "yes") => Ok(true),
        Some("false" | "0" | "no") => Ok(false),
        Some(_) => bail!("{name} must be true or false"),
    }
}

fn required(raw: Option<String>, name: &str) -> Result<String> {
    configured(raw).with_context(|| format!("{name} is required for the local-wallet backend"))
}

fn configured(raw: Option<String>) -> Option<String> {
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
