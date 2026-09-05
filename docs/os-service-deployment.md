# OS service deployment

NeoNexus should run under a service manager in unattended deployments. The
repository includes templates, but installing a service is host administration
and is not performed by the build or test suite.

## Linux/systemd

1. Create a dedicated `neo-nexus` service account and an owned data directory.
2. Install the release binary at `/opt/neo-nexus/neo-nexus`.
3. Review `deploy/systemd/neo-nexus.service` and run:

```sh
sudo PREFIX=/opt/neo-nexus DATA_DIR=/var/lib/neo-nexus \
  deploy/systemd/install.sh
sudo systemctl start neo-nexus.service
sudo systemctl status neo-nexus.service
```

The unit has `Restart=on-failure`, a bounded stop timeout, a private temporary
directory, `NoNewPrivileges`, `ProtectHome`, and a restricted writable data
path. `NEONEXUS_WEB_TOKEN`, signer credentials and other secrets are **not** in
the unit. Put them in a root-owned, service-readable `EnvironmentFile` or use
the deployment's secret provider; never commit them to this repository.

Place a TLS reverse proxy in front of the loopback listener for remote access.
The service's `/healthz` now reports both supervision and notification-worker
liveness, plus controller pending/stale/unknown counts.

## Windows

Run the elevated PowerShell installer:

```powershell
.\deploy\windows\install-service.ps1 `
  -BinaryPath 'C:\Program Files\NeoNexus\neo-nexus.exe'
sc.exe start NeoNexus
sc.exe query NeoNexus
```

The installer creates an auto-start service with restart actions. It does not
accept tokens or signer secrets. Configure those through the service account's
approved environment/secret mechanism. The Windows process-group/CTRL_BREAK
implementation is in the supervisor; the service install and a two-node
process-group acceptance run must still be executed on a real Windows host.

## Acceptance boundary

A successful service installation proves only that the OS can restart the
workbench. It does not prove five upstream clients reach consensus, that a
remote signer signs a block, or that a chain database is compatible with a
new version. Run `scripts/client-acceptance-matrix.sh` with the five real
client binaries and a live network before granting unattended production
ownership.
