#!/usr/bin/env bash
# Run the real-client acceptance matrix described in docs/client-acceptance.md.
#
# This script is intentionally conservative: it never invents credentials,
# never starts a node without an explicit workspace registration, and records a
# failed/blocked row rather than treating a missing binary or RPC endpoint as a
# pass. Credentials remain in the node/client environment and secret files.
set -Eeuo pipefail

usage() {
  printf 'usage: %s <matrix.json> [neo-nexus-binary] [report.json]\n' "$0" >&2
  exit 2
}

[[ $# -ge 1 && $# -le 3 ]] || usage
CONFIG=$1
NEONEXUS=${2:-neo-nexus}
REPORT=${3:-acceptance-report.json}

command -v jq >/dev/null || { printf 'jq is required\n' >&2; exit 2; }
[[ -r "$CONFIG" ]] || { printf 'matrix file is not readable: %s\n' "$CONFIG" >&2; exit 2; }
[[ -x "$NEONEXUS" || -n "$(command -v "$NEONEXUS" 2>/dev/null || true)" ]] || {
  printf 'neo-nexus binary is not executable or on PATH: %s\n' "$NEONEXUS" >&2
  exit 2
}

DB=$(jq -er '.db' "$CONFIG")
SOAK=$(jq -er '.soak_seconds // 3600 | numbers | select(. >= 0)' "$CONFIG")
CLIENT_COUNT=$(jq -er '.clients | length' "$CONFIG")
EXPECTED_TYPES='["neo-cli","neo-go","neo-rs","neox-geth","neox-rs"]'
ACTUAL_TYPES=$(jq -c '[.clients[].type] | sort' "$CONFIG")
if [[ "$ACTUAL_TYPES" != "$EXPECTED_TYPES" ]]; then
  printf 'matrix must contain exactly neo-cli, neo-go, neo-rs, neox-geth, neox-rs; got %s\n' "$ACTUAL_TYPES" >&2
  exit 2
fi
mkdir -p "$(dirname "$REPORT")" 2>/dev/null || true

TMP=$(mktemp -d "${TMPDIR:-/tmp}/neo-nexus-acceptance.XXXXXX")
trap 'rm -rf "$TMP"' EXIT

# JSON-string escaping is delegated to jq; command output is bounded and treated
# as untrusted evidence, never interpolated back into a shell command.
rows='[]'
add_row() {
  local client=$1 criterion=$2 status=$3 detail=$4
  rows=$(jq -c --arg client "$client" --arg criterion "$criterion" \
    --arg status "$status" --arg detail "$detail" \
    '. + [{client:$client,criterion:$criterion,status:$status,detail:$detail}]' <<<"$rows")
}

for index in $(seq 0 $((CLIENT_COUNT - 1))); do
  type=$(jq -er ".clients[$index].type" "$CONFIG")
  binary=$(jq -er ".clients[$index].binary" "$CONFIG")
  endpoint=$(jq -er ".clients[$index].endpoint" "$CONFIG")
  family=$(jq -er ".clients[$index].family" "$CONFIG")
  name=$(jq -er ".clients[$index].name // \"acceptance-$type\"" "$CONFIG")

  if [[ ! -x "$binary" ]]; then
    add_row "$type" identity-smoke blocked "binary is not executable: $binary"
    add_row "$type" managed-lifecycle blocked "identity smoke is blocked"
    add_row "$type" long-run-soak blocked "identity smoke is blocked"
    add_row "$type" cross-version-upgrade blocked "identity smoke is blocked"
    add_row "$type" kill-recovery blocked "identity smoke is blocked"
    continue
  fi

  if smoke=$($NEONEXUS --runtime-smoke "$type" "$binary" 2>&1); then
    add_row "$type" identity-smoke passed "$(tr '\n' ' ' <<<"$smoke" | cut -c1-1000)"
  else
    add_row "$type" identity-smoke failed "$(tr '\n' ' ' <<<"$smoke" | cut -c1-1000)"
    add_row "$type" managed-lifecycle blocked "identity smoke failed"
    add_row "$type" long-run-soak blocked "identity smoke failed"
    add_row "$type" cross-version-upgrade blocked "identity smoke failed"
    add_row "$type" kill-recovery blocked "identity smoke failed"
    continue
  fi

  if rpc=$($NEONEXUS --rpc-health-json "$endpoint" "$family" 2>&1); then
    rpc_status=$(jq -r '.report.status // "unknown"' <<<"$rpc")
    if [[ "$rpc_status" == healthy ]]; then
      add_row "$type" rpc-health passed "status=$rpc_status"
    else
      add_row "$type" rpc-health failed "status=$rpc_status"
    fi
  else
    add_row "$type" rpc-health failed "$(tr '\n' ' ' <<<"$rpc" | cut -c1-1000)"
  fi

  # Consensus/signing boundary: governance reads and designation checks (N3 only)
  # This is Row 4 from docs/client-acceptance.md - requires consensus node configuration
  if [[ "$family" == "neo-n3" ]]; then
    # Governance read: committee, validators, candidate votes
    if gov=$($NEONEXUS --governance-json "$endpoint" 2>&1); then
      add_row "$type" consensus-governance passed "committee/validators snapshot captured"
    else
      add_row "$type" consensus-governance failed "$(tr '\n' ' ' <<<"$gov" | cut -c1-1000)"
    fi

    # Designation check for state-validator role (non-fatal if not designated)
    if desc=$($NEONEXUS --designation-json "$endpoint" state-validator 2>&1); then
      designator_status=$(echo "$desc" | jq -r '.designated // false')
      if [[ "$designator_status" == "true" ]]; then
        add_row "$type" consensus-designation passed "designated for state-validator"
      else
        add_row "$type" consensus-designation manual "node exists but not designated as state-validator (requires validator selection)"
      fi
    else
      add_row "$type" consensus-designation manual "cannot query designation chain state may be syncing"
    fi
  else
    # Neo X chains do not use N3 governance model
    add_row "$type" consensus-governance skipped "neo-x does not use getcommittee"
    add_row "$type" consensus-designation skipped "neo-x does not use RoleManagement"
  fi

  # Lifecycle/upgrade/recovery require the node to be explicitly registered in
  # the supplied database. The script does not invent node-editor fields.
  if status=$($NEONEXUS --node-status "$DB" "$name" 2>&1); then
    add_row "$type" managed-lifecycle passed "registered node: $(tr '\n' ' ' <<<"$status" | cut -c1-900)"
    add_row "$type" kill-recovery manual "run the kill/recovery step against PID in node-status"
  else
    add_row "$type" managed-lifecycle blocked "node '$name' is not registered in $DB"
    add_row "$type" kill-recovery blocked "lifecycle is blocked"
  fi

  if (( SOAK == 0 )); then
    add_row "$type" long-run-soak skipped "soak_seconds is zero"
  else
    add_row "$type" long-run-soak manual "run for ${SOAK}s with 30s RPC polling; this script does not background a production node"
  fi

  target_version=$(jq -er ".clients[$index].upgrade_version // empty" "$CONFIG" 2>/dev/null || true)
  if [[ -n "$target_version" ]]; then
    if upgrade=$($NEONEXUS --release-transaction "$DB" "$name" "$target_version" 2>&1); then
      add_row "$type" cross-version-upgrade passed "$(tr '\n' ' ' <<<"$upgrade" | cut -c1-1000)"
    else
      add_row "$type" cross-version-upgrade failed "$(tr '\n' ' ' <<<"$upgrade" | cut -c1-1000)"
    fi
  else
    add_row "$type" cross-version-upgrade manual "set clients[$index].upgrade_version to an installed verified release"
  fi
done

failed=$(jq '[.[] | select(.status == "failed" or .status == "blocked")] | length' <<<"$rows")
manual=$(jq '[.[] | select(.status == "manual")] | length' <<<"$rows")
jq -n --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg db "$DB" --argjson soak_seconds "$SOAK" --argjson clients "$CLIENT_COUNT" \
  --argjson rows "$rows" --argjson failed "$failed" --argjson manual "$manual" \
  '{schema_version:1,generated_at,db,soak_seconds,client_count:clients,failed_or_blocked:failed,manual_rows:manual,success:(failed == 0 and manual == 0),rows}' \
  >"$REPORT"

jq . "$REPORT"
(( failed == 0 && manual == 0 ))
