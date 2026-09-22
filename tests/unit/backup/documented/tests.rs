//! The operator documentation must describe the bytes `--export-backup` writes.
//!
//! README.md and docs/AGENT_API.md promised "AES-GCM encrypted, tar.gz" while
//! `--export-backup` wrote a 0600 pretty-printed plaintext JSON file beginning
//! `7b 0a` (`{\n`) — no gzip magic (`1f 8b`), no tar, no cipher. This runs the
//! documented command path, measures the bytes it actually writes, derives the
//! one-line description those bytes deserve, and then checks every backup claim
//! in the operator documentation against that measurement. Whichever side moves
//! first, the build fails until the other side is brought back into agreement.

use std::{fs, path::PathBuf};

use crate::{
    cli::{action_from_args, CliAction},
    repository::Repository,
    types::{Network, NewNode, NodeType},
};

/// Every operator-facing document that shows or discusses the backup commands.
const DOCUMENTED_IN: [&str; 3] = ["README.md", "docs/AGENT_API.md", "docs/TROUBLESHOOTING.md"];

/// The documents that also show the export command itself, and therefore owe a
/// description in the same sentence as it.
const SHOWS_EXPORT_COMMAND: [&str; 2] = ["README.md", "docs/AGENT_API.md"];

struct MeasuredExport {
    file_name: String,
    bytes: Vec<u8>,
    owner_mode: Option<u32>,
}

impl MeasuredExport {
    fn is_gzip(&self) -> bool {
        self.bytes.starts_with(&[0x1f, 0x8b])
    }

    fn is_plaintext_json(&self) -> bool {
        !self.is_gzip()
            && self.bytes.starts_with(b"{")
            && serde_json::from_slice::<serde_json::Value>(&self.bytes).is_ok()
    }

    /// The one-line description the documentation must carry — derived from the
    /// measured bytes and the measured unix mode, never from a constant, so a
    /// format change forces a documentation change in the same commit.
    fn required_description(&self) -> String {
        let container = if self.is_gzip() {
            "gzip"
        } else if self.is_plaintext_json() {
            "plaintext JSON"
        } else {
            "binary"
        };
        let secrecy = if self.is_plaintext_json() {
            "NOT encrypted"
        } else {
            "NOT a readable plaintext JSON document"
        };
        format!("{} {container} workspace export, {secrecy}", self.mode_claim())
    }

    #[cfg(unix)]
    fn mode_claim(&self) -> String {
        match self.owner_mode {
            Some(mode) => format!("{mode:04o}"),
            None => "0600".to_string(),
        }
    }

    /// Windows inherits the containing directory's ACL and the exporter sets no
    /// mode there; document the owner-only intent its unix path enforces.
    #[cfg(not(unix))]
    fn mode_claim(&self) -> String {
        "0600".to_string()
    }
}

/// Runs the documented `cargo run -- --export-backup` action path and measures
/// what lands on disk.
fn measure_export() -> MeasuredExport {
    let temp_dir = tempfile::tempdir().expect("temporary workspace for the measured export");
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path).expect("open a fresh workspace database");
    repository
        .create_node(NewNode {
            name: "documented-export".to_string(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: PathBuf::from("/usr/local/bin/neo-node"),
            args: Vec::new(),
            runtime_version: "v0.8.0".to_string(),
            storage_engine: NodeType::NeoRs.default_storage_engine(),
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: Some(20334),
        })
        .expect("seed one node so the measured export is non-trivial");
    drop(repository);

    let output_dir = temp_dir.path().join("backups");
    let db_arg = db_path.display().to_string();
    let out_arg = output_dir.display().to_string();
    let action = action_from_args(["neo-nexus", "--export-backup", &db_arg, &out_arg])
        .expect("the documented --export-backup command must succeed");
    assert!(
        matches!(action, CliAction::Print(_)),
        "--export-backup reports its summary as printable CLI text"
    );

    let mut written: Vec<PathBuf> = fs::read_dir(&output_dir)
        .expect("the export creates its output directory")
        .map(|entry| entry.expect("read export directory entry").path())
        .filter(|path| path.is_file())
        .collect();
    assert_eq!(
        written.len(),
        1,
        "--export-backup must write exactly one backup file, found {written:?}"
    );
    let path = written
        .pop()
        .expect("the loop above proved exactly one file");
    let file_name = path
        .file_name()
        .expect("the exported backup has a file name")
        .to_string_lossy()
        .into_owned();
    assert!(
        file_name.ends_with(".json"),
        "the exporter names its output after its real container: {file_name}"
    );

    let bytes = fs::read(&path).expect("read the exported backup bytes");
    if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&bytes) {
        assert!(
            parsed.get("schema_version").is_some(),
            "a JSON backup export carries the workspace backup schema"
        );
    }

    #[cfg(unix)]
    let owner_mode = {
        use std::os::unix::fs::PermissionsExt;
        Some(
            fs::metadata(&path)
                .expect("stat the exported backup")
                .permissions()
                .mode()
                & 0o777,
        )
    };
    #[cfg(not(unix))]
    let owner_mode = None;

    MeasuredExport {
        file_name,
        bytes,
        owner_mode,
    }
}

