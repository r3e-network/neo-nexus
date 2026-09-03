# NeoOS Signer Integration

## Overview

NeoNexus is an operator and runtime **client** of the Rust NeoOS signer in
`neo-os-services/workers/neo-signer`. The signer owns private-key custody,
policy evaluation, caller authentication, rolling limits, consensus
anti-equivocation, and the audit journal. NeoNexus owns endpoint configuration,
operator presentation, and requests made on behalf of a configured workload.

The boundary is intentionally asymmetric:

- NeoNexus may hold an admin bearer credential or an Ed25519 workload
  authentication key.
- NeoNexus must never store a Neo P-256 custody private key, a NEP-2
  passphrase, an unsealed signer database, or a second copy of signer policy
  evaluation.
- A successful signature is accepted only as the result of a parsed,
  policy-bound signer request. NeoNexus never asks the service to sign a digest
  when a transaction or consensus intent exists.
- A signer refusal is normal data with a stable code. Network, protocol, and
  malformed-response failures are client errors and never become an implicit
  local-signing fallback.

The first integration increment covers endpoint validation, bearer and
Ed25519 workload authentication, health, all non-secret management operations,
transaction/consensus/raw signing, caller provisioning, and audit reads. The
workbench page exposes the management subset only. It never exposes a browser
form that forwards arbitrary bytes to a signing route.

### Trust boundary

```text
operator browser
    | HttpOnly NeoNexus session
    v
NeoNexus Rust workbench
    | validated signer URL
    | admin bearer or Ed25519 workload assertion
    v
NeoOS Rust signer / Nitro enclave
    | caller grant -> parser -> policy -> reservation -> audit -> signature
    v
public key, policy, audit metadata, or witness (never a private key)
```

HTTP is accepted only for loopback signer endpoints. A non-loopback endpoint
must use HTTPS. URL user-info, fragments, and query strings are rejected.
Redirects are disabled so credentials cannot be forwarded to another host.

### Secret import boundary

NeoNexus does not accept raw private keys or NEP-2 passphrases. Enclave-born
key generation is available through NeoNexus; importing an existing key is a
separate signer-side ceremony. A future attested import tool must verify a
fresh Nitro document, AWS certificate chain, nonce, PCR set, runtime name,
vault fingerprint, boundary digest, and nondecreasing state revision before it
sends import material directly to the signer.

TLS authenticates a network endpoint. It does not prove that the endpoint is
the expected enclave, so HTTPS alone is not enough to weaken this rule.

## API Reference

### Configuration

The common endpoint is:

- `NEONEXUS_SIGNER_URL`: signer base URL, for example
  `http://127.0.0.1:8081` or `https://custody.internal.example`.

One credential profile is loaded for the workbench management surface. The
default prefix is `NEONEXUS_SIGNER_ADMIN`.

Bearer mode:

- `NEONEXUS_SIGNER_ADMIN_TOKEN_FILE`: path to a file containing the one-time
  signer bearer token.

Workload mode:

- `NEONEXUS_SIGNER_ADMIN_CALLER_ID`: caller id provisioned by the signer.
- `NEONEXUS_SIGNER_ADMIN_WORKLOAD_KEY_FILE`: path to a file containing exactly
  32 Ed25519 secret bytes as 64 lowercase hexadecimal characters.
- `NEONEXUS_SIGNER_ADMIN_WORKLOAD_SUBJECT`: optional subject pinned on the
  signer caller row.

Bearer and workload settings are mutually exclusive. Secret values are read
from files, not command-line arguments or environment values. Missing
configuration leaves the signer page in an explicit unconfigured state; it
does not prevent unrelated fleet operations from starting.

Additional runtime consumers create their own profile with a distinct prefix,
for example `NEONEXUS_SIGNER_CONSENSUS`. They must use a signer caller whose
capability is `sign` and whose grant names only the validator key it needs.
Digest-only consumers use a separate `raw_sign` caller granted only dedicated
raw-only keys. They must never share a transaction or consensus authority key.

### Workload assertion

Every workload-authenticated request carries:

- `X-NeoOS-Caller`
- `X-NeoOS-Timestamp`
- `X-NeoOS-Nonce`
- `X-NeoOS-Signature`

The Ed25519 signature covers the exact UTF-8 bytes below:

```text
neoos-workload-v1
caller:<caller-id>
subject:<configured-subject-or-empty>
timestamp:<unix-seconds>
nonce:<fresh-uuid-without-hyphens>
method:<uppercase-method>
route:<exact-path-and-query>
body-sha256:<lowercase-sha256-of-exact-body>
origin:
```

NeoNexus serializes a request body once, hashes those exact bytes, signs the
canonical message, and sends the same bytes. It does not serialize a value
again after calculating the assertion. Server-to-server requests send no
`Origin`.

### Client outcomes

Every policy or authorization decision is represented as:

- `SignerOutcome::Allowed(T)` for an `allowed: true` response.
- `SignerOutcome::Refused { code, message, status }` for an
  `allowed: false` response, including HTTP 4xx/5xx decision responses.

Connection failures, redirects, oversized/non-JSON bodies, an absent
`allowed` discriminator, and a success body that does not match `T` are
`SignerClientError`. No caller retries a signing request automatically because
the first request may have committed a reservation and signature even if its
response was lost.

### Supported signer routes

Signing:

- `POST /signer/api/v1/sign/transaction`
- `POST /signer/api/v1/sign/consensus`
- `POST /signer/api/v1/sign/raw`
- `GET /signer/api/v1/keys/{key_id}`

Non-secret management:

- `POST /signer/api/v1/keys`
- `GET /signer/api/v1/keys`
- `GET|POST /signer/api/v1/keys/{key_id}/policy`
- `POST /signer/api/v1/keys/{key_id}/state`
- `DELETE /signer/api/v1/keys/{key_id}`
- `POST|GET /signer/api/v1/callers`
- `POST /signer/api/v1/callers/workload`
- `POST /signer/api/v1/callers/{caller_id}/rotate`
- `POST /signer/api/v1/callers/{caller_id}/state`
- `DELETE /signer/api/v1/callers/{caller_id}`
- `GET /signer/api/v1/audit`

Identity:

- `GET /health`
- `GET /attestation?nonce=<hex>` is retrievable only by the future verifier;
  an unverified document never authorizes secret import.

Deliberately absent from the NeoNexus client:

- `POST /signer/api/v1/keys/import`
- `POST /signer/api/v1/keys/import-nep2`
- `POST /provision`

Those routes carry custody or vault-release secrets and do not belong in an
operations workbench.

### Policy model

`SignerPolicy` mirrors the signer's wire contract; it does not reimplement its
decision rules:

- operation switches: consensus, transfer, contract call, global scope, raw;
- contract and exact contract-method allow/deny lists;
- asset and recipient allow/deny lists;
- global and per-asset single-amount and rolling amount windows;
- maximum transaction signers;
- maximum system and network fee;
- a key-wide rolling signature count shared by transaction, consensus, and raw
  signing routes.

Amounts are decimal strings in raw token units. Fees are decimal strings in
GAS fixed-8 raw units. A blank policy is closed. Blacklist precedence and all
other semantics are decided only by the signer. The signer rejects a policy
that combines `allow_raw` with consensus, transfer, contract-call, or global
authority. This is a hard invariant: raw signing
`networkMagicLE || SHA256(unsignedTransaction)` would otherwise produce a valid
Neo transaction signature without transaction policy evaluation.

## Usage Examples

### Configure a loopback development signer

```bash
export NEONEXUS_SIGNER_URL=http://127.0.0.1:8081
export NEONEXUS_SIGNER_ADMIN_TOKEN_FILE=/run/secrets/neoos-signer-admin
cargo run
```

Open `/signer` after signing in to the workbench. The page shows endpoint
health, public key metadata, policy advice, callers, and recent audit records.
Generating a key creates it inside the signer; no Neo private key crosses
NeoNexus.

The asset-ceiling editor accepts one limit per line as
`asset|single-maximum|window-seconds|window-maximum`; leave a field between
separators blank when that ceiling is not used. Repeating an asset with another
window enforces both periods.

