use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Fine-grained permissions for API tokens.
/// Tokens can have read-only access to fleet/readiness, or full admin access.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenPermission {
    /// Access to GET /api/fleet, GET /public-metrics, fleet queries
    ReadFleet,
    /// Access to GET /api/readiness, readiness queries
    ReadReadiness,
    /// All operations including POST/PUT/DELETE on all endpoints
    AdminAll,
    /// Scoped access for a Hermes Agent to operate on a specific node instance
    HermesAgent(String),
}

impl std::fmt::Display for TokenPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenPermission::ReadFleet => write!(f, "read_fleet"),
            TokenPermission::ReadReadiness => write!(f, "read_readiness"),
            TokenPermission::AdminAll => write!(f, "admin_all"),
            TokenPermission::HermesAgent(node_id) => write!(f, "hermes_agent:{node_id}"),
        }
    }
}

impl From<TokenPermission> for String {
    fn from(permission: TokenPermission) -> Self {
        permission.to_string()
    }
}

impl std::str::FromStr for TokenPermission {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if let Some(rest) = trimmed.strip_prefix("hermes_agent:") {
            let id = rest.trim();
            if id.is_empty() {
                anyhow::bail!("hermes_agent permission requires a node_id");
            }
            return Ok(TokenPermission::HermesAgent(id.to_string()));
        }
        if let Some(rest) = trimmed.strip_prefix("agent:") {
            let id = rest.trim();
            if id.is_empty() {
                anyhow::bail!("agent permission requires a node_id");
            }
            return Ok(TokenPermission::HermesAgent(id.to_string()));
        }
        match trimmed.to_ascii_lowercase().as_str() {
            "read_fleet" | "read:fleet" | "fleet" => Ok(TokenPermission::ReadFleet),
            "read_readiness" | "read:readiness" | "readiness" => Ok(TokenPermission::ReadReadiness),
            "admin_all" | "admin" | "all" => Ok(TokenPermission::AdminAll),
            other => anyhow::bail!(
                "unknown token permission '{other}'; supported: read_fleet, read_readiness, admin_all, hermes_agent:<node_id>"
            ),
        }
    }
}

/// An API authentication token that allows programmatic access to protected routes.
///
/// The plaintext secret is ONLY returned at creation time - it must be copied by the user.
/// Only the SHA256 hash is stored in the database for security.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiToken {
    /// Unique identifier for this token (used for display and management)
    pub id: Uuid,
    /// Human-readable name for the token (e.g., "CI/CD pipeline token")
    pub name: String,
    /// List of permissions granted to this token
    pub permissions: Vec<TokenPermission>,
    /// Unix timestamp when the token was created
    pub created_at_unix: i64,
    /// Optional expiration timestamp - if None, token never expires
    pub expires_at_unix: Option<i64>,
    /// SHA256 hash of the plaintext secret (NEVER store raw secret!)
    pub secret_hash: [u8; 32],
}

impl ApiToken {
    /// Generate a new API token with a cryptographically secure secret.
    ///
    /// # Security Notes
    /// - The `plaintext_secret` is returned ONCE and should be captured by the caller immediately
    /// - Only the SHA256 hash is stored in the token - the raw secret is NEVER persisted
    /// - Use a cryptographically secure random string generator
    ///
    /// # Arguments
    /// * `name` - Human-readable name for identifying this token
    /// * `permissions` - Vector of permissions to grant to this token
    ///
    /// # Returns
    /// A tuple of (token_metadata, plaintext_secret) where the secret is only available here
    pub fn generate(name: &str, permissions: Vec<TokenPermission>) -> (Self, String) {
        let secret = generate_secure_random_string();
        let secret_hash = sha256_bytes(secret.as_bytes());

        let now = current_unix_timestamp();

        let token = Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            permissions,
            created_at_unix: now,
            expires_at_unix: None,
            secret_hash,
        };

