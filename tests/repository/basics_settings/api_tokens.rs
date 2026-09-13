use super::*;
use neo_nexus::wallet::TokenPermission;

#[test]
fn create_and_list_api_tokens() {
    let home = tempfile::tempdir().expect("temporary workspace");
    let database = home.path().join("neonexus.db");
    let repo = Repository::open(&database).expect("workspace opens");

    let (token1, secret1) = repo
        .create_api_token("metrics-collector", vec![TokenPermission::ReadFleet], None)
        .expect("token 1 created");
    assert!(!secret1.is_empty());
    assert_eq!(token1.name, "metrics-collector");
    assert_eq!(token1.permissions, vec![TokenPermission::ReadFleet]);

    let (token2, secret2) = repo
        .create_api_token(
            "ci-admin",
            vec![TokenPermission::AdminAll, TokenPermission::ReadReadiness],
            Some(i64::MAX),
        )
        .expect("token 2 created");
    assert!(!secret2.is_empty());
    assert_eq!(token2.name, "ci-admin");

    let list = repo.list_api_tokens().expect("tokens listed");
    assert_eq!(list.len(), 2);
    assert!(list.iter().any(|t| t.id == token1.id));
    assert!(list.iter().any(|t| t.id == token2.id));
}

#[test]
fn verify_token_secret_and_expiration() {
    let home = tempfile::tempdir().expect("temporary workspace");
    let database = home.path().join("neonexus.db");
    let repo = Repository::open(&database).expect("workspace opens");

    let (token, secret) = repo
        .create_api_token("active-token", vec![TokenPermission::ReadFleet], None)
        .expect("token created");

    let verified = repo.verify_token_secret(&secret).expect("verify succeeds");
    assert!(verified.is_some());
    assert_eq!(verified.unwrap().id, token.id);

    let wrong = repo
        .verify_token_secret("totally-invalid-secret")
        .expect("verify succeeds");
    assert!(wrong.is_none());

    // Expired token (in the past: unix timestamp 1000)
    let (_expired_token, expired_secret) = repo
        .create_api_token(
            "expired-token",
            vec![TokenPermission::ReadFleet],
            Some(1000),
        )
        .expect("expired token created");

    let result = repo
        .verify_token_secret(&expired_secret)
        .expect("verify runs");
    assert!(result.is_none(), "expired tokens must be rejected");
}

#[test]
fn delete_api_token() {
    let home = tempfile::tempdir().expect("temporary workspace");
    let database = home.path().join("neonexus.db");
    let repo = Repository::open(&database).expect("workspace opens");

    let (token, _secret) = repo
        .create_api_token("temp-token", vec![TokenPermission::ReadFleet], None)
        .expect("token created");

    let deleted = repo
        .delete_api_token(&token.id.to_string())
        .expect("delete succeeds");
    assert_eq!(deleted, 1);

    let deleted_again = repo
        .delete_api_token(&token.id.to_string())
        .expect("delete succeeds");
    assert_eq!(deleted_again, 0);

    let list = repo.list_api_tokens().expect("list succeeds");
    assert!(list.is_empty());
}

#[test]
fn corrupted_token_records_fail_explicitly() {
    let home = tempfile::tempdir().expect("temporary workspace");
    let database = home.path().join("neonexus.db");
    let repo = Repository::open(&database).expect("workspace opens");

    let conn = rusqlite::Connection::open(&database).expect("sqlite opens");

    // Corrupt UUID: ensure it does NOT silently default to Uuid::nil()
    conn.execute(
        "INSERT INTO api_tokens (id, name, permissions, created_at_unix, expires_at_unix, secret_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            "not-a-valid-uuid",
            "corrupt-uuid",
            "read_fleet",
            100,
            None::<i64>,
            vec![0u8; 32],
        ],
    )
    .expect("corrupt row inserted");

    let err = repo.list_api_tokens();
    assert!(
        err.is_err(),
        "malformed UUID must return explicit error, not default"
    );
}
