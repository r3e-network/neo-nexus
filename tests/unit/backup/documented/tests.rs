//! The operator documentation, and the binary's own `--help`, must describe the
//! bytes `--export-backup` writes.
//!
//! README.md and docs/AGENT_API.md promised "AES-GCM encrypted, tar.gz", the
//! `--help` line promised "an encrypted full workspace backup archive", and
//! docs/MIGRATION-v4.3.md sent operators to a `make backup-export FORMAT=tar.gz`
//! target that never existed — while `--export-backup` wrote a 0600
//! pretty-printed plaintext JSON file beginning `7b 0a` (`{\n`): no gzip magic
//! (`1f 8b`), no tar, no cipher. This runs the documented command path, measures
//! the bytes it writes, derives the one-line description those bytes deserve,
//! and checks every backup claim on every operator surface against that
//! measurement. Whichever side moves first, the build fails until the other side
//! is brought back into agreement.
//!
//! Claims are judged per *statement*, not per line: a paragraph, list item,
//! table row, heading, CLI option entry, or blank-line-separated group inside a
//! code block, with its lines joined, so a claim wrapped across two lines is
//! still one claim. A statement is about the backup when it names the backup or
//! the workspace export, or sits under a heading that names the backup or
//! disaster recovery. Statements about anything else — signer, wallet, session
//! and TLS prose that says "encrypted" about its own subject — are out of scope
//! and cannot trip the gate.

use std::{fs, path::Path, path::PathBuf};

use anyhow::{Context, Result};

use crate::{
    cli::{action_from_args, CliAction},
    manager,
    repository::Repository,
    types::{Network, NewNode, NodeType},
};

/// Every operator document that shows or discusses the backup commands. The
/// rendered `neo-nexus --help` is checked alongside them.
const DOCUMENTED_IN: [&str; 4] = [
    "README.md",
    "docs/AGENT_API.md",
    "docs/TROUBLESHOOTING.md",
    "docs/MIGRATION-v4.3.md",
];

/// The only negation accepted for a secrecy claim. "unencrypted" and friends are
/// not negations the gate can trust: "unencrypted copies are never kept" is
/// itself a claim that the copies that are kept are encrypted.
const SECRECY_NEGATION: &str = "NOT encrypted";

/// The only negation accepted for a container claim.
const CONTAINER_NEGATION: &str = "not tar.gz";

/// Words that claim the backup is kept secret, matched case-insensitively in a
/// statement once every accepted negation has been removed from it.
const SECRECY_TERMS: &[Term] = &[
    // encrypt, encrypted, encryption, decrypt, cryptographic, cryptographically
    Term::Substring("crypt"),
    // cipher, ciphered, ciphertext, decipher
    Term::Substring("cipher"),
    // AES, AES-GCM, AES-256
    Term::Word("aes"),
    Term::Word("gcm"),
    Term::Substring("chacha"),
    // seal, sealed, sealing
    Term::Substring("seal"),
    // Only as "-protected": a bare "passphrase" also turns up in lists of what an
    // artifact excludes ("no wallet passwords, passphrases, mnemonics"), and
    // docs/native-rust.md says exactly that about support bundles next to the
    // word "backups".
    Term::Substring("passphrase-protected"),
    Term::Substring("passphrase protected"),
    Term::Substring("password-protected"),
    Term::Substring("password protected"),
];

/// Words that claim a compressed or archived container.
const CONTAINER_TERMS: &[Term] = &[
    Term::Substring("tar.gz"),
    Term::Word("tgz"),
    Term::Substring("tarball"),
    Term::Substring("gzip"),
    Term::Substring("tar archive"),
];

