//! The node editor's draft, and the rules that turn it into a `NewNode`.
//!
//! The draft keeps every field as the text the browser posted. That is the
//! whole point: a rejected submission re-renders with what the operator typed
//! still in the boxes, rather than throwing their work away and asking them to
//! start over. Validation borrows the domain's own checks — `validate_node_ports`,
//! `NodeType::supports_storage_engine`, `parse_argv_text` — so the form cannot
//! drift from what `Repository::create_node` will accept.

use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    argv::{format_argv, parse_argv_text},
    core::node::{
        plan_available_node_ports, validate_node_ports, Network, NewNode, NodeConfig, NodeType,
        StorageEngine, DEFAULT_RPC_PORT,
    },
    runtime::RuntimeInstallation,
    types::NodeTypeTraits,
};

fn is_default_or_generic_binary(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return true;
    }
    let normalized = trimmed.replace('\\', "/");
    let filename = normalized.rsplit('/').next().unwrap_or(trimmed);
    let stem = filename.strip_suffix(".exe").unwrap_or(filename);
    let stem = stem.strip_suffix(".dll").unwrap_or(stem);
    NodeType::ALL.iter().any(|node_type| {
        let default_name = node_type.default_binary_name();
        let default_stem = default_name.strip_suffix(".exe").unwrap_or(default_name);
        filename.eq_ignore_ascii_case(default_name) || stem.eq_ignore_ascii_case(default_stem)
    })
}

/// Field name → the message to show under that field.
pub type FieldErrors = BTreeMap<&'static str, String>;

/// Every node field as posted text. This doubles as the form model, so the
/// field names appear in exactly one place.
#[derive(Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(default)]
pub struct NodeDraft {
    pub name: String,
    pub node_type: String,
    pub network: String,
    pub binary_path: String,
    pub args: String,
    pub runtime_version: String,
    pub storage_engine: String,
    pub rpc_port: String,
    pub p2p_port: String,
    pub ws_port: String,
    pub role: String,
    pub enable_rpc: String,
    pub rpc_configured: String,
    #[serde(default)]
    pub hermes_enabled: String,
    #[serde(default)]
    pub signer_backend: String,
    #[serde(default)]
    pub signer_key: String,
    #[serde(deserialize_with = "deserialize_plugins")]
    pub plugins: Vec<String>,
    /// Not node properties: submit flags. Each is raised by its own button, or
    /// by the auto-submitting client select, so no two intents share a name.
    #[serde(default)]
    pub suggest: String,
    #[serde(default)]
    pub client: String,
}

fn deserialize_plugins<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Helper {
        List(Vec<String>),
        Single(String),
    }

    match <Option<Helper> as serde::Deserialize>::deserialize(deserializer)? {
        Some(Helper::List(list)) => {
            let mut result = Vec::new();
            for item in list {
                for part in item.split(',') {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() && !result.iter().any(|r| r == trimmed) {
                        result.push(trimmed.to_string());
                    }
                }
            }
            Ok(result)
        }
        Some(Helper::Single(single)) => {
            let mut result = Vec::new();
            for part in single.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() && !result.iter().any(|r| r == trimmed) {
                    result.push(trimmed.to_string());
                }
            }
            Ok(result)
        }
        None => Ok(Vec::new()),
    }
}

impl NodeDraft {
    /// Whether JSON-RPC API is enabled for this draft.
    pub fn is_rpc_enabled(&self) -> bool {
        if !self.rpc_configured.trim().is_empty() {
            return matches!(
                self.enable_rpc.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "on" | "yes"
            );
        }
        if self.rpc_port.trim() == "0" {
            return false;
        }
        let role = self.role.trim();
        if !role.is_empty()
            && (role == "relay" || role == "validator")
            && self.enable_rpc.trim().is_empty()
        {
            return false;
        }
        !self.rpc_port.trim().is_empty()
    }

    /// Whether Hermes guest agent is enabled for this draft.
    pub fn is_hermes_enabled(&self) -> bool {
        self.hermes_enabled.trim() != "0"
    }

