#!/usr/bin/env bash
# Install the NeoNexus systemd unit without placing credentials in the unit.
set -Eeuo pipefail

PREFIX=${PREFIX:-/opt/neo-nexus}
UNIT_SOURCE=${UNIT_SOURCE:-deploy/systemd/neo-nexus.service}
UNIT_TARGET=${UNIT_TARGET:-/etc/systemd/system/neo-nexus.service}
DATA_DIR=${DATA_DIR:-/var/lib/neo-nexus}
SERVICE_USER=${SERVICE_USER:-neo-nexus}

[[ ${EUID:-$(id -u)} -eq 0 ]] || { printf 'run as root\n' >&2; exit 2; }
[[ -r "$UNIT_SOURCE" ]] || { printf 'unit template is not readable: %s\n' "$UNIT_SOURCE" >&2; exit 2; }

# Refuse accidental credential leakage in a locally modified template.
if grep -Eq 'Environment=(NEONEXUS_)?(SIGNER|WEB_TOKEN).*=' "$UNIT_SOURCE"; then
  printf 'refusing a unit containing inline credentials; use EnvironmentFile or a secret provider\n' >&2
  exit 2
fi

install -d -m 0750 -o "$SERVICE_USER" -g "$SERVICE_USER" "$PREFIX" "$DATA_DIR" "$(dirname "$UNIT_TARGET")"
install -m 0644 "$UNIT_SOURCE" "$UNIT_TARGET"
systemctl daemon-reload
systemctl enable neo-nexus.service
printf 'installed %s; start with: systemctl start neo-nexus.service\n' "$UNIT_TARGET"