#[derive(Debug, Clone, Copy)]
enum Term {
    /// Anywhere in the statement.
    Substring(&'static str),
    /// Only where no other letter touches it, so "aes" does not fire inside
    /// "caesar" while "AES-256" still counts.
    Word(&'static str),
}

impl Term {
    fn text(self) -> &'static str {
        match self {
            Self::Substring(term) | Self::Word(term) => term,
        }
    }

    fn found_in(self, lower: &str) -> bool {
        match self {
            Self::Substring(term) => lower.contains(term),
            Self::Word(term) => lower.match_indices(term).any(|(start, _)| {
                let before = lower[..start].chars().next_back();
                let after = lower[start + term.len()..].chars().next();
                !before.is_some_and(|letter| letter.is_ascii_alphabetic())
                    && !after.is_some_and(|letter| letter.is_ascii_alphabetic())
            }),
        }
    }
}

/// What the measured export is, reduced to the properties documentation may or
/// may not claim for it.
#[derive(Debug, Clone, Copy)]
struct ExportFacts {
    gzip: bool,
    plaintext_json: bool,
}

struct MeasuredExport {
    file_name: String,
    bytes: Vec<u8>,
    owner_mode: Option<u32>,
}

impl MeasuredExport {
    fn facts(&self) -> ExportFacts {
        let gzip = self.bytes.starts_with(&[0x1f, 0x8b]);
        let plaintext_json = !gzip
            && self.bytes.starts_with(b"{")
            && serde_json::from_slice::<serde_json::Value>(&self.bytes).is_ok();
        ExportFacts {
            gzip,
            plaintext_json,
        }
    }

    /// The one-line description the documentation must carry — derived from the
    /// measured bytes and the measured unix mode, never from a constant, so a
    /// format change forces a documentation change in the same commit.
    fn required_description(&self) -> String {
        let facts = self.facts();
        let container = if facts.gzip {
            "gzip"
        } else if facts.plaintext_json {
            "plaintext JSON"
        } else {
            "binary"
        };
        let secrecy = if facts.plaintext_json {
            SECRECY_NEGATION
        } else {
            "NOT a readable plaintext JSON document"
        };
        format!(
            "{} {container} workspace export, {secrecy}",
            self.mode_claim()
        )
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
fn measure_export() -> Result<MeasuredExport> {
    let temp_dir = tempfile::tempdir().context("temporary workspace for the measured export")?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path).context("open a fresh workspace database")?;
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
        .context("seed one node so the measured export is non-trivial")?;
    drop(repository);

    let output_dir = temp_dir.path().join("backups");
    let db_arg = db_path.display().to_string();
    let out_arg = output_dir.display().to_string();
    let action = action_from_args(["neo-nexus", "--export-backup", &db_arg, &out_arg])
        .context("the documented --export-backup command must succeed")?;
    assert!(
        matches!(action, CliAction::Print(_)),
        "--export-backup reports its summary as printable CLI text"
    );

    let mut written = Vec::new();
    for entry in fs::read_dir(&output_dir).context("the export creates its output directory")? {
        let path = entry.context("read export directory entry")?.path();
        if path.is_file() {
            written.push(path);
        }
    }
    assert_eq!(
        written.len(),
        1,
        "--export-backup must write exactly one backup file, found {written:?}"
    );
    let path = written
        .pop()
        .context("the assertion above proved exactly one file")?;
    let file_name = path
        .file_name()
        .context("the exported backup has a file name")?
        .to_string_lossy()
        .into_owned();
    assert!(
        file_name.ends_with(".json"),
        "the exporter names its output after its real container: {file_name}"
    );

    let bytes = fs::read(&path).context("read the exported backup bytes")?;
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
                .context("stat the exported backup")?
                .permissions()
                .mode()
                & 0o777,
        )
    };
    #[cfg(not(unix))]
    let owner_mode = None;

    Ok(MeasuredExport {
        file_name,
        bytes,
        owner_mode,
    })
}

/// A place an operator reads about the backup: a repository document or the
/// binary's rendered `--help`.
struct Surface {
    name: String,
    text: String,
}

