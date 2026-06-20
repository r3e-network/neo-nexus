use super::*;

#[test]
fn repository_persists_runtime_installations() {
    let repo = create_repo();
    let installation = runtime_install(
        "neo-rs-local",
        NodeType::NeoRs,
        "v2.0.0",
        RuntimePlatform::current(),
        88,
    );

    repo.upsert_runtime_installation(&installation).unwrap();
    let persisted = repo.list_runtime_installations().unwrap();

    assert_eq!(persisted, vec![installation]);
}
