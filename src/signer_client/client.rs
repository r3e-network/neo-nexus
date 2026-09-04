//! Blocking HTTP client for the signer service.
//!
//! NeoNexus handlers call this client from `spawn_blocking`; keeping transport
//! synchronous matches the rest of the workbench's repository and probe
//! clients without ever blocking an async executor thread directly.

use std::{
    fmt,
    io::Read,
    time::{Duration, SystemTime},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use ureq::{Agent, AgentBuilder, Request, Response};
use uuid::Uuid;

use super::{
    ApiKeyCallerRequest, AuditBody, AuditEntry, AuditFilter, AuthHeaders, CallersBody,
    CreateCallerRequest, CreateWorkloadCallerRequest, CreatedApiKeyCaller, CreatedCaller,
    CreatedWorkloadCaller, GenerateKeyRequest, KeyPolicy, KeysBody, OidcCallerRequest,
    RawSignRequest, RawSignature, RemovedCaller, RemovedKey, RotatedCaller, SavedPolicy,
    SignRequest, SignedWitness, SignerCaller, SignerClientConfig, SignerEndpoint, SignerHealth,
    SignerKey, SignerOutcome, SignerPolicy, SignerRefusal, StateRequest,
};

const API: &str = "/signer/api/v1";
const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct SignerClient {
    endpoint: SignerEndpoint,
    credential: super::SignerCredential,
    agent: Agent,
}

impl fmt::Debug for SignerClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignerClient")
            .field("endpoint", &self.endpoint)
            .field("credential", &self.credential)
            .finish_non_exhaustive()
    }
}

impl SignerClient {
    pub fn new(config: SignerClientConfig) -> Self {
        Self {
            endpoint: config.endpoint,
            credential: config.credential,
            agent: AgentBuilder::new()
                .redirects(0)
                .timeout(REQUEST_TIMEOUT)
                .build(),
        }
    }

    pub fn from_env(prefix: &str) -> Result<Option<Self>, SignerClientError> {
        SignerClientConfig::from_env(prefix)
            .map(|config| config.map(Self::new))
            .map_err(SignerClientError::configuration)
    }

    pub fn endpoint(&self) -> &SignerEndpoint {
        &self.endpoint
    }

    pub fn health(&self) -> Result<SignerHealth, SignerClientError> {
        self.health_with_timeout(REQUEST_TIMEOUT)
    }

    /// Probe custody without sending an authentication credential. Background
    /// monitors use a shorter deadline than interactive management requests.
    pub fn health_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<SignerHealth, SignerClientError> {
        self.get_plain("/health", timeout)
    }

    pub fn sign_transaction(
        &self,
        request: &SignRequest,
    ) -> Result<SignerOutcome<SignedWitness>, SignerClientError> {
        let evm = request_family(request.chain_family.as_deref())?;
        if evm != request.chain_id.is_some() {
            return Err(SignerClientError::request(
                "chain_id is required for NeoX transactions and must be absent for Neo N3",
            ));
        }
        let outcome = self.post("/sign/transaction", request)?;
        validate_witness(outcome, &request.key_id, evm)
    }

    pub fn sign_consensus(
        &self,
        request: &SignRequest,
    ) -> Result<SignerOutcome<SignedWitness>, SignerClientError> {
        if request_family(request.chain_family.as_deref())? || request.chain_id.is_some() {
            return Err(SignerClientError::request(
                "consensus signing supports only Neo N3; NeoX consensus requires a native adapter",
            ));
        }
        let outcome = self.post("/sign/consensus", request)?;
        validate_witness(outcome, &request.key_id, false)
    }

    pub fn sign_raw(
        &self,
        request: &RawSignRequest,
    ) -> Result<SignerOutcome<RawSignature>, SignerClientError> {
        if request_family(request.chain_family.as_deref())? {
            return Err(SignerClientError::request(
                "raw signing supports only Neo N3",
            ));
        }
        self.post("/sign/raw", request)
    }

    pub fn key_info(&self, key_id: &str) -> Result<SignerOutcome<SignerKey>, SignerClientError> {
        self.get(&format!("/keys/{}", path_segment(key_id)))
    }

    pub fn generate_key(
        &self,
        request: &GenerateKeyRequest,
    ) -> Result<SignerOutcome<SignerKey>, SignerClientError> {
        let evm = request_family(request.chain_family.as_deref())?;
        if (evm && request.network_magic.is_some()) || (!evm && request.chain_id.is_some()) {
            return Err(SignerClientError::request(
                "use network_magic for Neo N3 keys and chain_id for NeoX keys",
            ));
        }
        if evm && request.network == "private" && request.chain_id.is_none() {
            return Err(SignerClientError::request(
                "a private NeoX key requires chain_id",
            ));
        }
        self.post("/keys", request)
    }