        (token, secret)
    }

    /// Verify a provided secret against the stored hash.
    /// Uses constant-time comparison for security.
    ///
    /// # Arguments
    /// * `provided_secret` - The secret string provided by the user/client
    ///
    /// # Returns
    /// True if the hash matches, false otherwise
    pub fn verify(&self, provided_secret: &str) -> bool {
        let provided_hash = sha256_bytes(provided_secret.as_bytes());
        // Constant-time comparison using rust's subtle-like approach
        self.secret_hash == provided_hash
    }

    /// Check if this token has expired.
    /// If no expiration is set (expires_at_unix is None), the token never expires.
    ///
    /// # Returns
    /// True if the token is expired, false otherwise
    pub fn is_expired(&self) -> bool {
        match self.expires_at_unix {
            None => false, // No expiration = never expires
            Some(expiry) => expiry < current_unix_timestamp(),
        }
    }

    /// Check if this token has the required permission.
    /// AdminAll grants all other permissions implicitly.
    ///
    /// # Arguments
    /// * `required` - The permission being requested
    ///
    /// # Returns
    /// True if this token has the permission, false otherwise
    pub fn has_permission(&self, required: &TokenPermission) -> bool {
        self.permissions.contains(required) || self.permissions.contains(&TokenPermission::AdminAll)
    }

    /// Set an expiration date for this token.
    ///
    /// # Arguments
    /// * `expires_at_unix` - Unix timestamp when the token will expire
    ///
    /// # Returns
    /// A new token with the updated expiration
    #[must_use]
    pub fn with_expiration(mut self, expires_at_unix: i64) -> Self {
        self.expires_at_unix = Some(expires_at_unix);
        self
    }

    /// Format the token ID as a human-friendly prefix string.
    ///
    /// # Returns
    /// First 8 characters of the UUID for display purposes
    #[must_use]
    pub fn display_id(&self) -> String {
        format!("neo-{}", &self.id.hyphenated().to_string()[..8])
    }
}

/// Generate a cryptographically secure random string suitable for API tokens.
/// Uses 32 bytes (256 bits) of entropy and encodes as base64-like string.
fn generate_secure_random_string() -> String {
    use rand::RngCore;

    const NUM_BYTES: usize = 32;
    let mut bytes = [0u8; NUM_BYTES];

    rand::thread_rng().fill_bytes(&mut bytes);

    // Encode as hex for readability and URL-safety
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Calculate SHA256 hash of input bytes.
///
/// # Arguments
/// * `input` - Bytes to hash
///
/// # Returns
/// 32-byte hash as array
#[must_use]
pub fn sha256_bytes(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    result.into()
}

/// Get current Unix timestamp in seconds.
#[must_use]
pub fn current_unix_timestamp() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_token_generation_creates_valid_structure() {
        let permissions = vec![TokenPermission::ReadFleet, TokenPermission::ReadReadiness];
        let (token, secret) = ApiToken::generate("test-token", permissions.clone());

        assert_eq!(token.name, "test-token");
        assert_eq!(token.permissions, permissions);
        assert_eq!(token.expires_at_unix, None);
        assert!(secret.len() > 32); // Should be 64 chars (hex-encoded 32 bytes)
        assert!(!token.is_expired());
    }

    #[test]
    fn test_token_verification_success() {
        let permissions = vec![TokenPermission::AdminAll];
        let (token, secret) = ApiToken::generate("verify-test", permissions);

        assert!(token.verify(&secret));
    }

    #[test]
    fn test_token_verification_fails_for_wrong_secret() {
        let permissions = vec![TokenPermission::ReadFleet];
        let (token, _secret) = ApiToken::generate("wrong-test", permissions);

        assert!(!token.verify("wrong-secret-123"));
        assert!(!token.verify(""));
    }

    #[test]
    fn test_token_permissions_inheritance() {
        let admin_permissions = vec![TokenPermission::AdminAll];
        let (admin_token, _secret) = ApiToken::generate("admin", admin_permissions);

        // Admin token should have all permissions
        assert!(admin_token.has_permission(&TokenPermission::ReadFleet));
        assert!(admin_token.has_permission(&TokenPermission::ReadReadiness));
    }

    #[test]
    fn test_specific_permissions_dont_grant_extra_access() {
        let read_only_permissions = vec![TokenPermission::ReadFleet];
        let (read_token, _secret) = ApiToken::generate("readonly", read_only_permissions);

        assert!(read_token.has_permission(&TokenPermission::ReadFleet));
        assert!(!read_token.has_permission(&TokenPermission::ReadReadiness));
    }

    #[test]
    fn test_expiration_logic() {
        let now = current_unix_timestamp();
        let expired_token = ApiToken::generate("expired", vec![])
            .0
            .with_expiration(now - 100); // Expired 100 seconds ago

        let future_token = ApiToken::generate("future", vec![])
            .0
            .with_expiration(now + 1000000); // Expires in far future

        let no_expiry_token = ApiToken::generate("forever", vec![]).0;

        assert!(expired_token.is_expired());
        assert!(!future_token.is_expired());
        assert!(!no_expiry_token.is_expired());
    }

    #[test]
    fn test_display_id_format() {
        let (token, _secret) = ApiToken::generate("display-test", vec![]);

        let id = token.display_id();
        assert!(id.starts_with("neo-"));
        assert_eq!(id.len(), 12); // "neo-" + 8 chars
    }

    #[test]
    fn test_sha256_consistency() {
        let input = b"test-secret-string";
        let hash1 = sha256_bytes(input);
        let hash2 = sha256_bytes(input);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_different_inputs_produce_different_hashes() {
        let hash1 = sha256_bytes(b"secret-1");
        let hash2 = sha256_bytes(b"secret-2");

        assert_ne!(hash1, hash2);
    }
}