fn surfaces() -> Result<Vec<Surface>> {
    let mut surfaces = Vec::new();
    for relative in DOCUMENTED_IN {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let text = fs::read_to_string(&path)
            .with_context(|| format!("{relative} is part of the repo and must be readable"))?;
        surfaces.push(Surface {
            name: relative.to_string(),
            text,
        });
    }
    surfaces.push(Surface {
        name: "neo-nexus --help".to_string(),
        text: rendered_help()?,
    });
    Ok(surfaces)
}

/// Exactly what the shipped binary prints for `--help`: `main` hands its argv to
/// the manager and writes this output to stdout.
fn rendered_help() -> Result<String> {
    let output = manager::action_from_args(["neo-nexus", "--help"])?
        .into_cli_output()
        .context("--help renders CLI output rather than starting the web workbench")?;
    Ok(output.text_with_trailing_newline())
}

/// One statement of a surface: its lines, trimmed, with runs of whitespace
/// collapsed and the lines joined by single spaces.
#[derive(Debug)]
struct Statement {
    line: usize,
    text: String,
    in_backup_section: bool,
}

impl Statement {
    fn is_about_the_backup(&self) -> bool {
        self.in_backup_section || names_the_backup(&self.text)
    }
}

fn names_the_backup(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("backup") || lower.contains("workspace export")
}

fn heading_names_the_backup(heading: &str) -> bool {
    let lower = heading.to_ascii_lowercase();
    lower.contains("backup") || lower.contains("disaster recovery")
}

fn statements(text: &str) -> Vec<Statement> {
    let mut statements = Vec::new();
    let mut current: Option<Statement> = None;
    let mut in_fence = false;
    // Open markdown headings, outermost first: (level, names the backup).
    let mut sections: Vec<(usize, bool)> = Vec::new();

    for (index, raw) in text.lines().enumerate() {
        let line = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        let number = index + 1;
        if line.is_empty() {
            statements.extend(current.take());
            continue;
        }
        if line.starts_with("```") || line.starts_with("~~~") {
            statements.extend(current.take());
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            if let Some(level) = heading_level(&line) {
                statements.extend(current.take());
                sections.retain(|&(open, _)| open < level);
                sections.push((level, heading_names_the_backup(&line)));
                statements.push(Statement {
                    line: number,
                    text: line,
                    in_backup_section: sections.iter().any(|&(_, backup)| backup),
                });
                continue;
            }
            if opens_a_statement(&line) {
                statements.extend(current.take());
            }
        }
        let table_row = !in_fence && line.starts_with('|');
        match current.as_mut() {
            Some(statement) => {
                statement.text.push(' ');
                statement.text.push_str(&line);
            }
            None => {
                current = Some(Statement {
                    line: number,
                    text: line,
                    in_backup_section: sections.iter().any(|&(_, backup)| backup),
                });
            }
        }
        if table_row {
            statements.extend(current.take());
        }
    }
    statements.extend(current);
    statements
}

fn heading_level(line: &str) -> Option<usize> {
    let level = line
        .chars()
        .take_while(|&character| character == '#')
        .count();
    (level > 0 && level <= 6 && line[level..].starts_with(' ')).then_some(level)
}

/// Lines that begin a new statement even without a blank line before them: list
/// items, table rows, and `--help` option and usage entries.
fn opens_a_statement(line: &str) -> bool {
    let ordered_digits = line.chars().take_while(char::is_ascii_digit).count();
    let ordered_item = ordered_digits > 0
        && (line[ordered_digits..].starts_with(". ") || line[ordered_digits..].starts_with(") "));
    ordered_item
        || line.starts_with("- ")
        || line.starts_with("* ")
        || line.starts_with("+ ")
        || line.starts_with('|')
        || line.starts_with("--")
        || line.starts_with("neo-nexus ")
}

#[derive(Debug)]
struct Finding {
    line: usize,
    claim: &'static str,
    statement: String,
}