    pub fn selected_plugins(&self) -> Vec<crate::catalog::PluginId> {
        use std::str::FromStr;
        self.plugins
            .iter()
            .filter_map(|name| crate::catalog::PluginId::from_str(name).ok())
            .collect()
    }

    pub fn resolved_role(&self) -> Option<crate::roles::NodeRole> {
        let role = self.role.trim();
        if role.is_empty() || role == "relay" || role == "custom" {
            None
        } else {
            crate::roles::NodeRole::from_persist_key(role)
        }
    }
    /// The operator asked for a free port block rather than a save.
    pub fn wants_suggested_ports(&self) -> bool {
        !self.suggest.trim().is_empty()
    }

    /// The operator changed client, so the form should re-render with that
    /// client's defaults rather than reporting errors on a half-filled form.
    pub fn wants_client_defaults(&self) -> bool {
        !self.client.trim().is_empty()
    }

    /// Adopt the selected client's storage default when its current choice is
    /// not usable, which is what the old desktop editor did on every change.
    /// Also updates binary path if it was empty or held a default binary name.
    pub fn with_client_defaults(mut self) -> Self {
        if let Some(node_type) = self.parsed_type() {
            if !self
                .parsed_storage()
                .is_some_and(|storage| node_type.supports_storage_engine(storage))
            {
                self.storage_engine = node_type.default_storage_engine().to_string();
            }
            if is_default_or_generic_binary(&self.binary_path) {
                self.binary_path = node_type.default_binary_name().to_string();
            }
        }
        self
    }

    /// Infers client type and storage engine from binary path if one can be detected.
    pub fn with_inferred_client(mut self) -> Self {
        if let Some(inferred) = NodeType::infer_from_str(&self.binary_path) {
            self.node_type = inferred.to_string();
            if !self
                .parsed_storage()
                .is_some_and(|storage| inferred.supports_storage_engine(storage))
            {
                self.storage_engine = inferred.default_storage_engine().to_string();
            }
        }
        self
    }
}

/// Either a node the repository will accept, or the reasons it will not.
pub enum DraftOutcome {
    Valid(NewNode),
    Invalid(FieldErrors),
}

/// A field that failed without belonging to one visible input.
const GENERAL: &str = "general";

impl NodeDraft {
    /// A blank draft carries the defaults an operator would usually keep: the first
    /// client, mainnet, its matching storage, its standard binary, and the conventional RPC / RPC+1
    /// pair. A collision is not guessed away — "Suggest free ports" resolves it
    /// against the fleet and the host.
    pub fn blank() -> Self {
        let node_type = NodeType::ALL[0];
        Self {
            node_type: node_type.to_string(),
            network: Network::Mainnet.to_string(),
            binary_path: node_type.default_binary_name().to_string(),
            storage_engine: node_type.default_storage_engine().to_string(),
            rpc_port: DEFAULT_RPC_PORT.to_string(),
            p2p_port: (DEFAULT_RPC_PORT + 1).to_string(),
            role: "rpc-api".to_string(),
            enable_rpc: "1".to_string(),
            hermes_enabled: "1".to_string(),
            plugins: vec!["RpcServer".to_string()],
            ..Self::default()
        }
    }

    /// Creates a blank draft prioritizing installed workspace runtimes.
    pub fn blank_with_installations(installations: &[RuntimeInstallation]) -> Self {
        let mut draft = Self::blank();
        if let Some(node_type) = draft.parsed_type() {
            if let Some(installation) = installations.iter().find(|i| i.node_type == node_type) {
                draft.binary_path = installation.binary_path.display().to_string();
                draft.runtime_version = installation.version.clone();
            }
        }
        draft
    }