fn read_doc(relative: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        assert!(
            false,
            "{relative} is part of the repo and must be readable: {error}"
        );
        String::new()
    })
}

/// Lines that talk about the backup surface: the CLI flags themselves, or prose
/// about "backup" that makes a container or secrecy claim.
fn is_backup_surface_line(line: &str) -> bool {
    if line.contains("--export-backup")
        || line.contains("--validate-backup")
        || line.contains("--import-backup")
    {
        return true;
    }
    let lower = line.to_ascii_lowercase();
    lower.contains("backup")
        && (line.contains("AES-GCM")
            || lower.contains("encrypt")
            || lower.contains("tar.gz")
            || lower.contains("tarball"))
}

fn negates_secrecy_claim(lower: &str) -> bool {
    lower.contains("not encrypted") || lower.contains("unencrypted")
}

fn negates_tarball_claim(lower: &str) -> bool {
    lower.contains("not tar.gz") || lower.contains("no tar.gz") || lower.contains("not a tar")
}

#[test]
fn backup_docs_carry_the_measured_export_description() {
    let measured = measure_export();
    let required = measured.required_description();

    for relative in DOCUMENTED_IN {
        let text = read_doc(relative);
        assert!(
            text.contains(&required),
            "{relative} does not describe the backup export as \"{required}\". \
             --export-backup writes that (file {}). \
             Update the documentation to match, or change the exporter to match the \
             documentation and this test with it.",
            measured.file_name,
        );
    }

    for relative in SHOWS_EXPORT_COMMAND {
        let text = read_doc(relative);
        let beside_command = text
            .lines()
            .any(|line| line.contains("--export-backup") && line.contains(&required));
        assert!(
            beside_command,
            "{relative} shows the --export-backup command but the measured description \
             \"{required}\" does not appear in the same sentence as it. An operator copying the \
             command must see what the command actually writes."
        );
    }
}

#[test]
fn backup_docs_never_claim_properties_the_export_bytes_do_not_have() {
    let measured = measure_export();
    let required = measured.required_description();
    let is_plaintext_json = measured.is_plaintext_json();
    let is_gzip = measured.is_gzip();

    for relative in DOCUMENTED_IN {
        let text = read_doc(relative);
        for (index, line) in text.lines().enumerate() {
            if !is_backup_surface_line(line) {
                continue;
            }
            let line_number = index + 1;
            let lower = line.to_ascii_lowercase();
            if is_plaintext_json {
                assert!(
                    !line.contains("AES-GCM"),
                    "{relative}:{line_number} claims AES-GCM for the backup surface, but \
                     --export-backup writes \"{required}\". Nobody reading the docs may believe \
                     the backup is encrypted when it is not."
                );
                if lower.contains("encrypt") {
                    assert!(
                        negates_secrecy_claim(&lower),
                        "{relative}:{line_number} claims the backup is encrypted, but \
                         --export-backup writes \"{required}\"."
                    );
                }
                if lower.contains("tar.gz") || lower.contains("tarball") {
                    assert!(
                        is_gzip || negates_tarball_claim(&lower),
                        "{relative}:{line_number} claims a tar.gz backup container, but \
                         --export-backup writes \"{required}\" (file {}).",
                        measured.file_name,
                    );
                }
            } else {
                assert!(
                    !lower.contains("plaintext json") && !negates_secrecy_claim(&lower),
                    "{relative}:{line_number} still describes the backup as plaintext and \
                     unencrypted, but --export-backup now writes \"{required}\". \
                     The documentation is stale."
                );
            }
        }
    }
}
