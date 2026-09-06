//! Caller-facing signer routes (§5).
//!
//! Every method takes the caller's credential. The NeoNexus relay must never
//! substitute its admin identity: the signer owns authorization, policy, and
//! audit attribution.

use anyhow::Result;

use super::SignerClient;
use crate::signer_client::{
    CallerToken, Eip191FulfillmentRequest, Eip191FulfillmentSignature, KeyPublic, Outcome,
    RawSignRequest, RawSignature, SignRequest, Signature,
};

impl SignerClient {
    /// `POST /sign/transaction` using the legacy Neo N3 request shape.
    pub fn sign_transaction(
        &self,
        credentials: &CallerToken<'_>,
        key_id: &str,
        unsigned_hex: &str,
    ) -> Result<Outcome<Signature>> {
        self.sign_transaction_request(credentials, &SignRequest::neo_n3(key_id, unsigned_hex))
    }

    /// `POST /sign/transaction` with idempotency and explicit chain identity.
    pub fn sign_transaction_request(
        &self,
        credentials: &CallerToken<'_>,
        request: &SignRequest,
    ) -> Result<Outcome<Signature>> {
        self.post_json(credentials, "/sign/transaction", request)
    }

    /// `POST /sign/consensus` using the legacy Neo N3 request shape.
    pub fn sign_consensus(
        &self,
        credentials: &CallerToken<'_>,
        key_id: &str,
        unsigned_hex: &str,
    ) -> Result<Outcome<Signature>> {
        self.sign_consensus_request(credentials, &SignRequest::neo_n3(key_id, unsigned_hex))
    }

    /// `POST /sign/consensus` with idempotency and explicit chain identity.
    pub fn sign_consensus_request(
        &self,
        credentials: &CallerToken<'_>,
        request: &SignRequest,
    ) -> Result<Outcome<Signature>> {
        self.post_json(credentials, "/sign/consensus", request)
    }

    /// `POST /sign/raw`, guarded remotely by the key's distinct `allow_raw`.
    pub fn sign_raw(
        &self,
        credentials: &CallerToken<'_>,
        key_id: &str,
        data_hex: &str,
    ) -> Result<Outcome<RawSignature>> {
        self.sign_raw_request(credentials, &RawSignRequest::new(key_id, data_hex))
    }

    /// `POST /sign/raw` with a caller-provided idempotency key.
    pub fn sign_raw_request(
        &self,
        credentials: &CallerToken<'_>,
        request: &RawSignRequest,
    ) -> Result<Outcome<RawSignature>> {
        self.post_json(credentials, "/sign/raw", request)
    }

    /// `POST /sign/eip191-fulfillment` with structured, contract-bound fields.
    pub fn sign_eip191_fulfillment(
        &self,
        credentials: &CallerToken<'_>,
        request: &Eip191FulfillmentRequest,
    ) -> Result<Outcome<Eip191FulfillmentSignature>> {
        self.post_json(credentials, "/sign/eip191-fulfillment", request)
    }

    /// `GET /keys/{id}`: public key identity guarded by `sign`.
    pub fn key_info(
        &self,
        credentials: &CallerToken<'_>,
        key_id: &str,
    ) -> Result<Outcome<KeyPublic>> {
        self.get(
            credentials,
            &format!("/keys/{}", super::transport::checked_id(key_id)?),
            None,
        )
    }
}