    /// Creates a blank draft prioritizing installed workspace runtimes and automatically
    /// allocating free non-overlapping ports across the existing fleet.
    pub fn blank_with_installations_and_fleet(
        installations: &[RuntimeInstallation],
        fleet: &[NodeConfig],
    ) -> Self {
        let mut draft = Self::blank_with_installations(installations);
        draft.name = format!("node-{:02}", fleet.len() + 1);
        if let Some(suggested) = draft.suggest_ports(fleet, None) {
            draft.p2p_port = suggested.p2p_port;
            draft.rpc_port = suggested.rpc_port;
            draft.ws_port = suggested.ws_port;
        }
        draft
    }

    pub fn from_node(node: &NodeConfig) -> Self {
        Self::from_node_with_role_and_plugins(node, None, &[])
    }

    pub fn from_node_with_role_and_plugins(
        node: &NodeConfig,
        role: Option<crate::roles::NodeRole>,
        plugin_states: &[crate::catalog::PluginState],
    ) -> Self {
        let enable_rpc = if node.rpc_port == 0 {
            "0".to_string()
        } else {
            "1".to_string()
        };
        let plugins = plugin_states
            .iter()
            .filter(|p| p.enabled)
            .map(|p| p.plugin_id.to_string())
            .collect();
        Self {
            name: node.name.clone(),
            node_type: node.node_type.to_string(),
            network: node.network.to_string(),
            binary_path: node.binary_path.display().to_string(),
            args: format_argv(&node.args),
            runtime_version: node.runtime_version.clone(),
            storage_engine: node.storage_engine.to_string(),
            rpc_port: if node.rpc_port == 0 {
                String::new()
            } else {
                node.rpc_port.to_string()
            },
            p2p_port: node.p2p_port.to_string(),
            ws_port: node
                .ws_port
                .map_or_else(String::new, |port| port.to_string()),
            role: role.map_or_else(String::new, |r| r.persist_key().to_string()),
            enable_rpc,
            plugins,
            ..Self::default()
        }
    }

    pub fn from_node_with_role_plugins_and_signer(
        node: &NodeConfig,
        role: Option<crate::roles::NodeRole>,
        plugin_states: &[crate::catalog::PluginState],
        signer: Option<&crate::signing::SignerKeyRef>,
    ) -> Self {
        let mut draft = Self::from_node_with_role_and_plugins(node, role, plugin_states);
        if let Some(s) = signer {
            draft.signer_backend = s.backend_id.clone();
            draft.signer_key = s.key_id.clone();
        }
        draft
    }

    pub fn resolved_signer_key(&self) -> Option<crate::signing::SignerKeyRef> {
        let backend = self.signer_backend.trim();
        let key = self.signer_key.trim();
        if !backend.is_empty() && !key.is_empty() {
            crate::signing::SignerKeyRef::new(backend, key).ok()
        } else {
            None
        }
    }

    /// The client drives nearly every other choice, so it is resolved first and
    /// the rest of the form renders from it.
    pub fn parsed_type(&self) -> Option<NodeType> {
        self.node_type.trim().parse::<NodeType>().ok()
    }

    pub fn parsed_network(&self) -> Option<Network> {
        self.network.trim().parse::<Network>().ok()
    }

    pub fn parsed_storage(&self) -> Option<StorageEngine> {
        self.storage_engine.trim().parse::<StorageEngine>().ok()
    }

    /// Storage engines the selected client can actually use. Neo N3 clients
    /// choose between LevelDB and RocksDB; neither Neo X client offers one.
    pub fn storage_options(&self) -> Vec<String> {
        let Some(node_type) = self.parsed_type() else {
            return Vec::new();
        };
        StorageEngine::ALL
            .iter()
            .filter(|engine| node_type.supports_storage_engine(**engine))
            .map(|engine| engine.to_string())
            .collect()
    }

    /// Whether storage is a real operator choice for this client.
    pub fn storage_is_selectable(&self) -> bool {
        self.storage_options().len() > 1
    }

