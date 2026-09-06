//! Signer administration routes (§5.1).
//!
//! Credentials remain explicit on every call. This prevents a future caller
//! from being silently upgraded to the console's whole-vault identity and keeps
//! the signer service authoritative for capability checks and audit ownership.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{transport::checked_id, SignerClient};
use crate::signer_client::{
    AuditRow, Caller, CallerToken, CreatedCaller, CreatedWorkloadCaller, GenerateKeyRequest, Grant,
    KeyBoundary, KeyPublic, Outcome, Policy, RemovedCaller, RemovedKey, RotatedCaller,
    SavedBoundary, WorkloadCallerRequest,
};

impl SignerClient {
    /// `POST /keys` using the legacy Neo N3 request shape.
    pub fn generate_key(
        &self,
        credentials: &CallerToken<'_>,
        label: &str,
        network: &str,
        network_magic: Option<u32>,
    ) -> Result<Outcome<KeyPublic>> {
        self.generate_key_request(
            credentials,
            &GenerateKeyRequest::neo_n3(label, network, network_magic),
        )
    }

    /// `POST /keys` with an explicit immutable chain family and chain id.
    pub fn generate_key_request(
        &self,
        credentials: &CallerToken<'_>,
        request: &GenerateKeyRequest,
    ) -> Result<Outcome<KeyPublic>> {
        self.post_json(credentials, "/keys", request)
    }

    /// `GET /keys`.
    pub fn list_keys(&self, credentials: &CallerToken<'_>) -> Result<Outcome<Vec<KeyPublic>>> {
        #[derive(Deserialize)]
        struct KeysBody {
            keys: Vec<KeyPublic>,
        }

        let outcome: Outcome<KeysBody> = self.get(credentials, "/keys", None)?;
        Ok(super::transport::map_outcome(outcome, |body| body.keys))
    }

    /// `POST /keys/{id}/state`.
    pub fn set_key_disabled(
        &self,
        credentials: &CallerToken<'_>,
        key_id: &str,
        disabled: bool,
    ) -> Result<Outcome<KeyPublic>> {
        #[derive(Serialize)]
        struct StateCall {
            disabled: bool,
        }

        let path = format!("/keys/{}/state", checked_id(key_id)?);
        self.post_json(credentials, &path, &StateCall { disabled })
    }

    /// `GET /keys/{id}/policy`.
    pub fn key_boundary(
        &self,
        credentials: &CallerToken<'_>,
        key_id: &str,
    ) -> Result<Outcome<KeyBoundary>> {
        self.get(
            credentials,
            &format!("/keys/{}/policy", checked_id(key_id)?),
            None,
        )
    }

    /// `POST /keys/{id}/policy`; returns the normalized stored boundary.
    pub fn save_policy(
        &self,
        credentials: &CallerToken<'_>,
        key_id: &str,
        policy: &Policy,
    ) -> Result<Outcome<SavedBoundary>> {
        let path = format!("/keys/{}/policy", checked_id(key_id)?);
        self.post_json(credentials, &path, policy)
    }

    /// `DELETE /keys/{id}`.
    pub fn delete_key(
        &self,
        credentials: &CallerToken<'_>,
        key_id: &str,
    ) -> Result<Outcome<RemovedKey>> {
        self.request(
            credentials,
            "DELETE",
            &format!("/keys/{}", checked_id(key_id)?),
            None,
            None,
        )
    }

    /// `POST /callers`; the response carries its bearer token exactly once.
    pub fn create_caller(
        &self,
        credentials: &CallerToken<'_>,
        label: &str,
        key_grant: &Grant,
        capabilities: &[String],
        allowed_origins: &[String],
    ) -> Result<Outcome<CreatedCaller>> {
        #[derive(Serialize)]
        struct CallerCall<'a> {
            label: &'a str,
            key_grant: &'a Grant,
            capabilities: &'a [String],
            allowed_origins: &'a [String],
        }

        self.post_json(
            credentials,
            "/callers",
            &CallerCall {
                label,
                key_grant,
                capabilities,
                allowed_origins,
            },
        )
    }

    /// `POST /callers/workload` using only public Ed25519 identity material.
    pub fn create_workload_caller(
        &self,
        credentials: &CallerToken<'_>,
        request: &WorkloadCallerRequest,
    ) -> Result<Outcome<CreatedWorkloadCaller>> {
        self.post_json(credentials, "/callers/workload", request)
    }

    /// `GET /callers`.
    pub fn list_callers(&self, credentials: &CallerToken<'_>) -> Result<Outcome<Vec<Caller>>> {
        #[derive(Deserialize)]
        struct CallersBody {
            callers: Vec<Caller>,
        }

        let outcome: Outcome<CallersBody> = self.get(credentials, "/callers", None)?;
        Ok(super::transport::map_outcome(outcome, |body| body.callers))
    }

    /// `POST /callers/{id}/rotate`.
    pub fn rotate_caller_token(
        &self,
        credentials: &CallerToken<'_>,
        caller_id: &str,
    ) -> Result<Outcome<RotatedCaller>> {
        self.request(
            credentials,
            "POST",
            &format!("/callers/{}/rotate", checked_id(caller_id)?),
            None,
            None,
        )
    }

    /// `POST /callers/{id}/state`.
    pub fn set_caller_disabled(
        &self,
        credentials: &CallerToken<'_>,
        caller_id: &str,
        disabled: bool,
    ) -> Result<Outcome<Caller>> {
        #[derive(Serialize)]
        struct StateCall {
            disabled: bool,
        }

        let path = format!("/callers/{}/state", checked_id(caller_id)?);
        self.post_json(credentials, &path, &StateCall { disabled })
    }

    /// `DELETE /callers/{id}`.
    pub fn delete_caller(
        &self,
        credentials: &CallerToken<'_>,
        caller_id: &str,
    ) -> Result<Outcome<RemovedCaller>> {
        self.request(
            credentials,
            "DELETE",
            &format!("/callers/{}", checked_id(caller_id)?),
            None,
            None,
        )
    }

    /// `GET /audit`, newest first and optionally filtered to one key.
    pub fn list_audit(
        &self,
        credentials: &CallerToken<'_>,
        key_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Outcome<Vec<AuditRow>>> {
        #[derive(Deserialize)]
        struct AuditBody {
            entries: Vec<AuditRow>,
        }

        let query = match key_id {
            Some(key_id) => format!("key_id={}", checked_id(key_id)?),
            None => String::new(),
        };
        let query = match limit {
            Some(limit) if query.is_empty() => format!("limit={limit}"),
            Some(limit) => format!("{query}&limit={limit}"),
            None => query,
        };
        let outcome: Outcome<AuditBody> = self.get(
            credentials,
            "/audit",
            (!query.is_empty()).then_some(query.as_str()),
        )?;
        Ok(super::transport::map_outcome(outcome, |body| body.entries))
    }
}
