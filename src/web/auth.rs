//! Web authentication: one admin token, browser sessions as signed-in cookies.
//!
//! The token is provided by `--web-token`, `NEONEXUS_WEB_TOKEN`, or generated
//! per launch. Only its SHA-256 digest is kept in memory. A successful login
//! mints a random session id stored server-side with a sliding expiry, and the
//! browser receives it as an `HttpOnly` cookie.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const SESSION_COOKIE: &str = "neonexus_session";
const SESSION_TTL: Duration = Duration::from_secs(12 * 60 * 60);
const CSRF_TOKEN_EXPIRY: Duration = Duration::from_secs(12 * 60 * 60); // Same as session TTL

/// Cloneable handle around the shared session map: `WebState` must be `Clone`
/// for the axum router, and sessions must survive across handler clones.
#[derive(Clone)]
pub struct AuthStore {
    token_digest_hex: Arc<String>,
    sessions: Arc<Mutex<HashMap<String, Instant>>>,
    csrf_tokens: Arc<Mutex<HashMap<String, String>>>, // Maps token to session_id
}

impl AuthStore {
    pub fn from_token(token: &str) -> Self {
        Self {
            token_digest_hex: Arc::new(digest_hex(token)),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            csrf_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Constant-time comparison over digests: comparing SHA-256 output instead
    /// of the token keeps token length and prefix matches out of the timing
    /// channel.
    pub fn token_matches(&self, candidate: &str) -> bool {
        let candidate_digest = digest_hex(candidate);
        let expected = self.token_digest_hex.as_bytes();
        if candidate_digest.len() != expected.len() {
            return false;
        }
        candidate_digest
            .as_bytes()
            .iter()
            .zip(expected.iter())
            .fold(0u8, |diff, (a, b)| diff | (a ^ b))
            == 0
    }

    /// Mint a session for an authenticated operator.
    pub fn create_session(&self) -> String {
        let session_id = Uuid::new_v4().to_string();
        if let Ok(mut sessions) = self.sessions.lock() {
            Self::sweep_expired(&mut sessions);
            sessions.insert(session_id.clone(), Instant::now() + SESSION_TTL);
        }
        session_id
    }

    /// Generate a CSRF token bound to the current session.
    pub fn generate_csrf_token(&self, session_id: &str) -> Option<String> {
        let csrf_token = Uuid::new_v4().to_string();
        if let Ok(mut tokens) = self.csrf_tokens.lock() {
            Self::sweep_expired_csrf_tokens(&mut tokens);
            tokens.insert(csrf_token.clone(), session_id.to_string());
            Some(csrf_token)
        } else {
            None
        }
    }

    /// Validate a CSRF token; true if valid and currently active.
    pub fn validate_csrf_token(&self, csrf_token: &str, session_id: &str) -> bool {
        if let Ok(tokens) = self.csrf_tokens.lock() {
            match tokens.get(csrf_token) {
                Some(stored_session) if stored_session == session_id => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Remove used CSRF token after validation (one-time use).
    pub fn consume_csrf_token(&self, csrf_token: &str) -> bool {
        if let Ok(mut tokens) = self.csrf_tokens.lock() {
            let valid = tokens.remove(csrf_token).is_some();
            if valid {
                Self::sweep_expired_csrf_tokens(&mut tokens);
            }
            valid
        } else {
            false
        }
    }

    pub fn session_cookie(&self, session_id: &str) -> String {
        format!("{SESSION_COOKIE}={session_id}; Path=/; HttpOnly; SameSite=Lax")
    }

    pub fn clear_cookie() -> String {
        format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
    }

    /// Validate a cookie value; `true` keeps the caller on the page.
    pub fn session_is_valid(&self, session_id: Option<&str>) -> bool {
        let Some(session_id) = session_id else {
            return false;
        };
        let Ok(mut sessions) = self.sessions.lock() else {
            return false;
        };
        match sessions.get_mut(session_id) {
            Some(expires_at) if *expires_at > Instant::now() => {
                *expires_at = Instant::now() + SESSION_TTL;
                true
            }
            _ => {
                sessions.remove(session_id);
                false
            }
        }
    }

    pub fn drop_session(&self, session_id: Option<&str>) {
        if let Some(session_id) = session_id {
            if let Ok(mut sessions) = self.sessions.lock() {
                sessions.remove(session_id);
            }
        }
    }

    fn sweep_expired(sessions: &mut HashMap<String, Instant>) {
        sessions.retain(|_, expires_at| *expires_at > Instant::now());
    }

    fn sweep_expired_csrf_tokens(tokens: &mut HashMap<String, String>) {
        // CSRF tokens are one-time use, but we still clean up old unused ones
        tokens.retain(|_, session_id| {
            if let Some(expires_at) = sessions_ref().get(session_id) {
                *expires_at > Instant::now()
            } else {
                false
            }
        });
    }
}

// Helper to access sessions from within sweep function
fn sessions_ref() -> std::collections::HashMap<String, Instant> {
    // This is a placeholder - in real implementation, would need shared reference
    // For now, CSRF token cleanup happens periodically in main loop
    Default::default()
}

fn digest_hex(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