    /// What the selected client stores its chain in, when it is not a choice.
    pub fn storage_note(&self) -> Option<String> {
        let node_type = self.parsed_type()?;
        if self.storage_is_selectable() {
            return None;
        }
        Some(format!(
            "{} uses {}. There is nothing to choose here.",
            node_type,
            node_type.storage_label(node_type.default_storage_engine()),
        ))
    }

    pub fn includes_ws(&self) -> bool {
        !self.ws_port.trim().is_empty()
    }

    /// Ask the planner for a port block no other node claims and that is free on
    /// this host. The current RPC value is a hint, not a promise: the planner
    /// walks forward from it.
    pub fn suggest_ports(&self, nodes: &[NodeConfig], current_id: Option<&str>) -> Option<Self> {
        let preferred = self
            .rpc_port
            .trim()
            .parse::<u16>()
            .unwrap_or(DEFAULT_RPC_PORT);
        let assignment =
            plan_available_node_ports(nodes, current_id, preferred, self.includes_ws()).ok()?;
        let mut next = self.clone();
        next.rpc_port = assignment.rpc_port.to_string();
        next.p2p_port = assignment.p2p_port.to_string();
        next.ws_port = assignment
            .ws_port
            .map_or_else(String::new, |port| port.to_string());
        Some(next)
    }

