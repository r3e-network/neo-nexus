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

        let tokens = statement.query_map([], |row| {
            Ok(ApiToken {
                id: row.get::<_, String>(0)?.parse().unwrap_or_default(),
                name: row.get(1)?,
                permissions: parse_permissions(row.get::<_, String>(2)?),
                created_at_unix: row.get(3)?,
                expires_at_unix: row.get(4)?,
                secret_hash: row
                    .get::<_, Vec<u8>>(5)?
                    .try_into()
                    .unwrap_or_else(|_| [0u8; 32]),
            })
        });

        let mut results = Vec::new();
        for token_result in tokens? {
            if let Ok(token) = token_result {
                results.push(token);
            }
        }

        Ok(results)
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
            |row| {
                Ok(ApiToken {
                    id: row.get::<_, String>(0)?.parse().unwrap_or_default(),
                    name: row.get(1)?,
                    permissions: parse_permissions(row.get::<_, String>(2)?),
                    created_at_unix: row.get(3)?,
                    expires_at_unix: row.get(4)?,
                    secret_hash: row
                        .get::<_, Vec<u8>>(5)?
                        .try_into()
                        .unwrap_or_else(|_| [0u8; 32]),
                })
            },
        );

        match maybe_token {
            Ok(token) if token.is_expired() => Ok(None), // Expired tokens are rejected
            Ok(token) => Ok(Some(token)),                // Valid token
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None), // No matching token
            Err(e) => Err(e).context("failed to verify API token"),
        }
    }
}

/// Parse a CSV string of permission names into a vector of TokenPermission enum variants.
fn parse_permissions(permissions_csv: String) -> Vec<TokenPermission> {
    if permissions_csv.is_empty() {
        return vec![];
    }

    permissions_csv
        .split(',')
        .filter_map(|perm| match perm.trim() {
            "read_fleet" => Some(TokenPermission::ReadFleet),
            "read_readiness" => Some(TokenPermission::ReadReadiness),
            "admin_all" => Some(TokenPermission::AdminAll),
            _ => None, // Ignore unknown permissions
        })
        .collect()
}
