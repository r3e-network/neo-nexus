//! Repository methods for API token management.
//!
//! Provides CRUD operations for API authentication tokens that allow programmatic
//! access to protected web routes via Bearer token headers.

use super::*;
use crate::wallet::{sha256_bytes, ApiToken, TokenPermission};

impl Repository {
    /// List all API tokens ordered by creation date (newest first).
    ///
    /// # Returns
    /// A vector of API tokens sorted by created_at_unix descending
    pub fn list_api_tokens(&self) -> Result<Vec<ApiToken>> {
        let connection = self.connection()?;

        let mut statement = connection.prepare(
            "SELECT id, name, permissions, created_at_unix, expires_at_unix, secret_hash
             FROM api_tokens
             ORDER BY created_at_unix DESC",
        )?;

        let tokens = statement
            .query_map([], row_to_api_token)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read API tokens from database")?;

        Ok(tokens)
    }

    /// Create a new API token and store it in the database.
    ///
    /// This is the ONLY time the plaintext secret is returned - after this, only
    /// the hash is stored. The token metadata is persisted to the database.
    ///
    /// # Arguments
    /// * `name` - Human-readable name for this token
    /// * `permissions` - Vector of permissions to grant
    /// * `expires_at_unix` - Optional expiration timestamp
    ///
    /// # Returns
    /// A tuple containing the stored token and its plaintext secret
    pub fn create_api_token(
        &self,
        name: &str,
        permissions: Vec<TokenPermission>,
        expires_at_unix: Option<i64>,
    ) -> Result<(ApiToken, String)> {
        let (mut token, plaintext_secret) = ApiToken::generate(name, permissions.clone());

        // Set expiration if provided
        if let Some(expiry) = expires_at_unix {
            token.expires_at_unix = Some(expiry);
        }

        let permissions_csv = permissions
            .into_iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let connection = self.connection()?;

        connection.execute(
            "INSERT INTO api_tokens (id, name, permissions, created_at_unix, expires_at_unix, secret_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                token.id.to_string(),
                token.name,
                permissions_csv,
                token.created_at_unix,
                token.expires_at_unix,
                token.secret_hash,
            ],
        )?;

        Ok((token, plaintext_secret))
    }

    /// Delete an API token by ID.
    ///
    /// # Arguments
    /// * `token_id` - UUID string of the token to delete
    ///
    /// # Returns
    /// Number of rows deleted (0 or 1)
    pub fn delete_api_token(&self, token_id: &str) -> Result<usize> {
        let connection = self.connection()?;

        let id = Uuid::parse_str(token_id)
            .with_context(|| format!("invalid UUID for API token: {token_id}"))?;

        let deleted = connection.execute(
            "DELETE FROM api_tokens WHERE id = ?1",
            params![id.to_string()],
        )?;

        Ok(deleted)
    }

    /// Verify an API token secret against stored hashes.
    ///
    /// This method hashes the provided secret and compares it to stored hashes.
    /// It also checks if the token has expired before returning it.
    ///
    /// # Arguments
    /// * `provided_secret` - The plaintext secret to verify
    ///
    /// # Returns
    /// Some(ApiToken) if verification succeeds and token is not expired,
    /// None if no match found or token is expired
    pub fn verify_token_secret(&self, provided_secret: &str) -> Result<Option<ApiToken>> {
        let provided_hash = sha256_bytes(provided_secret.as_bytes());

        let connection = self.connection()?;

        let maybe_token = connection.query_row(
            "SELECT id, name, permissions, created_at_unix, expires_at_unix, secret_hash
             FROM api_tokens
             WHERE secret_hash = ?1",
            params![&provided_hash[..]],
            row_to_api_token,
        );

        match maybe_token {
            Ok(token) if token.is_expired() => Ok(None), // Expired tokens are rejected
            Ok(token) => Ok(Some(token)),                // Valid token
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None), // No matching token
            Err(e) => Err(e).context("failed to verify API token"),
        }
    }
}

/// Convert a database row to an ApiToken, strictly validating the UUID and secret hash length.
fn row_to_api_token(row: &rusqlite::Row<'_>) -> rusqlite::Result<ApiToken> {
    let id_str: String = row.get(0)?;
    let id = id_str.parse().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let name: String = row.get(1)?;
    let permissions_csv: String = row.get(2)?;
    let created_at_unix: i64 = row.get(3)?;
    let expires_at_unix: Option<i64> = row.get(4)?;
    let hash_bytes: Vec<u8> = row.get(5)?;
    let secret_hash: [u8; 32] = hash_bytes.try_into().map_err(|bytes: Vec<u8>| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Blob,
            format!("expected 32-byte secret hash, got {} bytes", bytes.len()).into(),
        )
    })?;

    Ok(ApiToken {
        id,
        name,
        permissions: parse_permissions(permissions_csv),
        created_at_unix,
        expires_at_unix,
        secret_hash,
    })
}

fn parse_permissions(permissions_csv: String) -> Vec<TokenPermission> {
    if permissions_csv.is_empty() {
        return vec![];
    }

    permissions_csv
        .split(',')
        .filter_map(|perm| perm.trim().parse::<TokenPermission>().ok())
        .collect()
}
