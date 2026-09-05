# Configuration Reference

This document lists all supported configuration options for NeoNexus deployment.

---

## Environment Variables

### Authentication

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `NEONEXUS_WEB_TOKEN` | Sign-in token for web workbench | Generated at startup if unset | `abc123def456` |
| `NEONEXUS_DATA_DIR` | Override data directory path | Auto-detected beside database | `/var/lib/neo-nexus` |

### Signer Service

The following variables configure external signer access:

| Variable | Description | Required For |
|----------|-------------|--------------|
| `NEONEXUS_SIGNER_ADMIN` | Admin endpoint URL or file path | Consensus signing operations |
| `SIGNER_...` | Various signer credentials | HSM/cloud KMS integration |

See [signer-service.md](signer-service.md) for full signer configuration details.

### Agent Profiles (Hermes)

Environment variables prefixed with `NEONEXUS_ASSISTANT_` are used to configure agent profiles in `hermes_config.rs`. These avoid hardcoding secrets:

- `NEONEXUS_ASSISTANT_BEARER` - Bearer token for API access
- Other `NEONEXUS_ASSISTANT_*` vars resolved dynamically

---

## Deployment-Specific Variables

### Linux systemd Installation

Used by `deploy/systemd/install.sh`:

| Variable | Default | Purpose |
|----------|---------|---------|
| `PREFIX` | `/opt/neo-nexus` | Binary installation root |
| `DATA_DIR` | `/var/lib/neo-nexus` | Persistent data directory |
| `UNIT_SOURCE` | `deploy/systemd/neo-nexus.service` | Unit template location |
| `UNIT_TARGET` | `/etc/systemd/system/neo-nexus.service` | Installed unit path |
| `SERVICE_USER` | `neo-nexus` | Service account name |

**Credential files:** Use `EnvironmentFile=-/etc/neo-nexus/credentials.env` to supply sensitive values like `NEONEXUS_WEB_TOKEN` without exposing them in logs.

### Windows Service Installation

Parameters passed to PowerShell installer `deploy/windows/install-service.ps1`:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `-BinaryPath` | **Required** | Full path to neo-nexus.exe |
| `-DataDir` | `%ProgramData%\NeoNexus` | Data directory |
| `-ServiceName` | `NeoNexus` | Windows service name |
| `-DisplayName` | `NeoNexus Node Operations Workbench` | Display name in Services MMC |
| `-Account` | `NT AUTHORITY\LocalService` | Service account |

---

## Web UI Runtime Settings

These settings can be configured via the **Settings** page in the web workbench:

### Watchdog Policy

| Setting | Type | Allowed Values | Default |
|---------|------|----------------|---------|
| `enabled` | boolean | true/false | false |
| `max_restart_attempts` | integer | 1–20 | 5 |
| `base_delay_seconds` | integer | 1–60 | 5 |
| `max_delay_seconds` | integer | base × 2 to 60 | 20 |

### RPC Health Monitor

| Setting | Type | Allowed Values | Default |
|---------|------|----------------|---------|
| `enabled` | boolean | true/false | true |
| `interval_seconds` | integer | 5–3600 | 30 |

### Federation Monitor

Same schema as RPC Health Monitor, monitors remote node synchronization.

---

## Alert Routing Policies

Configured via **Alerts → Routing** or JSON policy files.

### Alert Provider Schemes

Supported providers require specific URL formats:

| Provider | Scheme Format | Notes |
|----------|---------------|-------|
| **Opsgenie** | `https://api.opsgenie.com/v2/alerts?api_key=<KEY>` | api_key required |
| **PagerDuty** | `https://events.pagerduty.com/v2/enqueue` | Event routing key in body |
| **Datadog** | `https://event-management-intake.datadoghq.com/api/v2/events?api_key=<KEY>` | api_key required |
| **Telegram** | `https://api.telegram.org/bot<TOKEN>/sendMessage` | Bot token + chat_id |
| **URL Webhook** | `https://...` | HTTPS preferred; no credentials allowed |

### Severity Filtering

| Severity | Min Value | Description |
|----------|-----------|-------------|
| `info` | 0 | All events |
| `warning` | 1 | Warning and critical |
| `critical` | 2 | Critical only |

---

## Node Configuration Templates

Generated via `--generate-node-config <type> <network> <storage> <rpc-port> <p2p-port> <output-path>`:

### Client Types

| Type | Family | Config File | Notes |
|------|--------|-------------|-------|
| `neo-cli` | N3 | config.mainnet.json / config.testnet.json | C# .NET CLI |
| `neo-go` | N3 | config.yml | Go implementation |
| `neo-rs` | N3 | config.toml | Native Rust implementation |
| `neox-geth` | X | reth.toml | Geth-based bridge |
| `neox-rs` | X | neox.yaml | Rust-native Neo X |

### Storage Engines

| Engine | Extension | Notes |
|--------|-----------|-------|
| `leveldb` | `_db` folder | Classic LevelDB storage |
| `rocksdb` | `data` folder | Facebook RocksDB storage |

---

## Session Management

### Browser Sessions

- **Cookie Name**: `neonexus_session`
- **TTL**: 12 hours (configurable via SESSION_TTL constant)
- **Security**: HttpOnly, SameSite=Lax
- **Auto-refresh**: Sliding expiry on activity

### CSRF Protection

One-time-use tokens stored server-side bound to session. Tokens consumed immediately after validation.

---

## Database Paths

Workspace organization under `$DATA_DIR`:

```
/var/lib/neo-nexus/
├── neonexus.db          # Main workspace database
├── nodes/               # Node configs and wallets
│   ├── {node-id}/
│   │   ├── managed.config
│   │   └── plugins/
├── logs/                # Supervised node logs
│   ├── {node-id}-start.log
│   └── {node-id}-restart.log
└── snapshots/           # Database backups
```

---

## Logging and Events

### Event Journal Retention

Currently append-only; recommended archival strategy:

- Keep last 10,000 entries
- Export older entries to external storage before purging
- Consider monthly partitions for large deployments

### Log Levels

Events recorded with severity:

- `info` - Routine operations (node start/stop/config change)
- `warning` - Non-fatal issues (RPC timeout, designation not found)
- `critical` - Operational failures (launch blocked, consensus offline)

---

*Last updated: 2026-09-06*  
*Author: Qoder Agent (Documentation Audit)*
