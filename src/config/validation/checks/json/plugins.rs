use serde_json::Value;

use super::super::{super::model::ConfigValidationReport, paths::json_path};

/// neo-cli enables a plugin by the presence of its assembly under
/// `Plugins/<Name>/`, not by any configuration key. The only plugin-related key
/// in `config.json` is where the node fetches packages from.
///
/// This used to assert a top-level `Plugins` **array** of `{"Name": …}` — a
/// NeoNexus-internal manifest that neo-cli never reads. It passed on every
/// export while telling an operator nothing true about their node.
pub(in crate::config::validation) fn check_neo_cli_plugin_source(
    report: &mut ConfigValidationReport,
    value: &Value,
) {
    let url = json_path(
        value,
        &["ApplicationConfiguration", "Plugins", "DownloadUrl"],
    )
    .and_then(Value::as_str);
    match url {
        // The plugins moved out of neo-modules, which is archived and publishes
        // no releases, so a node pointed there can install nothing.
        Some(url) if url.contains("neo-modules") => report.warning(
            "Plugin source",
            format!("{url} is the archived neo-modules repository; plugin installs will fail."),
        ),
        Some(url) if url.starts_with("https://") => report.pass(
            "Plugin source",
            format!("Plugin packages are fetched from {url}."),
        ),
        Some(url) => report.critical(
            "Plugin source",
            format!("Plugin download URL {url} is not HTTPS."),
        ),
        None => report.critical(
            "Plugin source",
            "ApplicationConfiguration.Plugins.DownloadUrl is missing; the node cannot install plugins.",
        ),
    }
}
