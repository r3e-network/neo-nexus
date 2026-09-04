# Connect an existing Hermes assistant

NeoNexus connects an installed [Nous Research Hermes](https://github.com/NousResearch/hermes-agent) instance to scoped node tools. Configure your conversation channels, model provider and allowed conversation users in Hermes itself. NeoNexus does not duplicate channel setup or send a test message.

1. Register the existing Hermes installation on **Agents**, including its Python interpreter, source directory and profile's `config.yaml`.
2. Stop that instance, then open **Assistants** and select it.
3. Choose individual nodes or explicitly grant access to all current and future nodes. Monitoring is the default. Enable operations only when the assistant should be able to start, stop and restart the selected nodes.
4. Enter the NeoNexus `/mcp` address reachable from that Hermes process. The form initially uses the workbench address. HTTPS and loopback HTTP are supported; embedded credentials, query parameters and fragments are rejected.
5. Save the connection, then start Hermes from **Agents**. Use your existing Hermes conversation channel to ask about the authorized nodes.

Saving configures the connection; it does not claim that Hermes has completed an MCP handshake. Hermes must have its optional MCP dependency installed and permit the configured MCP tools. Channel authentication and delivery remain part of the existing Hermes installation.

## Configuration and credentials

Each connection adds one `mcp_servers.neonexus_<connection-id>` entry:

```yaml
mcp_servers:
  neonexus_example:
    transport: http
    url: http://127.0.0.1:8080/mcp
    headers:
      Authorization: Bearer ${NEONEXUS_ASSISTANT_<encoded-id>_TOKEN}
    connect_timeout: 10
    tool_timeout: 30
```

The credential is written to a uniquely named variable in the same Hermes profile's `.env`. The database retains its digest. The UI, workspace profile metadata and YAML contain no bearer value. Files containing original configuration or credentials use private permissions: mode `0600` on Unix and a protected owner/administrator/SYSTEM ACL on Windows, applied before writing content.

The MCP configuration shape and recursive `${VAR}` expansion were checked against upstream commit `b0ab2e163a50d4e6c36507eba955a6067fde6abc`: the [configuration loader](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/tools/mcp_tool_config.py) loads the profile environment before expanding configured values, and the [HTTP transport](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/tools/mcp_tool_transport.py) consumes the `url` and `headers` fields. Missing variables remain literal references, so an unavailable credential cannot silently become a different credential.

Existing channel/provider values and other MCP entries remain intact. Other headers and timeout customizations on the managed entry are preserved. YAML is parsed and serialized, so comments and formatting can change; byte-for-byte originals are retained in adjacent `.neonexus-backup-<uuid>` files. Unrelated `.env` lines remain unchanged.

## Updates and recovery

Reconnecting an existing connection rotates its credential and invalidates the previous token. Use a separate connection when switching Hermes instances, then revoke the old one. **Revoke access** disables a grant immediately; it does not alter Hermes channels or remove the local MCP entry. Restart or edit Hermes through its normal workflow if you also want to remove that entry.

Configuration shape and transport conflicts are rejected before issuing a grant. Connection writes are serialized with agent start/stop operations. If a later step fails, the new grant is revoked and the original files are restored where safe. A concurrent manual file edit is preserved; retained backups are available for recovery.

Connecting refreshes only the managed configuration fingerprint. It does not accept a changed Python interpreter or Hermes source tree as an approved version. Review those changes through **Agents** before starting.

Regression coverage includes preservation of existing channel and provider settings, credential rotation and redaction, stale-file rollback protection, invalid endpoint rejection, conflicting MCP entries, binding an existing grant to the wrong instance, and maintaining the executable/source fingerprint boundary. These checks use isolated local fixtures and do not contact a conversation platform.
