#!/usr/bin/env bash
# NeoNexus entry point for the sibling service-owned compatibility harness.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
nexus_root="$(cd -- "$script_dir/.." && pwd -P)"

if [[ -n "${NEO_OS_SERVICES_DIR:-}" ]]; then
  services_candidate="$NEO_OS_SERVICES_DIR"
else
  services_candidate=""
  services_candidates=(
    "$nexus_root/../neo-os-services"
    "$nexus_root/../neo-os/neo-os-services"
  )
  for candidate in "${services_candidates[@]}"; do
    if [[ -f "$candidate/scripts/run-neo-nexus-signer-contract.sh" ]]; then
      services_candidate="$candidate"
      break
    fi
  done
fi

if [[ -z "$services_candidate" || ! -f "$services_candidate/scripts/run-neo-nexus-signer-contract.sh" ]]; then
  echo "neo-os-services was not found beside neo-nexus or under ../neo-os; set NEO_OS_SERVICES_DIR to its checkout" >&2
  exit 2
fi
services_root="$(cd -- "$services_candidate" && pwd -P)"

NEO_NEXUS_DIR="$nexus_root" \
  bash "$services_root/scripts/run-neo-nexus-signer-contract.sh"
