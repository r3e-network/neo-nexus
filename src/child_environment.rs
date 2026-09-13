//! Environment boundaries shared by every child process NeoNexus launches.

use std::{ffi::OsStr, process::Command};

/// NeoNexus's own settings belong to NeoNexus, never to a runtime binary, node,
/// sidecar, or helper it launches.
///
/// The whole namespace, not a list of prefixes. Scrubbing only
/// `NEONEXUS_SIGNER_` and `NEONEXUS_WEB_` left `NEONEXUS_LOCAL_SIGNER_ENDPOINT`
/// and `NEONEXUS_LOCAL_SIGNER_PUBLIC_KEY` — the consensus signer endpoint and
/// the identity behind it — inherited by every node process, which is exactly
/// the cross-instance reach the lease model exists to prevent. A node is
/// configured by its managed config file and its command line; nothing it needs
/// arrives this way, so the safe boundary is the namespace itself and a setting
/// added later is excluded by default.
///
/// The match is ASCII case-insensitive because environment names are
/// case-insensitive on Windows, where `neonexus_signer_service_token` names the
/// same setting as the upper-case spelling read by [`std::env::var`].
const CONTROL_PLANE_ENV_PREFIXES: [&str; 1] = ["NEONEXUS_"];

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