    /// Validate against the domain rules and the rest of the fleet.
    ///
    /// `existing` is every node in the workspace and `current_id` is set when
    /// editing, so a node never collides with itself.
    pub fn validate(&self, existing: &[NodeConfig], current_id: Option<&str>) -> DraftOutcome {
        let mut errors = FieldErrors::new();

        let name = self.name.trim();
        if name.is_empty() {
            errors.insert("name", "A node needs a name.".to_string());
        } else if find_name(existing, current_id, name).is_some() {
            errors.insert(
                "name",
                format!("\"{name}\" is already used by another node."),
            );
        }

        let node_type = self
            .parsed_type()
            .or_else(|| NodeType::infer_from_str(&self.binary_path));
        if self.node_type.trim().is_empty() && node_type.is_none() {
            errors.insert(
                "node_type",
                "Choose which client this node runs.".to_string(),
            );
        } else if node_type.is_none() {
            errors.insert(
                "node_type",
                format!("{} is not a supported client.", self.node_type.trim()),
            );
        }

        let network = self.parsed_network();
        if network.is_none() {
            errors.insert(
                "network",
                format!("{} is not a network.", self.network.trim()),
            );
        }

        if self.binary_path.trim().is_empty() && node_type.is_none() {
            errors.insert(
                "binary_path",
                "The node binary path is required.".to_string(),
            );
        }

        let args = match parse_argv_text(&self.args) {
            Ok(args) => Some(args),
            Err(error) => {
                errors.insert("args", error.to_string());
                None
            }
        };

        let storage = match (self.parsed_type(), self.parsed_storage(), node_type) {
            (None, _, Some(inferred)) => {
                if let Some(explicit_storage) = self.parsed_storage() {
                    if inferred.supports_storage_engine(explicit_storage) {
                        Some(explicit_storage)
                    } else {
                        Some(inferred.default_storage_engine())
                    }
                } else {
                    Some(inferred.default_storage_engine())
                }
            }
            (_, storage, Some(nt)) => storage.or_else(|| Some(nt.default_storage_engine())),
            (_, storage, None) => storage,
        };
        match storage {
            None => errors.insert("storage_engine", "Choose a storage engine.".to_string()),
            Some(storage)
                if node_type
                    .is_some_and(|node_type| !node_type.supports_storage_engine(storage)) =>
            {
                errors.insert(
                    "storage_engine",
                    format!(
                        "{} cannot run on {storage} storage.",
                        node_type.map_or_else(String::new, |node_type| node_type.to_string()),
                    ),
                )
            }
            Some(_) => None,
        };

        let is_rpc_enabled = self.is_rpc_enabled();
        let rpc_port = if is_rpc_enabled {
            if self.rpc_port.trim().is_empty() {
                let assigned =
                    plan_available_node_ports(existing, current_id, DEFAULT_RPC_PORT, false)
                        .map(|a| a.rpc_port)
                        .unwrap_or(10332);
                Some(assigned)
            } else {
                parse_port(self.rpc_port.trim(), "RPC", "rpc_port", &mut errors)
            }
        } else {
            Some(0u16)
        };
        let p2p_port = if self.p2p_port.trim().is_empty() {
            let assigned = plan_available_node_ports(existing, current_id, DEFAULT_RPC_PORT, false)
                .map(|a| a.p2p_port)
                .unwrap_or(20333);
            Some(assigned)
        } else {
            parse_port(self.p2p_port.trim(), "P2P", "p2p_port", &mut errors)
        };
        let ws_port = if is_rpc_enabled {
            parse_optional_port(self.ws_port.trim(), &mut errors)
        } else {
            None
        };

        if let (Some(rpc_port), Some(p2p_port)) = (rpc_port, p2p_port) {
            let ws_value = ws_port.flatten();
            if let Err(error) = validate_node_ports(rpc_port, p2p_port, ws_value) {
                add_port_error(&mut errors, &error.to_string());
            }
            // Only report a fleet collision once the numbers are internally
            // sound, so one mistake produces one message.
            if !has_port_error(&errors) {
                if let Some((port, owner)) =
                    find_port_conflict(existing, current_id, rpc_port, p2p_port, ws_value)
                {
                    let conflict_key = if ws_value.is_some_and(|ws| ws == port) {
                        "ws_port"
                    } else if port == p2p_port {
                        "p2p_port"
                    } else {
                        "rpc_port"
                    };
                    errors.insert(
                        conflict_key,
                        format!(
                            "Port {port} is already used by \"{}\". Choose free ports, or use Suggest free ports.",
                            owner.name
                        ),
                    );
                }
            }
        }

        if !self.signer_backend.trim().is_empty() || !self.signer_key.trim().is_empty() {
            let backend = self.signer_backend.trim();
            let key = self.signer_key.trim();
            if backend.is_empty() || key.is_empty() {
                errors.insert(
                    "signer_backend",
                    "Signer backend and key identifier must both be specified or both left blank."
                        .to_string(),
                );
            } else if let Err(err) = crate::signing::SignerKeyRef::new(backend, key) {
                errors.insert("signer_backend", format!("Invalid signer key: {err}"));
            }
        }

        if !errors.is_empty() {
            return DraftOutcome::Invalid(errors);
        }

        // Each `None` above already inserted an error, so reaching `None` here
        // means those two paths disagree; the operator gets a readable message
        // rather than a form that silently refuses to save.
        let parsed = Parsed {
            name,
            node_type,
            network,
            storage,
            args,
            rpc_port,
            p2p_port,
            ws_port: ws_port.flatten(),
        };
        match parsed.into_node(self) {
            Some(node) => DraftOutcome::Valid(node),
            None => {
                let mut errors = FieldErrors::new();
                errors.insert(
                    GENERAL,
                    "The form could not be read. Reload the page and try again.".to_string(),
                );
                DraftOutcome::Invalid(errors)
            }
        }
    }
}

/// Every field of a draft that has already been parsed and checked.
struct Parsed<'a> {
    name: &'a str,
    node_type: Option<NodeType>,
    network: Option<Network>,
    storage: Option<StorageEngine>,
    args: Option<Vec<String>>,
    rpc_port: Option<u16>,
    p2p_port: Option<u16>,
    ws_port: Option<u16>,
}

impl Parsed<'_> {
    /// Assemble the accepted node. The `?` marks are a consistency check between
    /// the error map and this path, not a silent failure.
    fn into_node(self, draft: &NodeDraft) -> Option<NewNode> {
        let node_type = self.node_type?;
        let binary_path = if draft.binary_path.trim().is_empty() {
            PathBuf::from(node_type.default_binary_name())
        } else {
            PathBuf::from(draft.binary_path.trim())
        };
        Some(NewNode {
            name: self.name.to_string(),
            node_type,
            network: self.network?,
            binary_path,
            args: self.args?,
            runtime_version: normalize_version(draft.runtime_version.trim()),
            storage_engine: self.storage?,
            rpc_port: self.rpc_port?,
            p2p_port: self.p2p_port?,
            ws_port: self.ws_port,
        })
    }
}

