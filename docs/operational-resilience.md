# Operational resilience

The workbench supervises node processes independently of Hermes. An assistant or
RPC outage does not disable the local guardian. The controls below distinguish
process failure, missing observations, wrong-chain connections and host pressure.

## Host capacity

**Monitor → Storage and memory** shows available memory and storage capacity.
**Settings → Host resource alerts** controls the interval, thresholds and up to
16 extra absolute storage-directory paths. The workspace directory is always
included. Add external chain-data mounts explicitly; arguments such as a custom
datadir are not guessed or scanned recursively.

Defaults are a 30-second interval, disk warning at 5120 MiB available and critical
at 1024 MiB, memory warning at 10% available and critical at 5%. These are initial
operator settings, not universal requirements for every blockchain database.
Adjust them for database growth, snapshot expansion and compaction headroom.

Critical observations alert immediately. Warning and recovery transitions need
two consecutive observations. Confirmation state and the last observation are
stored together with the event in a database transaction; unchanged pressure does
not send a warning on every tick or workbench restart. Old snapshots are marked
stale. A failed sample is Unknown, never zero capacity or Healthy.

Storage statistics are queried on the selected directory, following the actual
target filesystem instead of inferring its mount from a string prefix. Windows
uses available capacity for the calling account, including quotas; Unix checks
available blocks and also detects read-only filesystems and exhausted inodes
when the filesystem reports those values. See [GetDiskFreeSpaceExW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdiskfreespaceexw)
and [statvfs](https://man7.org/linux/man-pages/man3/statvfs.3.html).

Memory observations describe the host values exposed by the OS; they do not
measure a container's separate cgroup limit. CPU saturation, per-node cache
budgets and swap thrashing require separate investigation. No resource alert
automatically deletes chain data, trims logs, changes a cache or stops a node.

Filesystem sampling runs in one background worker. A slow or disconnected mount
does not block process supervision. After ten seconds, a warning reports the
sampling failure. Only one outstanding sample is allowed, so repeated timeouts
cannot spawn an unlimited number of threads. Late results are discarded; an
unrecoverably blocked filesystem may require the mount or workbench to recover.

The Prometheus endpoint includes `neonexus_resource_sample_fresh`, the observation
timestamp, pressure and available/capacity byte gauges. Stale/absent samples omit
capacity gauges. The `fleet_resources` MCP tool is available only to assistants
with all-node scope; it returns capacity and freshness without filesystem paths.
This does not add or configure a Hermes conversation channel.

Resource policy is included in reference-only backups. Observations, pending
samples and alert confirmation state are operational data and are excluded.

## Failure recovery and chain observations

The recovery ledger records attempted restarts before executing them. Restarting
the workbench must not replenish an exhausted budget or lose a delayed retry.
Stopping a node cancels its pending recovery. Disabling the policy cancels
pending work while preserving the number of attempts already consumed.

RPC health is checked against the configured public chain identity, not just a
successful HTTP response. Neo N3 network magic and NeoX chain ID are different
contracts. Private-network identity is displayed as observed/unverified; the
probe does not infer an expected identity from private deployment files.
A private node with zero peers can be
intentional; a public node with zero peers warrants investigation.

Missing or historical RPC observations must not be presented as current health.
Check the sample time and process identity. Block progress alerts require fresh,
comparable observations on a public network. They report the symptom and do not
automatically restart a node that might be importing state or waiting for peers.

## Restore review

A configuration backup is not a running blockchain database snapshot. Restore
must preserve ownership of live node and companion processes, validate referenced
wallets and signer trust, and apply the complete import in one transaction.
Stop affected instances and review changes before restoring their configuration.
Secret values in arguments or authenticated URLs do not belong in a reference-only
backup; provision them through the client's supported local credential files.

Node and companion starts reserve their state before spawning. An agent's
`Starting` intent also blocks restore of its associated node while launch arguments
are being prepared. A stale controller cannot claim the same agent. If the
workbench disappears during this narrow launch interval before a PID is recorded,
the pending intent remains visible and prevents an automatic duplicate launch;
inspect the host process and use Stop to clear the intent before retrying.

## Further deployment work

Production acceptance still needs the actual client versions and networks. The
next substantial areas are full multi-file release deployment with rollback,
bounded process-log retention with reliable stream reopening, OS-service setup
for the workbench, and native consensus-signer adapters. Those are separate from
the health and recovery features above; no live network consensus or Telegram
delivery is claimed by the local regression tests.

## Verification of this batch

On 2026-09-05, Windows `cargo test --all-targets` passed 765 tests. Three ignored
entries comprise the subprocess fixture and two contracts requiring an external
signer deployment. The fixture is launched by lifecycle tests; the external
contracts were not executed in this batch. CI now runs all test targets so new
standalone integration suites are not silently omitted.

Windows and WSL Ubuntu passed the real-child launch-persistence tests. Ubuntu
also passed supervision and native filesystem resource-sampling regressions.
Both platforms passed all-target Clippy with warnings denied. Formatting, source
purity, source quality, CI policy, the binary self-check, `cargo audit` and the
commit-range secret scan passed.

These checks cover fixtures, transaction fault injection and process control.
They do not establish long-running live-network acceptance of every upstream
node release, consensus signing, cgroup resource accounting or Telegram delivery.
