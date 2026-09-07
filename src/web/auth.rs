//! Web authentication and deployment boundaries.
//!
//! One strong operator token mints short-lived, server-side browser sessions.
//! The token itself comes from a protected file or an interactive bootstrap;
//! browser mutations are admitted only from the configured public origin.

use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use axum::http::{header, HeaderMap};
use sha2::{Digest, Sha256};
use url::{Host, Position, Url};
use uuid::Uuid;

pub const SESSION_COOKIE: &str = "neonexus_session";
pub const TOKEN_FILE_ENV: &str = "NEONEXUS_WEB_TOKEN_FILE";
pub const PUBLIC_ORIGIN_ENV: &str = "NEONEXUS_WEB_PUBLIC_ORIGIN";
pub const LEGACY_TOKEN_ENV: &str = "NEONEXUS_WEB_TOKEN";
pub const MIN_OPERATOR_TOKEN_BYTES: usize = 32;
pub const METRICS_TOKEN_ENV: &str = "NEONEXUS_METRICS_TOKEN";

const MAX_OPERATOR_TOKEN_BYTES: usize = 4 * 1024;
const SESSION_TTL: Duration = Duration::from_secs(12 * 60 * 60);
const LOGIN_FAILURES_BEFORE_BACKOFF: u32 = 5;
const LOGIN_BACKOFF_BASE: Duration = Duration::from_secs(5);
const LOGIN_BACKOFF_MAX: Duration = Duration::from_secs(5 * 60);
const LOGIN_ATTEMPT_RETENTION: Duration = Duration::from_secs(15 * 60);
const MAX_LOGIN_PEERS: usize = 1_024;

/// Deployment facts that must be explicit before a router can be served.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSecurity {
    public_origin: Option<String>,
    secure_cookies: bool,
}

impl WebSecurity {
    /// Validate the browser-facing origin against the interface being exposed.
    ///
    /// A loopback listener may omit it for local development. Every non-loopback
    /// listener needs an HTTPS origin, and an explicit HTTP origin is accepted
    /// only when its host is itself loopback.
    pub fn resolve(bind: IpAddr, requested_origin: Option<&str>) -> Result<Self> {
        let public_origin = match requested_origin {
            Some(value) if value.trim().is_empty() => {
                bail!("the web public origin cannot be blank")
            }
            Some(value) => Some(validated_origin(value)?),
            None => None,
        };

        if !bind.is_loopback() && public_origin.is_none() {
            bail!(
                "a non-loopback web bind requires --web-public-origin or {PUBLIC_ORIGIN_ENV}, and it must use HTTPS"
            );
        }

        if let Some(origin) = public_origin.as_deref() {
            let parsed = Url::parse(origin).context("validated public origin became unreadable")?;
            if parsed.scheme() == "http" && (!bind.is_loopback() || !is_loopback_host(&parsed)) {
                bail!("the web public origin must use HTTPS unless both listener and origin are loopback");
            }
        }

        let secure_cookies = public_origin
            .as_deref()
            .is_some_and(|origin| origin.starts_with("https://"));
        Ok(Self {
            public_origin,
            secure_cookies,
        })
    }

    /// Explicit local-development posture for callers that build the router
    /// themselves, including integration tests.
    pub fn loopback_http() -> Self {
        Self {
            public_origin: None,
            secure_cookies: false,
        }
    }

    pub fn public_origin(&self) -> Option<&str> {
        self.public_origin.as_deref()
    }

    pub fn secure_cookies(&self) -> bool {
        self.secure_cookies
    }

    /// Check the browser provenance of a state-changing request.
    ///
    /// `Origin` is authoritative when present. `Referer` is a compatibility
    /// fallback only when `Origin` is absent; a bad Origin cannot be rescued by a
    /// good Referer. Local development derives its expected origin from a
    /// loopback-only Host header.
    pub fn allows_unsafe_request(&self, headers: &HeaderMap) -> bool {
        let Some(expected) = self
            .public_origin
            .clone()
            .or_else(|| loopback_origin_from_host(headers))
        else {
            return false;
        };

        if headers.contains_key(header::ORIGIN) {
            return single_header_text(headers, &header::ORIGIN).is_some_and(|origin| {
                validated_origin(origin).is_ok_and(|origin| origin == expected)
            });
        }
        single_header_text(headers, &header::REFERER)
            .and_then(referer_origin)
            .is_some_and(|origin| origin == expected)
    }
}