fn parse_port(raw: &str, label: &str, key: &'static str, errors: &mut FieldErrors) -> Option<u16> {
    if raw.is_empty() {
        errors.insert(key, format!("A {label} port is required."));
        return None;
    }
    match raw.parse::<u16>() {
        Ok(0) => {
            errors.insert(key, format!("The {label} port must be above 0."));
            None
        }
        Ok(port) => Some(port),
        Err(_) => {
            errors.insert(key, format!("\"{raw}\" is not a port number (1-65535)."));
            None
        }
    }
}

/// Blank means "no WebSocket port", which is a valid choice, not an error. The
/// outer `Option` distinguishes those two states; `flatten()` collapses it once
/// validation has passed.
fn parse_optional_port(raw: &str, errors: &mut FieldErrors) -> Option<Option<u16>> {
    if raw.is_empty() {
        return None;
    }
    match raw.parse::<u16>() {
        Ok(0) => {
            errors.insert("ws_port", "The WebSocket port must be above 0.".to_string());
            Some(None)
        }
        Ok(port) => Some(Some(port)),
        Err(_) => {
            errors.insert(
                "ws_port",
                format!("\"{raw}\" is not a port number (1-65535)."),
            );
            Some(None)
        }
    }
}

fn has_port_error(errors: &FieldErrors) -> bool {
    ["rpc_port", "p2p_port", "ws_port"]
        .iter()
        .any(|key| errors.contains_key(*key))
}

/// `validate_node_ports` reports the first problem as a sentence. Attach it to
/// the field it names so the mark appears where the mistake is, not in a banner
/// above the form.
fn add_port_error(errors: &mut FieldErrors, message: &str) {
    let key = if message.contains("WebSocket") {
        "ws_port"
    } else if message.contains("P2P") {
        "p2p_port"
    } else {
        "rpc_port"
    };
    errors.entry(key).or_insert_with(|| message.to_string());
}

fn find_name<'a>(
    existing: &'a [NodeConfig],
    current_id: Option<&str>,
    name: &str,
) -> Option<&'a NodeConfig> {
    existing.iter().find(|node| {
        current_id.is_none_or(|current| node.id != current) && node.name.eq_ignore_ascii_case(name)
    })
}

/// Ports collide across nodes as well as within one node: a new node that takes
/// an existing node's P2P port for its own RPC port would start fine and then
/// fail to bind, so it is refused here rather than at launch.
fn find_port_conflict<'a>(
    existing: &'a [NodeConfig],
    current_id: Option<&str>,
    rpc_port: u16,
    p2p_port: u16,
    ws_port: Option<u16>,
) -> Option<(u16, &'a NodeConfig)> {
    let wanted = [
        (rpc_port > 0).then_some(rpc_port),
        (p2p_port > 0).then_some(p2p_port),
        ws_port.filter(|&port| port > 0),
    ];
    for node in existing {
        if current_id.is_some_and(|current| current == node.id) {
            continue;
        }
        let held = [
            (node.rpc_port > 0).then_some(node.rpc_port),
            (node.p2p_port > 0).then_some(node.p2p_port),
            node.ws_port.filter(|&port| port > 0),
        ];
        for port in wanted.iter().flatten().copied() {
            if held.iter().flatten().copied().any(|held| held == port) {
                return Some((port, node));
            }
        }
    }
    None
}

/// A blank version means "whatever is current", which the workspace already
/// spells `latest`; an empty string would render as nothing.
fn normalize_version(raw: &str) -> String {
    if raw.is_empty() {
        "latest".to_string()
    } else {
        raw.to_string()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/web/node_form/tests.rs"]
mod tests;