/// Every statement about the backup that claims a property the measured export
/// does not have.
fn false_backup_claims(text: &str, facts: ExportFacts) -> Vec<Finding> {
    statements(text)
        .into_iter()
        .filter(Statement::is_about_the_backup)
        .filter_map(|statement| {
            false_claim(&statement.text, facts).map(|claim| Finding {
                line: statement.line,
                claim,
                statement: statement.text,
            })
        })
        .collect()
}

fn false_claim(statement: &str, facts: ExportFacts) -> Option<&'static str> {
    if facts.plaintext_json {
        let rest = statement
            .replace(SECRECY_NEGATION, " ")
            .to_ascii_lowercase();
        if let Some(term) = SECRECY_TERMS.iter().find(|term| term.found_in(&rest)) {
            return Some(term.text());
        }
    } else {
        let lower = statement.to_ascii_lowercase();
        if lower.contains("plaintext json") || lower.contains("not encrypted") {
            return Some("plaintext JSON / NOT encrypted");
        }
    }
    if !facts.gzip {
        let rest = statement
            .replace(CONTAINER_NEGATION, " ")
            .to_ascii_lowercase();
        if let Some(term) = CONTAINER_TERMS.iter().find(|term| term.found_in(&rest)) {
            return Some(term.text());
        }
    }
    None
}

#[test]
fn backup_surfaces_carry_the_measured_export_description() -> Result<()> {
    let measured = measure_export()?;
    let required = measured.required_description();

    for surface in surfaces()? {
        let statements = statements(&surface.text);
        assert!(
            statements
                .iter()
                .any(|statement| statement.text.contains(&required)),
            "{} does not describe the backup export as \"{required}\". \
             --export-backup writes that (file {}). \
             Update the documentation to match, or change the exporter to match the \
             documentation and this test with it.",
            surface.name,
            measured.file_name,
        );

        let shows_export_command = statements
            .iter()
            .any(|statement| statement.text.contains("--export-backup"));
        if shows_export_command {
            assert!(
                statements.iter().any(|statement| {
                    statement.text.contains("--export-backup") && statement.text.contains(&required)
                }),
                "{} shows the --export-backup command but the measured description \
                 \"{required}\" does not appear in the same statement as it. An operator \
                 copying the command must see what the command actually writes.",
                surface.name,
            );
        }
    }
    Ok(())
}

#[test]
fn backup_surfaces_never_claim_properties_the_export_bytes_do_not_have() -> Result<()> {
    let measured = measure_export()?;
    let required = measured.required_description();
    let mut report = Vec::new();

    for surface in surfaces()? {
        for finding in false_backup_claims(&surface.text, measured.facts()) {
            report.push(format!(
                "  {}:{}: \"{}\" in: {}",
                surface.name, finding.line, finding.claim, finding.statement
            ));
        }
    }

    assert!(
        report.is_empty(),
        "--export-backup writes \"{required}\" (file {}), but these statements about the \
         backup claim otherwise. Nobody reading them may believe the backup is encrypted or \
         archived when it is not:\n{}",
        measured.file_name,
        report.join("\n"),
    );
    Ok(())
}

const PLAINTEXT_JSON: ExportFacts = ExportFacts {
    gzip: false,
    plaintext_json: true,
};

fn claimed_lines(text: &str, facts: ExportFacts) -> Vec<usize> {
    false_backup_claims(text, facts)
        .into_iter()
        .map(|finding| finding.line)
        .collect()
}

#[test]
fn a_workspace_export_claim_counts_even_without_the_word_backup() {
    assert_eq!(
        claimed_lines(
            "The workspace export is AES-GCM encrypted at rest.\n",
            PLAINTEXT_JSON
        ),
        vec![1]
    );
}

#[test]
fn a_claim_wrapped_across_two_lines_is_still_one_claim() {
    let wrapped =
        "Every backup the export command writes is\nencrypted with a key the operator holds.\n";
    assert_eq!(claimed_lines(wrapped, PLAINTEXT_JSON), vec![1]);
}