/// Reject credentials that are short enough to be practical online guesses.
pub fn validated_operator_token(token: &str) -> Result<&str> {
    let token = token.trim();
    if token.len() < MIN_OPERATOR_TOKEN_BYTES {
        bail!(
            "the operator token must contain at least {MIN_OPERATOR_TOKEN_BYTES} UTF-8 bytes; generate it with a cryptographic secret tool"
        );
    }
    if token.len() > MAX_OPERATOR_TOKEN_BYTES {
        bail!("the operator token exceeds the {MAX_OPERATOR_TOKEN_BYTES}-byte limit");
    }
    Ok(token)
}

/// Result of one peer's attempt to exchange the operator token for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginDecision {
    Accepted,
    Rejected,
    Throttled { retry_after_seconds: u64 },
}

/// Cloneable handle around shared sessions and a bounded login-attempt table.
#[derive(Clone)]
pub struct AuthStore {
    token_digest_hex: Arc<String>,
    sessions: Arc<Mutex<HashMap<String, Instant>>>,
    login_attempts: Arc<Mutex<LoginAttempts>>,
}

impl AuthStore {
    pub fn from_token(token: &str) -> Result<Self> {
        let token = validated_operator_token(token)?;
        Ok(Self {
            token_digest_hex: Arc::new(digest_hex(token)),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            login_attempts: Arc::new(Mutex::new(LoginAttempts::default())),
        })
    }

    /// Authenticate and update the peer's limiter as one serialized operation.
    pub fn authenticate(&self, peer: IpAddr, candidate: &str) -> LoginDecision {
        self.authenticate_at(peer, candidate, Instant::now())
    }

    fn authenticate_at(&self, peer: IpAddr, candidate: &str, now: Instant) -> LoginDecision {
        let Ok(mut attempts) = self.login_attempts.lock() else {
            return LoginDecision::Throttled {
                retry_after_seconds: LOGIN_BACKOFF_MAX.as_secs(),
            };
        };
        attempts.sweep(now);

        if let Some(blocked_until) = attempts
            .peers
            .get(&peer)
            .and_then(|entry| entry.blocked_until)
            .filter(|blocked_until| *blocked_until > now)
        {
            return LoginDecision::Throttled {
                retry_after_seconds: retry_after_seconds(blocked_until.duration_since(now)),
            };
        }

        if self.token_matches(candidate.trim()) {
            attempts.peers.remove(&peer);
            return LoginDecision::Accepted;
        }

        attempts.make_room_for(peer);
        let entry = attempts.peers.entry(peer).or_insert(LoginAttempt {
            failures: 0,
            blocked_until: None,
            last_seen: now,
        });
        entry.failures = entry.failures.saturating_add(1);
        entry.last_seen = now;
        if entry.failures < LOGIN_FAILURES_BEFORE_BACKOFF {
            return LoginDecision::Rejected;
        }

        let exponent = entry
            .failures
            .saturating_sub(LOGIN_FAILURES_BEFORE_BACKOFF)
            .min(6);
        let seconds = LOGIN_BACKOFF_BASE
            .as_secs()
            .saturating_mul(1_u64 << exponent)
            .min(LOGIN_BACKOFF_MAX.as_secs());
        let backoff = Duration::from_secs(seconds);
        entry.blocked_until = Some(now + backoff);
        LoginDecision::Throttled {
            retry_after_seconds: backoff.as_secs(),
        }
    }

    /// Constant-time comparison over digests: comparing SHA-256 output instead
    /// of the token keeps token length and prefix matches out of the timing
    /// channel.
    fn token_matches(&self, candidate: &str) -> bool {
        if candidate.len() > MAX_OPERATOR_TOKEN_BYTES {
            return false;
        }
        let candidate_digest = digest_hex(candidate);
        let expected = self.token_digest_hex.as_bytes();
        candidate_digest
            .as_bytes()
            .iter()
            .zip(expected.iter())
            .fold(0u8, |diff, (a, b)| diff | (a ^ b))
            == 0
    }

