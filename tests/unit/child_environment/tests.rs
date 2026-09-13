use std::{collections::BTreeMap, ffi::OsString, process::Command};

use super::{is_control_plane_environment, remove_control_plane_environment};

#[test]
fn every_control_plane_setting_is_removed_from_the_child() {
    let control_plane_names = [
        "NEONEXUS_SIGNER_SERVICE_URL",
        "NEONEXUS_SIGNER_SERVICE_TOKEN",
        "NEONEXUS_SIGNER_SERVICE_ORIGIN",
        "NEONEXUS_SIGNER_SERVICE_TIMEOUT_SECONDS",
        "NEONEXUS_SIGNER_TIMEOUT_SECONDS",
        // A future setting is covered by the namespace boundary without adding
        // its secret-bearing name to every process launcher.
        "NEONEXUS_SIGNER_ADMIN_WORKLOAD_KEY_FILE",
        // Windows treats this as the same name as the upper-case spelling.
        "neonexus_signer_admin_token_file",
        "NEONEXUS_WEB_TOKEN",
        "NEONEXUS_WEB_TOKEN_FILE",
        "NEONEXUS_WEB_PUBLIC_ORIGIN",
        "neonexus_web_future_credential_file",
        // The consensus signer endpoint and the identity expected behind it.
        // These sit outside `NEONEXUS_SIGNER_` and were inherited by every node
        // process while the boundary was a list of two prefixes.
        "NEONEXUS_LOCAL_SIGNER_ENDPOINT",
        "NEONEXUS_LOCAL_SIGNER_PUBLIC_KEY",
        "NEONEXUS_LOCAL_SIGNER_NETWORK_MAGIC",
        "NEONEXUS_METRICS_TOKEN",
    ];
    let mut command = Command::new("not-started-by-this-test");
    remove_control_plane_environment(
        &mut command,
        control_plane_names
            .iter()
            .copied()
            .chain(["PATH", "HOME", "NEO_NODE_MARKER"]),
    );

    let overrides = command
        .get_envs()
        .map(|(name, value)| (name.to_os_string(), value.map(OsString::from)))
        .collect::<BTreeMap<_, _>>();
    for name in control_plane_names {
        assert_eq!(
            overrides.get(&OsString::from(name)),
            Some(&None),
            "{name} was still inheritable"
        );
    }
    for inherited in ["PATH", "HOME", "NEO_NODE_MARKER"] {
        assert!(
            !overrides.contains_key(&OsString::from(inherited)),
            "{inherited} is not NeoNexus's to withhold from a node"
        );
    }
}

#[test]
fn the_whole_neonexus_namespace_is_withheld_from_a_child() {
    for withheld in [
        "NEONEXUS_SIGNER_SERVICE_TOKEN",
        "neonexus_signer_future_secret",
        "NEONEXUS_WEB_TOKEN_FILE",
        "NEONEXUS_LOCAL_SIGNER_ENDPOINT",
        "NEONEXUS_DATA_DIR",
        // The point of matching the namespace: a setting nobody has written yet
        // is withheld without this list being revisited.
        "NEONEXUS_SOME_FUTURE_CREDENTIAL",
        "neonexus_lowercase_on_windows",
    ] {
        assert!(
            is_control_plane_environment(withheld.as_ref()),
            "{withheld} belongs to NeoNexus and must not reach a node"
        );
    }

    // A node's own environment is its operator's business, not ours to strip.
    for inherited in [
        "NEONEXUS",
        "NEONEXUSX_THING",
        "NEO_NODE_SIGNER_ENDPOINT",
        "PATH",
        "HOME",
    ] {
        assert!(
            !is_control_plane_environment(inherited.as_ref()),
            "{inherited} is outside NeoNexus's namespace"
        );
    }
}