    pub fn list_keys(&self) -> Result<SignerOutcome<Vec<SignerKey>>, SignerClientError> {
        self.get::<KeysBody>("/keys")
            .map(|outcome| map_outcome(outcome, |body| body.keys))
    }

    pub fn key_policy(&self, key_id: &str) -> Result<SignerOutcome<KeyPolicy>, SignerClientError> {
        self.get(&format!("/keys/{}/policy", path_segment(key_id)))
    }

    pub fn save_policy(
        &self,
        key_id: &str,
        policy: &SignerPolicy,
    ) -> Result<SignerOutcome<SavedPolicy>, SignerClientError> {
        self.post(&format!("/keys/{}/policy", path_segment(key_id)), policy)
    }

    pub fn set_key_disabled(
        &self,
        key_id: &str,
        disabled: bool,
    ) -> Result<SignerOutcome<SignerKey>, SignerClientError> {
        self.post(
            &format!("/keys/{}/state", path_segment(key_id)),
            &StateRequest { disabled },
        )
    }

    pub fn delete_key(&self, key_id: &str) -> Result<SignerOutcome<RemovedKey>, SignerClientError> {
        self.delete(&format!("/keys/{}", path_segment(key_id)))
    }

    pub fn create_caller(
        &self,
        request: &CreateCallerRequest,
    ) -> Result<SignerOutcome<CreatedCaller>, SignerClientError> {
        self.post("/callers", request)
    }

    pub fn create_workload_caller(
        &self,
        request: &CreateWorkloadCallerRequest,
    ) -> Result<SignerOutcome<CreatedWorkloadCaller>, SignerClientError> {
        self.post("/callers/workload", request)
    }

    pub fn create_oidc_caller(
        &self,
        _request: &OidcCallerRequest,
    ) -> Result<SignerOutcome<CreatedCaller>, SignerClientError> {
        Err(SignerClientError::request(
            "the NeoOS signer does not implement OIDC caller provisioning",
        ))
    }

    pub fn create_api_key_caller(
        &self,
        _request: &ApiKeyCallerRequest,
    ) -> Result<SignerOutcome<CreatedApiKeyCaller>, SignerClientError> {
        Err(SignerClientError::request(
            "the NeoOS signer does not implement API-key caller provisioning",
        ))
    }

    pub fn list_callers(&self) -> Result<SignerOutcome<Vec<SignerCaller>>, SignerClientError> {
        self.get::<CallersBody>("/callers")
            .map(|outcome| map_outcome(outcome, |body| body.callers))
    }

    pub fn rotate_caller(
        &self,
        caller_id: &str,
    ) -> Result<SignerOutcome<RotatedCaller>, SignerClientError> {
        self.post_empty(&format!("/callers/{}/rotate", path_segment(caller_id)))
    }

    pub fn set_caller_disabled(
        &self,
        caller_id: &str,
        disabled: bool,
    ) -> Result<SignerOutcome<SignerCaller>, SignerClientError> {
        self.post(
            &format!("/callers/{}/state", path_segment(caller_id)),
            &StateRequest { disabled },
        )
    }

    pub fn delete_caller(
        &self,
        caller_id: &str,
    ) -> Result<SignerOutcome<RemovedCaller>, SignerClientError> {
        self.delete(&format!("/callers/{}", path_segment(caller_id)))
    }

    pub fn audit(
        &self,
        filter: &AuditFilter,
    ) -> Result<SignerOutcome<Vec<AuditEntry>>, SignerClientError> {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        if let Some(key_id) = filter.key_id.as_deref() {
            serializer.append_pair("key_id", key_id);
        }
        if let Some(limit) = filter.limit {
            serializer.append_pair("limit", &limit.to_string());
        }
        let query = serializer.finish();
        let route = if query.is_empty() {
            "/audit".to_string()
        } else {
            format!("/audit?{query}")
        };
        self.get::<AuditBody>(&route)
            .map(|outcome| map_outcome(outcome, |body| body.entries))
    }

    fn get<T: DeserializeOwned>(
        &self,
        relative_route: &str,
    ) -> Result<SignerOutcome<T>, SignerClientError> {
        self.request("GET", &api_route(relative_route), None)
    }

    fn delete<T: DeserializeOwned>(
        &self,
        relative_route: &str,
    ) -> Result<SignerOutcome<T>, SignerClientError> {
        self.request("DELETE", &api_route(relative_route), None)
    }

