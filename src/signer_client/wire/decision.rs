//! Contract-wide decision and authentication envelope types.

/// What the service decided.
///
/// A policy denial is a completed conversation rather than a transport error,
/// so it remains a distinct return-value variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome<T> {
    Allowed(T),
    Refused(Refusal),
}

impl<T> Outcome<T> {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed(_))
    }

    pub fn refusal(&self) -> Option<&Refusal> {
        match self {
            Self::Allowed(_) => None,
            Self::Refused(refusal) => Some(refusal),
        }
    }

    pub fn into_parts(self) -> Result<T, Refusal> {
        match self {
            Self::Allowed(payload) => Ok(payload),
            Self::Refused(refusal) => Err(refusal),
        }
    }
}

/// A denial exactly as the caller sees it.
///
/// There is deliberately no `detail`; detailed refusal evidence is available
/// only through the privileged [`crate::signer_client::AuditRow`] route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub code: String,
    pub message: String,
    /// The service-provided status. NeoNexus does not duplicate code→status
    /// policy that could drift from the custody service.
    pub status: u16,
}

impl Refusal {
    pub fn summary(&self) -> String {
        if self.message.trim().is_empty() {
            self.code.clone()
        } else {
            format!("{}: {}", self.code, self.message)
        }
    }

    /// A bounded retry hint for the signer's stable overload refusal.
    ///
    /// NeoNexus intentionally derives this from the exact status/code contract
    /// instead of retaining arbitrary upstream response headers. That keeps the
    /// relay's caller-facing header surface explicit and injection-proof.
    pub fn retry_after_seconds(&self) -> Option<u64> {
        (self.status == 503 && self.code == "signer-service-busy").then_some(1)
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.summary())
    }
}

impl std::error::Error for Refusal {}

/// A caller credential and the browser-origin inputs the signer uses during
/// authentication.
///
/// Public constructors create bearer credentials for the public relay API.
/// Production configuration can additionally borrow a proof-of-possession
/// workload identity; that constructor remains crate-private so a browser relay
/// cannot reinterpret bearer input as an admin workload assertion.
///
/// Deliberately not `Debug`: this value may borrow a signing or whole-vault
/// admin credential.
#[derive(Clone, Copy)]
pub struct CallerToken<'a> {
    bearer: Option<&'a str>,
    workload: Option<&'a WorkloadCredential>,
    origin: Option<&'a str>,
    referer: Option<&'a str>,
}

impl<'a> CallerToken<'a> {
    /// Program-to-program credential with no browser origin.
    pub fn bearer(token: &'a str) -> Self {
        Self {
            bearer: Some(token),
            workload: None,
            origin: None,
            referer: None,
        }
    }

    /// Browser credential retaining the exact headers received by the relay.
    pub fn browser(token: &'a str, origin: Option<&'a str>, referer: Option<&'a str>) -> Self {
        Self {
            bearer: Some(token),
            workload: None,
            origin,
            referer,
        }
    }

    /// Backwards-compatible bearer accessor.
    ///
    /// Workload credentials have no bearer secret and return an empty string;
    /// transport code uses the typed accessors below and never turns that empty
    /// value into an `Authorization` header.
    pub fn token(&self) -> &str {
        self.bearer.unwrap_or_default()
    }

    pub fn origin(&self) -> Option<&str> {
        self.origin
    }

    pub fn referer(&self) -> Option<&str> {
        self.referer
    }

    pub(crate) fn workload(workload: &'a WorkloadCredential) -> Self {
        Self {
            bearer: None,
            workload: Some(workload),
            origin: None,
            referer: None,
        }
    }

    pub(crate) fn bearer_value(&self) -> Option<&str> {
        self.bearer
    }

    pub(crate) fn workload_value(&self) -> Option<&WorkloadCredential> {
        self.workload
    }
}

/// A configured Ed25519 proof-of-possession identity.
///
/// It has no `Debug`, serialization, or public field access. The 32-byte seed is
/// parsed from a protected file at startup and held by `SigningKey`, whose
/// zeroize-on-drop implementation clears its secret representation.
#[derive(Clone)]
pub(crate) struct WorkloadCredential {
    caller_id: String,
    subject: Option<String>,
    signing_key: ed25519_dalek::SigningKey,
}

impl WorkloadCredential {
    pub(crate) fn new(caller_id: String, subject: Option<String>, mut seed: [u8; 32]) -> Self {
        use zeroize::Zeroize;

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        seed.zeroize();
        Self {
            caller_id,
            subject,
            signing_key,
        }
    }

    pub(crate) fn caller_id(&self) -> &str {
        &self.caller_id
    }

    pub(crate) fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    pub(crate) fn sign(&self, message: &[u8]) -> [u8; 64] {
        use ed25519_dalek::Signer;

        self.signing_key.sign(message).to_bytes()
    }
}
