//! Environment boundaries shared by every child process NeoNexus launches.

use std::{ffi::OsStr, process::Command};

/// Web and signer control-plane settings belong to NeoNexus, never to a runtime
/// binary, node, sidecar, or helper it launches.
///
/// Match the namespace instead of enumerating today's four settings. That keeps
/// a future credential-file or workload-key setting from becoming inherited
/// merely because this boundary was not updated in the same release. The match
/// is ASCII case-insensitive because environment names are case-insensitive on
/// Windows, where `neonexus_signer_service_token` names the same setting as the
/// upper-case spelling read by [`std::env::var`].
const CONTROL_PLANE_ENV_PREFIXES: [&str; 2] = ["NEONEXUS_SIGNER_", "NEONEXUS_WEB_"];

/// Record explicit removals on a child command for every NeoNexus control-plane
/// variable present in this process. `Command::env_remove` blocks inheritance;
/// it does not mutate NeoNexus's own environment.
pub(crate) fn scrub_control_plane_environment(command: &mut Command) {
    remove_control_plane_environment(command, std::env::vars_os().map(|(name, _value)| name));
}

fn remove_control_plane_environment(
    command: &mut Command,
    names: impl IntoIterator<Item = impl AsRef<OsStr>>,
) {
    for name in names {
        let name = name.as_ref();
        if is_control_plane_environment(name) {
            command.env_remove(name);
        }
    }
}

fn is_control_plane_environment(name: &OsStr) -> bool {
    let name = name.to_string_lossy();
    CONTROL_PLANE_ENV_PREFIXES.iter().any(|prefix| {
        name.get(..prefix.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
    })
}

#[cfg(test)]
#[path = "../tests/unit/child_environment/tests.rs"]
mod tests;