### Configure workload authentication

```bash
export NEONEXUS_SIGNER_URL=https://custody.internal.example
export NEONEXUS_SIGNER_ADMIN_CALLER_ID=caller-admin-workbench
export NEONEXUS_SIGNER_ADMIN_WORKLOAD_KEY_FILE=/run/secrets/neoos-admin-ed25519
export NEONEXUS_SIGNER_ADMIN_WORKLOAD_SUBJECT=neo-nexus:production
cargo run
```

The corresponding caller is provisioned in the signer with the `admin`
capability. A validator uses a separate Ed25519 key, `sign` capability, and an
`Only([validator-key-id])` grant.

A raw compatibility client instead uses `raw_sign` and an
`Only([raw-role-key-id])` grant; that key's policy enables only `allow_raw`.

### Request a transaction signature from Rust

```rust
let outcome = client.sign_transaction(SignRequest {
    key_id: "key-validator-1".to_string(),
    unsigned_hex: unsigned_transaction_hex,
    request_id: Some("payment-job-018f".to_string()),
})?;
match outcome {
    SignerOutcome::Allowed(witness) => install_witness(witness),
    SignerOutcome::Refused(refusal) => stop_with_code(refusal.code),
}
```

The client does not retry this call automatically and has no local private-key
fallback. After an unknown transport outcome, the caller may retry the exact
request with the same `request_id`; changed bytes under that id are refused.

## Design Decisions

- **Client, not copied engine.** Keeping parsers, policy rules, sealed-key
  storage, and audit tables in NeoNexus would create two authorities and let
  their behavior drift.
- **Workload proof of possession.** URL or `Origin` binding alone does not
  identify a server. A pinned Ed25519 identity binds caller, request, body,
  time, and nonce without fetching remote identity metadata from the enclave.
- **File-backed credentials.** Process arguments and plaintext secret
  environment variables are too easy to expose through diagnostics and process
  inspection. The configured file path is not itself a credential.
- **No automatic signing retry.** A transport timeout is an unknown outcome,
  not proof that no signature or reservation was committed.
- **No import relay.** The workbench cannot accidentally retain, log, crash
  dump, or back up a private key it never accepts.
- **No policy duplication.** Client-side forms validate shape for usability;
  the signer remains the only authority that evaluates meaning.
- **Staged integration.** The non-secret management and signing client lands
  before attested import and durable Nitro-state acceptance. Missing production
  evidence remains visible instead of being represented as complete.

## Test Coverage

Automated tests must prove:

1. endpoint validation accepts loopback HTTP and remote HTTPS, while rejecting
   remote cleartext, credentials in URLs, fragments, and redirects;
2. bearer tokens and Ed25519 seeds are redacted from `Debug` and errors;
3. workload assertions bind caller, subject, method, exact route, exact body,
   timestamp, nonce, and empty server origin;
4. one body serialization is both hashed and transmitted;
5. refusal JSON is returned as data for every HTTP status;
6. malformed and oversized responses fail closed;
7. signing requests are never automatically retried;
8. NeoNexus source contains no P-256 signing, WIF, NEP-2, AES key sealing, or
   `signer_*` custody tables;
9. the authenticated `/signer` page exposes management metadata but no
   private-key, passphrase, raw-sign, transaction-sign, or consensus-sign form;
10. an ignored real-service contract test exercises every supported route
    against a spawned Rust signer and checks workload replay refusal;
11. Gitleaks reports no source credential. The two wallet-parser tests contain
    the same public NEP-2 encrypted-wallet fixture; `.gitleaks.toml` exempts only
    the `"key"` line in those exact test paths, not other values or files;
12. `cargo audit` reports no vulnerability or unsound dependency. The direct
    `anyhow` floor is `1.0.103`, which contains the `downcast_mut` fix for
    RUSTSEC-2026-0190.

Expected verification:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test web
cargo test --test signer_service_contract -- --ignored
cargo run -- --source-purity .
cargo run -- --source-quality .
cargo audit
gitleaks dir --config .gitleaks.toml --redact --no-banner src
gitleaks dir --config .gitleaks.toml --redact --no-banner tests
```
