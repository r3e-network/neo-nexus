mod sidecar;

use anyhow::Result;
use serde_json::{json, Value};

use crate::{catalog::PluginState, types::NodeConfig};

use super::super::format::{
    effective_network_magic, max_transactions_per_block, neo_cli_storage_engine, GenerationContext,
    RuntimeConfigProfile,
};
use super::ConfigGenerator;

pub use sidecar::PluginSidecar;

/// Where neo-cli fetches plugin packages from. The stock value moved with the
/// plugins themselves, from the archived neo-modules repository to neo-node.
const PLUGIN_DOWNLOAD_URL: &str = "https://api.github.com/repos/neo-project/neo-node/releases";

/// neo-cli substitutes `{0}` with the network magic, so one workspace can hold
/// several networks without their chain data colliding.
const STORAGE_PATH_TEMPLATE: &str = "Data_LevelDB_{0}";

impl ConfigGenerator {
    pub(super) fn neo_cli_sidecars(
        node: &NodeConfig,
        enabled: &[crate::catalog::PluginId],
    ) -> Vec<PluginSidecar> {
        sidecar::sidecars_for(node, enabled)
    }

    pub fn neo_cli(node: &NodeConfig, plugins: &[PluginState]) -> Result<Value> {
        Self::neo_cli_with_profile(node, plugins, None)
    }

    /// The primary `config.json`.
    ///
    /// Two keys this used to emit are not read by neo-cli at all:
    /// - A top-level `Plugins` **array** of `{"Name": …}`. neo-cli's
    ///   `ApplicationConfiguration.Plugins` is an object with a single
    ///   `DownloadUrl`; a plugin is enabled by the presence of its assembly
    ///   under `Plugins/<Name>/`, and no config key expresses that.
    /// - `ApplicationConfiguration.RPC.Port`. There is no `RPC` section here.
    ///   The listening port lives in `Plugins/RpcServer/RpcServer.json`, so an
    ///   operator who changed the port got a node still on the old one.
    ///
    /// Shape verified against `src/Neo.CLI/config.json` in neo-project/neo-node.
    pub fn neo_cli_with_profile(
        node: &NodeConfig,
        plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
    ) -> Result<Value> {
        Self::neo_cli_with_context(node, plugins, profile, &GenerationContext::default())
    }

    /// The primary config for a node performing a duty.
    ///
    /// neo-cli opens one wallet for the whole node — `UnlockWallet` — and the
    /// plugins that sign (dBFT, OracleService, StateService) use it. So unlike
    /// neo-go, the wallet lives here rather than in each service section.
    pub fn neo_cli_with_context(
        node: &NodeConfig,
        _plugins: &[PluginState],
        profile: Option<&RuntimeConfigProfile>,
        context: &GenerationContext,
    ) -> Result<Value> {
        let mut protocol = json!({
            "Network": effective_network_magic(node.network, profile)
        });
        if let Some(profile) = profile {
            protocol["SeedList"] = json!(profile.seed_nodes);
            protocol["ValidatorsCount"] = json!(profile.validators_count);
            protocol["StandbyCommittee"] = json!(profile.committee_public_keys);
            protocol["MillisecondsPerBlock"] = json!(15_000);
            protocol["MaxTransactionsPerBlock"] = json!(max_transactions_per_block(node.network));
        }

        Ok(json!({
            "ProtocolConfiguration": protocol,
            "ApplicationConfiguration": {
                // `Active` defaults to false, and neo-cli only installs a log
                // sink when it is true *and* `Path` is set. Left at the default
                // the node writes nothing at all, and the Logs surface reads a
                // file that is never created.
                "Logger": {
                    "Path": "Logs",
                    "ConsoleOutput": true,
                    "Active": true
                },
                "Storage": {
                    "Engine": neo_cli_storage_engine(node.storage_engine),
                    "Path": STORAGE_PATH_TEMPLATE
                },
                "P2P": {
                    "Port": node.p2p_port
                },
                "UnlockWallet": unlock_wallet(context),
                "Plugins": {
                    "DownloadUrl": PLUGIN_DOWNLOAD_URL
                }
            }
        }))
    }
}

/// The wallet neo-cli opens at startup, which every signing plugin then uses.
///
/// `IsActive` is the switch: without a password the node would prompt on a
/// console the supervisor does not have, so the path is written for the
/// operator to see and the wallet stays closed until they supply one.
fn unlock_wallet(context: &GenerationContext) -> Value {
    let wallet = context.wallet.as_ref();
    json!({
        "Path": wallet.map(|wallet| wallet.path.as_str()).unwrap_or_default(),
        "Password": wallet
            .and_then(|wallet| wallet.password.as_deref())
            .unwrap_or_default(),
        "IsActive": wallet.is_some_and(crate::config::ServiceWallet::can_unlock)
    })
}
