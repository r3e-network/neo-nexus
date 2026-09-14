use super::*;

/// The expectation is read from a workspace this build creates, so it covers
/// **everything** the schema builder makes. The hand-maintained list it
/// replaces had already drifted: it omitted `node_signer_bindings`, whose
/// unique index is the constraint that stops two nodes signing with one key,
/// along with `node_hermes_agents`, `node_runtime_quarantine` and `api_tokens`.
/// A workspace missing any of them passed.
#[test]
fn the_reference_covers_every_table_the_schema_builder_creates() {
    let reference = ReferenceSchema::build().expect("the reference schema builds");

    for table in [
        "nodes",
        "node_roles",
        "node_wallets",
        "node_signer_bindings",
        "node_hermes_agents",
        "node_runtime_quarantine",
        "node_restart_holds",
        "node_samples",
        "node_health_state",
        "node_health_transitions",
        "api_tokens",
        "plugin_states",
        "runtime_events",
        "remote_servers",
    ] {
        assert!(
            reference.tables.contains_key(table),
            "{table} is missing from the reference: {:?}",
            reference.tables.keys().collect::<Vec<_>>()
        );
    }
}

/// Columns come with the tables, so a column added to a creation statement is
/// expected of a real workspace from the same commit — no second edit.
#[test]
fn the_reference_carries_each_tables_columns() {
    let reference = ReferenceSchema::build().expect("the reference schema builds");
    let nodes = reference
        .tables
        .get("nodes")
        .expect("the nodes table is in the reference");
    for column in ["id", "name", "node_type", "network", "status", "pid"] {
        assert!(nodes.contains(column), "nodes.{column} is not expected");
    }
}

/// SQLite invents `sqlite_autoindex_*` for primary keys and unique constraints.
/// Those follow from the column definitions, so requiring them by name would be
/// asserting on an implementation detail rather than on a migration anyone can
/// forget to run.
#[test]
fn only_named_indexes_are_expected() {
    let reference = ReferenceSchema::build().expect("the reference schema builds");
    for (table, indexes) in &reference.indexes {
        for index in indexes {
            assert!(
                !index.starts_with("sqlite_autoindex_"),
                "{table} expects an implicit index by name: {index}"
            );
        }
    }
    assert!(
        reference.indexes.contains_key("node_samples"),
        "the sample index the derivations walk is expected"
    );
}

/// A workspace this build just created must, by construction, satisfy the
/// expectation this build derives. If it does not, the reference is being read
/// differently from the way the live database is.
#[test]
fn a_fresh_workspace_satisfies_its_own_reference() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let path = dir.path().join("fresh.db");
    drop(crate::repository::Repository::open(&path).expect("a fresh workspace"));

    let report = crate::workspace_integrity::WorkspaceIntegrityChecker::check(&path, "test")
        .expect("the check runs");
    let unsatisfied: Vec<&str> = report
        .required_tables
        .iter()
        .filter(|check| !check.present || !check.missing_columns.is_empty())
        .map(|check| check.table.as_str())
        .collect();
    assert!(
        unsatisfied.is_empty(),
        "a workspace this build created does not satisfy this build's expectation: {unsatisfied:?}"
    );
    assert!(
        report.required_indexes.iter().all(|check| check.present),
        "an index the schema builder creates is reported missing"
    );
}