    pub fn create_session(&self) -> String {
        let session_id = Uuid::new_v4().to_string();
        if let Ok(mut sessions) = self.sessions.lock() {
            Self::sweep_expired(&mut sessions);
            sessions.insert(session_id.clone(), Instant::now() + SESSION_TTL);
        }
        session_id
    }

    pub fn session_cookie(&self, session_id: &str, secure: bool) -> String {
        format!(
            "{SESSION_COOKIE}={session_id}; Path=/; HttpOnly; SameSite=Strict{}",
            if secure { "; Secure" } else { "" }
        )
    }

    pub fn clear_cookie(&self, secure: bool) -> String {
        format!(
            "{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{}",
            if secure { "; Secure" } else { "" }
        )
    }

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
}

#[derive(Default)]
struct LoginAttempts {
    peers: HashMap<IpAddr, LoginAttempt>,
}

impl LoginAttempts {
    fn sweep(&mut self, now: Instant) {
        self.peers.retain(|_, entry| {
            entry.blocked_until.is_some_and(|until| until > now)
                || now.saturating_duration_since(entry.last_seen) < LOGIN_ATTEMPT_RETENTION
        });
    }

    fn make_room_for(&mut self, peer: IpAddr) {
        if self.peers.contains_key(&peer) || self.peers.len() < MAX_LOGIN_PEERS {
            return;
        }
        if let Some(oldest) = self
            .peers
            .iter()
            .min_by_key(|(_, entry)| entry.last_seen)
            .map(|(peer, _)| *peer)
        {
            self.peers.remove(&oldest);
        }
    }
}

struct LoginAttempt {
    failures: u32,
    blocked_until: Option<Instant>,
    last_seen: Instant,
}

fn validated_origin(value: &str) -> Result<String> {
    let parsed = Url::parse(value.trim()).context("the web public origin is not a valid URL")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        bail!("the web public origin must use http or https");
    }
    if parsed.host().is_none() {
        bail!("the web public origin has no host");
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        bail!("the web public origin cannot contain credentials");
    }
    if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
        bail!("the web public origin cannot contain a path, query, or fragment");
    }
    Ok(serialized_origin(&parsed))
}

fn referer_origin(value: &str) -> Option<String> {
    let parsed = Url::parse(value.trim()).ok()?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return None;
    }
    Some(serialized_origin(&parsed))
}

fn loopback_origin_from_host(headers: &HeaderMap) -> Option<String> {
    let host = single_header_text(headers, &header::HOST)?;
    let parsed = Url::parse(&format!("http://{host}")).ok()?;
    (parsed.path() == "/"
        && parsed.query().is_none()
        && parsed.fragment().is_none()
        && parsed.username().is_empty()
        && parsed.password().is_none()
        && is_loopback_host(&parsed))
    .then(|| serialized_origin(&parsed))
}

fn serialized_origin(url: &Url) -> String {
    url[..Position::BeforePath]
        .trim_end_matches('/')
        .to_string()
}

fn is_loopback_host(url: &Url) -> bool {
    match url.host() {
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => {
            address.is_loopback()
                || address
                    .to_ipv4_mapped()
                    .is_some_and(|mapped| mapped.is_loopback())
        }
        Some(Host::Domain(domain)) => {
            let domain = domain.trim_end_matches('.');
            domain.eq_ignore_ascii_case("localhost")
                || domain
                    .strip_suffix(".localhost")
                    .is_some_and(|prefix| !prefix.is_empty())
        }
        None => false,
    }
}

fn single_header_text<'a>(headers: &'a HeaderMap, name: &header::HeaderName) -> Option<&'a str> {
    let mut values = headers.get_all(name).iter();
    let value = values.next()?.to_str().ok()?;
    values.next().is_none().then_some(value)
}

fn retry_after_seconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis().div_ceil(1_000))
        .unwrap_or(u64::MAX)
        .max(1)
}

fn digest_hex(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
#[path = "../../tests/unit/web/auth/tests.rs"]
mod tests;