#[test]
fn unencrypted_is_not_accepted_as_a_negation() {
    assert_eq!(
        claimed_lines(
            "Backup files are AES-256 encrypted; unencrypted copies are never kept.\n",
            PLAINTEXT_JSON
        ),
        vec![1]
    );
}

#[test]
fn secrecy_vocabulary_reaches_past_the_word_encrypt() {
    assert_eq!(
        claimed_lines(
            "The backup is cryptographically sealed and ciphered.\n",
            PLAINTEXT_JSON
        ),
        vec![1]
    );
    // Each word on its own is a claim, not only the three together.
    for claim in [
        "The backup is cryptographically protected.",
        "The backup is sealed.",
        "The backup is ciphered.",
        "The backup is AES-256 protected.",
        "The backup uses GCM.",
        "The backup is ChaCha20-Poly1305 protected.",
        "The backup is passphrase-protected.",
        "The backup is password-protected.",
    ] {
        assert_eq!(
            claimed_lines(claim, PLAINTEXT_JSON),
            vec![1],
            "not rejected: {claim}"
        );
    }
}

#[test]
fn a_negation_does_not_excuse_a_positive_claim_in_the_same_statement() {
    assert_eq!(
        claimed_lines(
            "The backup is NOT encrypted on disk, but copies sent off-host are AES encrypted.\n",
            PLAINTEXT_JSON
        ),
        vec![1]
    );
}

#[test]
fn the_explicit_negation_is_accepted() {
    let truthful =
        "# --export-backup writes a 0600 plaintext JSON workspace export, NOT encrypted \
                    (one pretty-printed JSON file, not tar.gz)\n";
    assert!(claimed_lines(truthful, PLAINTEXT_JSON).is_empty());
}

#[test]
fn a_backup_section_scopes_statements_that_do_not_name_the_backup() {
    let recovery = "## Disaster recovery\n\nExports are sealed before they leave the host.\n";
    assert_eq!(claimed_lines(recovery, PLAINTEXT_JSON), vec![3]);

    let unrelated = "## Signer service\n\nExports are sealed before they leave the host.\n";
    assert!(claimed_lines(unrelated, PLAINTEXT_JSON).is_empty());
}

/// Real operator text that says "encrypted" about wallets, sessions and TLS —
/// next to truthful backup statements — must not trip the gate.
#[test]
fn encryption_prose_about_other_subjects_is_out_of_scope() {
    let neighbours = "\
**Signing and key custody.** NeoNexus can load named profiles for all three
backend families in one process: process-local encrypted NEP-6 wallets,
locally deployed signers, and NeoOS signer services. Service profiles keep
admin and least-privilege signing credentials separate and use authenticated
TLS, including for loopback production deployments.

Browser-based requests use encrypted, short-lived HTTP session cookies:

OPTIONS:
  --export-backup              Write a 0600 plaintext JSON workspace export, NOT encrypted
  --import-backup              Import a previously exported workspace backup file
  --validate-wallet            Validate an encrypted NEP-6 Neo wallet file
  --validate-wallet-json       Print encrypted NEP-6 wallet validation as JSON
";
    assert!(
        claimed_lines(neighbours, PLAINTEXT_JSON).is_empty(),
        "{:?}",
        false_backup_claims(neighbours, PLAINTEXT_JSON)
    );
}

#[test]
fn a_container_claim_needs_the_measured_container() {
    assert_eq!(
        claimed_lines("The backup is a tar.gz archive.\n", PLAINTEXT_JSON),
        vec![1]
    );
    assert!(claimed_lines(
        "The backup is one pretty-printed JSON file, not tar.gz.\n",
        PLAINTEXT_JSON
    )
    .is_empty());
}

#[test]
fn plaintext_wording_goes_stale_once_the_export_stops_being_plaintext() {
    let sealed_export = ExportFacts {
        gzip: false,
        plaintext_json: false,
    };
    assert_eq!(
        claimed_lines(
            "The backup is a 0600 plaintext JSON workspace export, NOT encrypted.\n",
            sealed_export
        ),
        vec![1]
    );
}
