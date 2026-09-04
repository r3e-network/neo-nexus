# Manage an existing Hermes agent

NeoNexus supervises an installed Nous Research Hermes gateway and gives it scoped
access to node monitoring and lifecycle tools. **Agents** manages the process;
**Assistants** manages its node permissions and MCP connection. Telegram, other
conversation channels, allowed conversation users and the model provider remain
configured in Hermes itself.

The upstream contracts cited here were reviewed at commit
`b0ab2e163a50d4e6c36507eba955a6067fde6abc`. This is a compatibility reference,
not a claim that every future Hermes release has the same behavior.

## Register, authorize and start

1. Install Hermes and configure the intended profile through its own setup
   workflow. The reviewed [upstream installation guide](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/README.md#quick-install)
   covers Linux and native Windows, among other platforms. NeoNexus does not
   install Python dependencies or provision a Telegram bot. Stop the profile's
   existing gateway before handing its supervision to NeoNexus.
2. In **Agents → Register companion**, select **Hermes** and supply the following
   fields. Use paths visible to the machine and OS running NeoNexus.

   | Field | Value |
   | --- | --- |
   | Name | A recognizable name for this Hermes instance |
   | Declared version | The installed release or commit you reviewed |
   | Executable | Absolute path to that installation's Python interpreter, such as `.venv/Scripts/python.exe` on Windows or `.venv/bin/python` on Linux |
   | Working directory | Absolute Hermes source root containing `hermes_cli/main.py` |
   | Arguments | `[]`; NeoNexus supplies the gateway arguments |
   | Configuration | Absolute path to the intended profile's `config.yaml` |
   | Health URL | Leave empty for Hermes; its local gateway state is used |
   | Automatic restart | Enable if bounded recovery is desired |

   Save the stopped profile. The optional node association organizes the process;
   it does not grant assistant access to that node.
3. Open **Assistants → Connect Hermes**, select the registered instance and name
   the connection. Select individual nodes, or explicitly select **All current
   and future nodes**. Leave operations unchecked for monitoring only; enable
   **Allow node start, stop and restart** when those operations are intended.
4. Enter the `/mcp` address reachable from the Hermes process, for example
   `http://127.0.0.1:8080/mcp`. Use the actual NeoNexus port. HTTPS or loopback HTTP
   is accepted; credentials in the URL, query parameters and fragments are not.
   Choose **Connect and save permissions** while Hermes is stopped.
5. Return to **Agents** and choose **Start**. Use the existing Hermes conversation
   channel to query the authorized nodes. Hermes needs its optional MCP dependency
   and the relevant tools enabled. Saving a connection does not establish or
   verify a live Hermes MCP handshake, LLM session or Telegram delivery.

See [Connect an existing Hermes assistant](hermes-assistants.md) for the generated
MCP configuration and detailed connection rollback behavior.

## Profile isolation and configuration changes

The managed foreground command is:

```text
<configured-python> -m hermes_cli.main gateway run --external-supervisor
```

The child receives `HERMES_HOME` equal to the parent of its configured
`config.yaml`, plus `HERMES_GATEWAY_EXTERNAL_SUPERVISOR=1`,
`HERMES_SUPERVISED_CHILD=1` and `PYTHONUNBUFFERED=1`. These variables are set on
that child process, not globally. This prevents Hermes's sticky active-profile
selection from redirecting the gateway to another home; see the reviewed
[profile-selection logic](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/hermes_cli/main.py#L464).
Give separate Hermes profiles separate homes.

Saving an Agent pins its Python executable, the selected Hermes source files and
`config.yaml` by SHA-256. A later managed start refuses changed code or
configuration until the stopped profile is reviewed and saved again. The source
fingerprint covers Python files in the adapter's known source directories and
`pyproject.toml`; it is not verification of every virtualenv dependency or an
upstream release signature. The declared version is operator-supplied metadata.

Upgrade Hermes using its own installation workflow while stopped, review the
release and configuration changes, then use **Edit / review version** and save
before starting. Connecting an Assistant refreshes only the configuration
fingerprint; it does not approve a changed executable or source tree. Hermes's
`.env` is not part of the Agent's configuration fingerprint.

## Credentials and node tools

Connection setup writes a generated bearer token into a dedicated
`NEONEXUS_ASSISTANT_<encoded-id>_TOKEN` variable in the profile's `.env`. The managed
YAML entry contains an environment reference, and the database stores only a
token digest. The token is not shown in the web form or put in process arguments.
Hermes [loads the profile environment and expands references](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/tools/mcp_tool_config.py#L210)
before its [HTTP transport](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/tools/mcp_tool_transport.py#L366)
uses the configured authorization header.

Setup preserves unrelated configuration values and `.env` lines and retains
adjacent backups. YAML formatting and comments may change during serialization.
The live token is stored in `.env`; retained environment backups can contain
older credentials and other existing secrets. Files written by this setup use
mode `0600` on Unix or a protected owner/Administrators/SYSTEM ACL on Windows.

The `/mcp` endpoint uses an Assistant bearer credential, independently of browser
sessions and the workbench operator token. Its current tools are:

| Permission | Tools | Returned data or effect |
| --- | --- | --- |
| Monitoring | `nodes_list`, `node_status` | Authorized node identities, versions, lifecycle state and the last recorded RPC observation |
| Monitoring | `node_logs`, `node_events` | Bounded, redacted log tails and node event history |
| Monitoring | `node_plugins`, `node_config_conflicts` | Plugin activation/versions and conflict metadata; no raw configuration file contents |
| Operations | `node_start`, `node_stop`, `node_restart` | The same readiness checks, configuration guards and supervised lifecycle used by the workbench |

Tools act only within the grant's node scope. They do not provide wallet export,
private keys, signer credentials, arbitrary RPC methods or arbitrary shell
commands. Configuration conflicts are resolved in the workbench. RPC observations
are stored samples: check `checked_at_unix` rather than treating every answer as
a new live probe.

**Review / reconnect** rotates the credential when permissions are saved.
**Revoke access** disables the grant without stopping Hermes or changing its
conversation channels. Requests and queued lifecycle operations recheck the
grant; operations that already entered process control can finish. Revocation
cannot retract data already returned. The old local MCP entry remains until
reconnected or removed through the stopped Hermes profile's configuration.

## Shutdown and bounded recovery

**Stop** first checks the recorded executable and OS process start time. A missing
process is settled; a reused or unverifiable PID is not signalled. **Clear stale
PID** explicitly removes an absent or mismatched record and disables its pending
recovery, without signalling an OS process. It refuses a still-matching live
process.

For Hermes, NeoNexus first runs the configured Python interpreter with
`-m hermes_cli.main gateway stop` in the same source directory and isolated home.
It does not pass `--force` or `--all`. Upstream writes the profile's
`.gateway-planned-stop.json` marker with its own process identity information,
and the gateway watches that marker. This is especially relevant on native
Windows, where Hermes does not install the generic SIGBREAK handler needed for
CTRL_BREAK alone to prove graceful shutdown. See the upstream
[planned-stop marker](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/gateway/status.py#L1419)
and [gateway stop watcher](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/gateway/run.py#L5208).

The helper gets up to 20 seconds, with its stdout/stderr discarded. NeoNexus
checks that the target actually exited; a successful helper exit alone is
insufficient. If the helper completes or times out while the target remains alive,
normal supervised termination follows, including the platform grace period and
force-stop fallback. Identity errors
stop the operation instead of falling back against an unrelated PID. A forced
stop is recorded as such.

Agent recovery allows at most three retries, delayed by 5, 10 and 20 seconds.
The desired-running flag, retry count and next attempt are persisted, so restarting
NeoNexus does not reset this budget. Manual Start or saving a stopped Agent resets
it; Stop cancels pending recovery. The following distinctions follow Hermes's
[restart exit-code contract](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/gateway/restart.py):

| Observed exit | Recorded state and recovery |
| --- | --- |
| `0` | Stopped; no automatic restart |
| `75` | Planned restart; retry only when automatic restart and the remaining budget allow it |
| `78` | Error; fatal configuration failure disables automatic restart |
| Other failure or an externally managed process disappears | Crashed; bounded recovery when enabled |
| Recorded PID belongs to another process | Error; automatic restart blocked and PID retained for review |

## Health, auditing and validation limits

Running means that a process is present. Hermes health is a separate observation
of `HERMES_HOME/gateway_state.json`, tied to the recorded PID, profile and timestamp.
The reader rejects oversized or malformed state, snapshots older than 120 seconds
and timestamps more than 30 seconds in the future. Missing, stale or unattributable
state appears as **Not verified**, not Healthy. Idle Hermes gateways can retain an
old timestamp, so stale telemetry is not itself evidence of a crash. A fresh
running snapshot can still be Unhealthy when its current platform adapters or
session store report failure; see the upstream
[gateway state writer](https://github.com/NousResearch/hermes-agent/blob/b0ab2e163a50d4e6c36507eba955a6067fde6abc/gateway/status.py#L795).
A change to Unhealthy generates an event; it does not by itself force a restart.
Transitions to Not verified do not emit a health-change alert.

Agent lifecycle events and changes to a verified health result enter the runtime
journal and use the configured alert routing policy. Assistant tools record intent before executing and then
record completion, failure or denial. A failed intent write prevents execution;
if recording the outcome fails, the tool response reports
`_meta.operationJournaled=false`. This is local operational history, not a
tamper-proof audit trail or a transcript of the assistant's reasoning. Webhook
delivery is bounded and can fail or be duplicated during recovery.

Logs are available through **Agents → Logs** and scoped node tools. Redaction
filters known secret patterns; it is not a guarantee that arbitrary upstream log
output contains no sensitive material. Treat returned logs as untrusted data,
not instructions to the assistant.

Validation used isolated configuration/MCP fixtures, local HTTP servers and real
owned child processes on Windows and Ubuntu. It covered targeted Windows
shutdown and force fallback, Linux termination, helper timeouts, PID identity,
health parsing, bounded recovery and alert replay. **A real Hermes gateway with
an LLM provider and a Telegram conversation was not exercised.** These checks do
not establish channel delivery, model behavior, every installed dependency or
compatibility with an unreviewed Hermes release.