    fn post<T: DeserializeOwned>(
        &self,
        relative_route: &str,
        body: &impl Serialize,
    ) -> Result<SignerOutcome<T>, SignerClientError> {
        let body = serde_json::to_vec(body)
            .map_err(|error| SignerClientError::request(error.to_string()))?;
        self.request("POST", &api_route(relative_route), Some(&body))
    }

    fn post_empty<T: DeserializeOwned>(
        &self,
        relative_route: &str,
    ) -> Result<SignerOutcome<T>, SignerClientError> {
        self.request("POST", &api_route(relative_route), Some(b"{}"))
    }

    fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        route: &str,
        body: Option<&[u8]>,
    ) -> Result<SignerOutcome<T>, SignerClientError> {
        let timestamp = unix_now()?;
        let nonce = Uuid::new_v4().simple().to_string();
        self.request_at(method, route, body, timestamp, &nonce)
    }

    fn request_at<T: DeserializeOwned>(
        &self,
        method: &str,
        route: &str,
        body: Option<&[u8]>,
        timestamp: u64,
        nonce: &str,
    ) -> Result<SignerOutcome<T>, SignerClientError> {
        let exact_body = body.unwrap_or_default();
        let headers = self
            .credential
            .headers(
                (self.endpoint.as_str(), method, route),
                exact_body,
                timestamp,
                nonce,
            )
            .map_err(SignerClientError::configuration)?;
        let url = self
            .endpoint
            .request_url(route)
            .map_err(SignerClientError::configuration)?;
        let request = apply_headers(self.agent.request(method, url.as_str()), &headers);
        let result = match body {
            Some(body) => request
                .set("content-type", "application/json")
                .send_bytes(body),
            None => request.call(),
        };
        parse_outcome(transport_response(result)?)
    }

    fn get_plain<T: DeserializeOwned>(
        &self,
        route: &str,
        timeout: Duration,
    ) -> Result<T, SignerClientError> {
        let url = self
            .endpoint
            .request_url(route)
            .map_err(SignerClientError::configuration)?;
        let response = transport_response(self.agent.get(url.as_str()).timeout(timeout).call())?;
        if !(200..300).contains(&response.status()) {
            return Err(SignerClientError::protocol(format!(
                "signer health returned HTTP {}",
                response.status()
            )));
        }
        let body = response_bytes(response)?;
        serde_json::from_slice(&body)
            .map_err(|_| SignerClientError::protocol("signer health response is not valid JSON"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerClientError {
    kind: SignerClientErrorKind,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignerClientErrorKind {
    Configuration,
    Request,
    Transport,
    Protocol,
}

impl SignerClientError {
    pub fn kind(&self) -> SignerClientErrorKind {
        self.kind
    }

    fn configuration(error: impl fmt::Display) -> Self {
        Self::new(SignerClientErrorKind::Configuration, error)
    }

    fn request(error: impl fmt::Display) -> Self {
        Self::new(SignerClientErrorKind::Request, error)
    }

    fn transport(error: impl fmt::Display) -> Self {
        Self::new(SignerClientErrorKind::Transport, error)
    }

    fn protocol(error: impl fmt::Display) -> Self {
        Self::new(SignerClientErrorKind::Protocol, error)
    }

    fn new(kind: SignerClientErrorKind, error: impl fmt::Display) -> Self {
        Self {
            kind,
            message: error.to_string(),
        }
    }
}

impl fmt::Display for SignerClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SignerClientError {}

fn apply_headers(mut request: Request, headers: &AuthHeaders) -> Request {
    request = request.set("accept", "application/json");
    if let Some(value) = headers.workload_protocol.as_deref() {
        request = request.set("x-neoos-workload-protocol", value);
    }
    if let Some(value) = headers.audience.as_deref() {
        request = request.set("x-neoos-audience", value);
    }
    if let Some(value) = headers.authorization.as_deref() {
        request = request.set("authorization", value);
    }
    if let Some(value) = headers.caller_id.as_deref() {
        request = request.set("x-neoos-caller", value);
    }
    if let Some(value) = headers.timestamp.as_deref() {
        request = request.set("x-neoos-timestamp", value);
    }
    if let Some(value) = headers.nonce.as_deref() {
        request = request.set("x-neoos-nonce", value);
    }
    if let Some(value) = headers.signature.as_deref() {
        request = request.set("x-neoos-signature", value);
    }
    request
}

fn transport_response(
    result: Result<Response, ureq::Error>,
) -> Result<Response, SignerClientError> {
    match result {
        Ok(response) => Ok(response),
        Err(ureq::Error::Status(_, response)) => Ok(response),
        Err(ureq::Error::Transport(error)) => Err(SignerClientError::transport(error)),
    }
}

fn parse_outcome<T: DeserializeOwned>(
    response: Response,
) -> Result<SignerOutcome<T>, SignerClientError> {
    let status = response.status();
    if (300..400).contains(&status) {
        return Err(SignerClientError::protocol(
            "signer redirected an authenticated request",
        ));
    }
    let body = response_bytes(response)?;
    let mut document: Value = serde_json::from_slice(&body)
        .map_err(|_| SignerClientError::protocol("signer response is not valid JSON"))?;
    let allowed = document
        .get("allowed")
        .and_then(Value::as_bool)
        .ok_or_else(|| SignerClientError::protocol("signer response has no allowed boolean"))?;
    if !allowed {
        let refusal: RefusalBody = serde_json::from_value(document)
            .map_err(|_| SignerClientError::protocol("signer refusal shape is invalid"))?;
        return Ok(SignerOutcome::Refused(SignerRefusal {
            status,
            code: refusal.code,
            message: refusal.message,
        }));
    }
    if !(200..300).contains(&status) {
        return Err(SignerClientError::protocol(
            "signer returned an allowed body with a failing status",
        ));
    }
    if let Value::Object(fields) = &mut document {
        fields.remove("allowed");
    }
    serde_json::from_value(document)
        .map(SignerOutcome::Allowed)
        .map_err(|_| SignerClientError::protocol("signer success shape is invalid"))
}

fn response_bytes(response: Response) -> Result<Vec<u8>, SignerClientError> {
    if response
        .header("content-length")
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|length| length > MAX_RESPONSE_BYTES)
    {
        return Err(SignerClientError::protocol(
            "signer response exceeds the size limit",
        ));
    }
    let mut body = Vec::new();
    response
        .into_reader()
        .take(MAX_RESPONSE_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(SignerClientError::transport)?;
    if body.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(SignerClientError::protocol(
            "signer response exceeds the size limit",
        ));
    }
    Ok(body)
}

fn api_route(relative: &str) -> String {
    format!("{API}{relative}")
}

fn path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn map_outcome<T, U>(outcome: SignerOutcome<T>, map: impl FnOnce(T) -> U) -> SignerOutcome<U> {
    match outcome {
        SignerOutcome::Allowed(value) => SignerOutcome::Allowed(map(value)),
        SignerOutcome::Refused(refusal) => SignerOutcome::Refused(refusal),
    }
}

fn request_family(family: Option<&str>) -> Result<bool, SignerClientError> {
    match family {
        None | Some("neo-n3") => Ok(false),
        Some("neox") => Ok(true),
        _ => Err(SignerClientError::request(
            "signer chain_family must be neo-n3 or neox",
        )),
    }
}

fn validate_witness(
    outcome: SignerOutcome<SignedWitness>,
    key_id: &str,
    evm: bool,
) -> Result<SignerOutcome<SignedWitness>, SignerClientError> {
    let SignerOutcome::Allowed(mut witness) = outcome else {
        return Ok(outcome);
    };
    let family_matches = if evm {
        witness.chain_family.as_deref() == Some("neox")
    } else {
        matches!(witness.chain_family.as_deref(), None | Some("neo-n3"))
    };
    if witness.key_id != key_id || !family_matches {
        return Err(SignerClientError::protocol(
            "signer witness key or chain family does not match the request",
        ));
    }
    if evm {
        let transaction = witness
            .signed_transaction
            .as_deref()
            .or(witness.signature_hex.as_deref());
        let Some(transaction) = transaction else {
            return Err(SignerClientError::protocol(
                "NeoX signer response has no signed_transaction",
            ));
        };
        let hex = transaction.strip_prefix("0x").unwrap_or(transaction);
        if hex.is_empty()
            || !hex.len().is_multiple_of(2)
            || !hex.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(SignerClientError::protocol(
                "NeoX signed_transaction is not hexadecimal bytes",
            ));
        }
        if let (Some(canonical), Some(alias)) =
            (&witness.signed_transaction, &witness.signature_hex)
        {
            if !canonical.eq_ignore_ascii_case(alias) {
                return Err(SignerClientError::protocol(
                    "NeoX signed_transaction conflicts with its compatibility alias",
                ));
            }
        }
        witness.signed_transaction = Some(transaction.to_string());
    } else if witness.signed_transaction.is_some() || witness.signature_hex.is_some() {
        return Err(SignerClientError::protocol(
            "Neo N3 signer returned an EVM transaction",
        ));
    }
    Ok(SignerOutcome::Allowed(witness))
}

fn unix_now() -> Result<u64, SignerClientError> {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .map_err(SignerClientError::request)
}

#[derive(Deserialize)]
struct RefusalBody {
    code: String,
    message: String,
}
