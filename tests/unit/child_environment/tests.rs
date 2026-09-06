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
    ];
    let mut command = Command::new("not-started-by-this-test");
    remove_control_plane_environment(
        &mut command,
        control_plane_names
            .iter()
            .copied()
            .chain(["NEONEXUS_NODE_MARKER"]),
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
    assert!(
        !overrides.contains_key(&OsString::from("NEONEXUS_NODE_MARKER")),
        "an unrelated node setting was removed"
    );
}

#[test]
fn only_the_control_plane_namespaces_are_classified_as_control_plane_state() {
    assert!(is_control_plane_environment(
        "NEONEXUS_SIGNER_SERVICE_TOKEN".as_ref()
    ));
    assert!(is_control_plane_environment(
        "neonexus_signer_future_secret".as_ref()
    ));
    assert!(is_control_plane_environment(
        "NEONEXUS_WEB_TOKEN_FILE".as_ref()
    ));
    assert!(is_control_plane_environment(
        "neonexus_web_future_secret".as_ref()
    ));
    for ordinary in [
        "NEONEXUS_SIGNER",
        "NEONEXUS_SIGNERS_SERVICE_TOKEN",
        "NEONEXUS_NODE_SIGNER_ENDPOINT",
        "NEONEXUS_WEB",
        "NEONEXUS_WEBHOOK_URL",
        "PATH",
    ] {
        assert!(
            !is_control_plane_environment(ordinary.as_ref()),
            "{ordinary} is outside the control-plane namespace"
        );
    }
}
