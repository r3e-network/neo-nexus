.PHONY: check fmt test clippy build release dist smoke purity-smoke quality-smoke native-ui-smoke ci-policy-smoke alert-smoke runtime-smoke json-smoke wallet-smoke launch-pack-smoke readiness-smoke metrics-smoke integrity-smoke report-smoke support-smoke event-smoke config-smoke backup-smoke verify clean

fmt:
	cargo fmt --all

check:
	cargo check

clippy:
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

build:
	cargo build

release:
	cargo build --release

dist: release
	./target/release/neo-nexus --package-release dist
	./target/release/neo-nexus --verify-release-package dist
	./target/release/neo-nexus --verify-release-package-json dist

smoke:
	cargo run -- --self-check
	$(MAKE) purity-smoke
	$(MAKE) quality-smoke
	$(MAKE) native-ui-smoke
	$(MAKE) ci-policy-smoke
	$(MAKE) alert-smoke
	$(MAKE) runtime-smoke
	cargo run -- --rpc-health 127.0.0.1:1
	$(MAKE) json-smoke
	$(MAKE) wallet-smoke
	$(MAKE) launch-pack-smoke
	$(MAKE) readiness-smoke
	$(MAKE) metrics-smoke
	$(MAKE) integrity-smoke
	$(MAKE) report-smoke
	$(MAKE) support-smoke
	$(MAKE) event-smoke
	$(MAKE) config-smoke
	$(MAKE) backup-smoke

runtime-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; runtime="$$tmp_dir/neo-node"; printf '%s\n' '#!/bin/sh' 'echo "neo-rs neo-node version smoke"' 'exit 0' > "$$runtime"; chmod +x "$$runtime"; cargo run -- --runtime-smoke neo-rs "$$runtime" > "$$tmp_dir/runtime-smoke.txt"; grep 'runtime-smoke: passed' "$$tmp_dir/runtime-smoke.txt"; grep 'runtime-binary-sha256:' "$$tmp_dir/runtime-smoke.txt"; cargo run -- --runtime-smoke-json neo-rs "$$runtime" > "$$tmp_dir/runtime-smoke.json"; grep '"status": "passed"' "$$tmp_dir/runtime-smoke.json"; grep '"binary_evidence"' "$$tmp_dir/runtime-smoke.json"; grep '"status": "verified"' "$$tmp_dir/runtime-smoke.json"; grep '"sha256":' "$$tmp_dir/runtime-smoke.json"

json-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; if cargo run -- --rpc-health-json 127.0.0.1:1; then echo "expected rpc-health-json to fail for unreachable endpoint" >&2; exit 1; fi

purity-smoke:
	cargo run -- --source-purity .
	cargo run -- --source-purity-json .

quality-smoke:
	cargo run -- --source-quality src
	cargo run -- --source-quality-json src
	cargo run -- --source-quality tests
	cargo run -- --source-quality-json tests

native-ui-smoke:
	cargo run -- --native-ui-audit .
	cargo run -- --native-ui-audit-json .

ci-policy-smoke:
	cargo run -- --ci-policy .github/workflows/ci.yml
	cargo run -- --ci-policy-json .github/workflows/ci.yml

alert-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; secret="dd123"; target="https://event-management-intake.datadoghq.com/api/v2/events?api_key=$$secret"; cargo run -- --alert-preview datadog "$$target" critical "RPC health unreachable" > "$$tmp_dir/alert.txt"; grep 'alert-preview: ready' "$$tmp_dir/alert.txt"; grep 'provider: datadog' "$$tmp_dir/alert.txt"; grep 'header: DD-API-KEY=<redacted>' "$$tmp_dir/alert.txt"; if grep -q "$$secret" "$$tmp_dir/alert.txt"; then echo "alert preview leaked secret in text output" >&2; exit 1; fi; cargo run -- --alert-preview-json datadog "$$target" critical "RPC health unreachable" > "$$tmp_dir/alert.json"; grep '"status": "ready"' "$$tmp_dir/alert.json"; grep '"provider": "datadog"' "$$tmp_dir/alert.json"; grep '<redacted>' "$$tmp_dir/alert.json"; if grep -q "$$secret" "$$tmp_dir/alert.json"; then echo "alert preview leaked secret in JSON output" >&2; exit 1; fi

wallet-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; wallet="$$tmp_dir/validator.wallet.json"; key="036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0"; printf '%s\n' '{' '  "name": "NeoNexus validator wallet",' '  "version": "3.0",' '  "scrypt": {' '    "n": 16384,' '    "r": 8,' '    "p": 8' '  },' '  "accounts": [' '    {' '      "address": "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq",' '      "label": "validator-1",' '      "isDefault": true,' '      "lock": false,' '      "key": "6PYWB8m1bCnu5bQkRUKAwbZp2BHNvQ3BQRLbpLdTuizpyLkQPSZbtZfoxx",' '      "contract": {' '        "script": "21036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0ac",' '        "parameters": [],' '        "deployed": false' '      },' '      "extra": null' '    }' '  ],' '  "extra": null' '}' > "$$wallet"; cargo run -- --validate-wallet "$$wallet" > "$$tmp_dir/wallet-validation.txt"; grep 'wallet-validation: ok' "$$tmp_dir/wallet-validation.txt"; grep "$$key" "$$tmp_dir/wallet-validation.txt"; grep 'account address matches contract script hash' "$$tmp_dir/wallet-validation.txt"; cargo run -- --validate-wallet-json "$$wallet" > "$$tmp_dir/wallet-validation.json"; grep '"status": "ok"' "$$tmp_dir/wallet-validation.json"; grep '"encrypted_account_count": 1' "$$tmp_dir/wallet-validation.json"; grep '"contract_public_keys"' "$$tmp_dir/wallet-validation.json"; grep "$$key" "$$tmp_dir/wallet-validation.json"; grep '"account-address-contract"' "$$tmp_dir/wallet-validation.json"

launch-pack-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; key="02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; manifest="$$tmp_dir/manifest.json"; printf '%s\n' '{' '  "schema_version": 10,' '  "generated_at_unix": 1800000000,' '  "template": "Single validator",' '  "runtime": "neo-rs",' '  "network": "private",' '  "network_magic": 1230301,' '  "validators_count": 1,' '  "seed_nodes": ["127.0.0.1:30333"],' '  "committee": {' '    "signer_count": 1,' '    "wallet_reference_count": 1,' '    "endpoint_reference_count": 1,' '    "sidecar_command_count": 1,' '    "public_keys": ["'"$$key"'"],' '    "secret_material_policy": "references-only-no-private-keys-or-passwords",' '    "preflight_policy": "check-native-wallet-paths-http-endpoints-and-sidecar-commands",' '    "signers": [' '      {' '        "label": "committee-signer-1",' '        "public_key": "'"$$key"'",' '        "wallet_path": "wallets/validator-1.wallet.json",' '        "signer_endpoint": "http://127.0.0.1:9021",' '        "signer_command_template": "signer-bin/neo-signer --wallet {wallet} --listen {endpoint}",' '        "signer_command": "signer-bin/neo-signer --wallet wallets/validator-1.wallet.json --listen http://127.0.0.1:9021",' '        "signer_command_plan": {' '          "execution_policy": "argv-no-shell",' '          "binary": "signer-bin/neo-signer",' '          "arguments": ["--wallet", "wallets/validator-1.wallet.json", "--listen", "http://127.0.0.1:9021"]' '        }' '      }' '    ]' '  },' '  "secret_provisioning": {' '    "schema_version": 1,' '    "policy": "operator-provided-wallets-no-secret-material-in-launch-pack",' '    "wallet_provisioning_file": "wallet-provisioning.json",' '    "wallet_instructions_file": "wallets/README.md",' '    "recommended_wallet_root": "wallets",' '    "required_wallet_count": 1,' '    "wallet_reference_count": 1,' '    "missing_wallet_reference_count": 0,' '    "generated_secret_count": 0' '  },' '  "scripts": {' '    "runbook": "RUNBOOK.md",' '    "preflight_unix": "preflight-unix.sh",' '    "preflight_windows": "preflight-windows.ps1",' '    "health_unix": "health-unix.sh",' '    "health_windows": "health-windows.ps1",' '    "start_unix": "start-unix.sh",' '    "stop_unix": "stop-unix.sh",' '    "start_windows": "start-windows.ps1",' '    "stop_windows": "stop-windows.ps1"' '  },' '  "artifacts": [],' '  "nodes": []' '}' > "$$manifest"; cargo run -- --launch-pack-sidecars "$$manifest" > "$$tmp_dir/sidecars.txt"; grep 'launch-pack-sidecars: planned' "$$tmp_dir/sidecars.txt"; grep 'signer:committee-signer-1' "$$tmp_dir/sidecars.txt"; cargo run -- --launch-pack-sidecars-json "$$tmp_dir" > "$$tmp_dir/sidecars.json"; grep '"status": "planned"' "$$tmp_dir/sidecars.json"; grep '"sidecar_count": 1' "$$tmp_dir/sidecars.json"; grep '"kind": "sidecar"' "$$tmp_dir/sidecars.json"

readiness-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; cargo run -- --workspace-readiness "$$tmp_dir/neonexus.db"; cargo run -- --workspace-readiness-json "$$tmp_dir/neonexus.db"

metrics-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; cargo run -- --workspace-readiness "$$tmp_dir/neonexus.db"; cargo run -- --workspace-metrics "$$tmp_dir/neonexus.db"; cargo run -- --workspace-metrics-json "$$tmp_dir/neonexus.db"; cargo run -- --workspace-metrics-prometheus "$$tmp_dir/neonexus.db" > "$$tmp_dir/metrics.prom"; grep 'neonexus_workspace_status 1' "$$tmp_dir/metrics.prom"; grep 'neonexus_system_processes' "$$tmp_dir/metrics.prom"

integrity-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; cargo run -- --workspace-readiness "$$tmp_dir/neonexus.db"; cargo run -- --workspace-integrity "$$tmp_dir/neonexus.db"; cargo run -- --workspace-integrity-json "$$tmp_dir/neonexus.db"

report-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; cargo run -- --export-readiness-report "$$tmp_dir/neonexus.db" "$$tmp_dir/reports"; test -f "$$tmp_dir/reports"/workspace-readiness-*.txt; test -f "$$tmp_dir/reports"/workspace-readiness-*.json

support-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; cargo run -- --workspace-readiness "$$tmp_dir/neonexus.db"; cargo run -- --export-support-bundle "$$tmp_dir/neonexus.db" "$$tmp_dir/support"; cargo run -- --export-support-bundle-json "$$tmp_dir/neonexus.db" "$$tmp_dir/support-json"; bundle_dir=$$(find "$$tmp_dir/support" -maxdepth 1 -type d -name 'neo-nexus-support-bundle-*' | head -1); test -n "$$bundle_dir"; test -f "$$bundle_dir/metrics.txt"; test -f "$$bundle_dir/metrics.json"; test -f "$$bundle_dir/metrics.prom"; find "$$tmp_dir/support" -maxdepth 1 -type f -name 'neo-nexus-support-bundle-*.zip' | grep .; bundle_json_dir=$$(find "$$tmp_dir/support-json" -maxdepth 1 -type d -name 'neo-nexus-support-bundle-*' | head -1); test -n "$$bundle_json_dir"; test -f "$$bundle_json_dir/metrics.txt"; test -f "$$bundle_json_dir/metrics.json"; test -f "$$bundle_json_dir/metrics.prom"; find "$$tmp_dir/support-json" -maxdepth 1 -type f -name 'neo-nexus-support-bundle-*.zip' | grep .

event-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; cargo run -- --workspace-readiness "$$tmp_dir/neonexus.db"; cargo run -- --export-event-journal "$$tmp_dir/neonexus.db" "$$tmp_dir/events"; test -f "$$tmp_dir/events"/event-journal-*.txt; test -f "$$tmp_dir/events"/event-journal-*.json

config-smoke:
	@set -e; \
	tmp_dir=$$(mktemp -d); \
	trap 'rm -rf "$$tmp_dir"' EXIT; \
	backup_file="$$tmp_dir/neonexus-config-smoke.json"; \
	printf '%s\n' \
		'{' \
		'  "schema_version": 5,' \
		'  "application": "NeoNexus",' \
		'  "application_version": "smoke",' \
		'  "exported_at_unix": 1800000000,' \
		'  "workspace_settings": [],' \
		'  "runtime_catalog_profiles": [],' \
		'  "runtime_signer_profiles": [],' \
		'  "fast_sync_snapshots": [],' \
		'  "nodes": [' \
		'    {' \
		'      "id": "smoke-neo-rs",' \
		'      "name": "smoke neo-rs",' \
		'      "node_type": "neo-rs",' \
		'      "network": "testnet",' \
		'      "binary_path": "/usr/local/bin/neo-node",' \
		'      "args": [],' \
		'      "runtime_version": "v0.8.0",' \
		'      "storage_engine": "rocksdb",' \
		'      "rpc_port": 10332,' \
		'      "p2p_port": 10333,' \
		'      "ws_port": 10334,' \
		'      "status": "stopped",' \
		'      "pid": null,' \
		'      "plugins": []' \
		'    }' \
		'  ],' \
		'  "events": []' \
		'}' > "$$backup_file"; \
	cargo run -- --import-backup "$$tmp_dir/neonexus.db" "$$backup_file"; \
	cargo run -- --export-node-configs "$$tmp_dir/neonexus.db" "$$tmp_dir/configs"; \
	cargo run -- --export-node-configs-json "$$tmp_dir/neonexus.db" "$$tmp_dir/configs-json"; \
	config_file="$$tmp_dir/configs/nodes/smoke-neo-rs/smoke-neo-rs-neo-rs-config.toml"; \
	test -f "$$tmp_dir/configs"/node-config-export-*.txt; \
	test -f "$$tmp_dir/configs"/node-config-export-*.json; \
	test -f "$$config_file"; \
	test -f "$$tmp_dir/configs-json"/node-config-export-*.json; \
	cargo run -- --generate-node-config neo-rs testnet rocksdb 20332 20333 "$$tmp_dir/generated-neo-rs.toml"; \
	cargo run -- --generate-node-config-json neo-rs testnet rocksdb 20342 20343 "$$tmp_dir/generated-neo-rs-json.toml" > "$$tmp_dir/generated-neo-rs.json"; \
	test -f "$$tmp_dir/generated-neo-rs.toml"; \
	test -f "$$tmp_dir/generated-neo-rs-json.toml"; \
	grep '"status": "ready"' "$$tmp_dir/generated-neo-rs.json"; \
	grep '"node_type": "neo-rs"' "$$tmp_dir/generated-neo-rs.json"; \
	cargo run -- --validate-node-config neo-rs testnet rocksdb 10332 10333 "$$config_file"; \
	cargo run -- --validate-node-config-json neo-rs testnet rocksdb 10332 10333 "$$config_file"

backup-smoke:
	@set -e; tmp_dir=$$(mktemp -d); trap 'rm -rf "$$tmp_dir"' EXIT; cargo run -- --workspace-readiness "$$tmp_dir/neonexus.db"; cargo run -- --export-backup "$$tmp_dir/neonexus.db" "$$tmp_dir/backups"; test -f "$$tmp_dir/backups"/neonexus-backup-*.json; cargo run -- --workspace-readiness "$$tmp_dir/neonexus-json.db"; cargo run -- --export-backup-json "$$tmp_dir/neonexus-json.db" "$$tmp_dir/backups-json"; test -f "$$tmp_dir/backups-json"/neonexus-backup-*.json; backup_file="$$tmp_dir/neonexus-backup-1800000000.json"; printf '%s\n' '{' '  "schema_version": 5,' '  "application": "NeoNexus",' '  "application_version": "smoke",' '  "exported_at_unix": 1800000000,' '  "workspace_settings": [],' '  "runtime_catalog_profiles": [],' '  "runtime_signer_profiles": [],' '  "fast_sync_snapshots": [],' '  "nodes": [],' '  "events": []' '}' > "$$backup_file"; cargo run -- --validate-backup "$$backup_file"; cargo run -- --validate-backup-json "$$backup_file"; cargo run -- --import-backup "$$tmp_dir/import.db" "$$backup_file"; cargo run -- --import-backup-json "$$tmp_dir/import-json.db" "$$backup_file"

verify:
	cargo fmt --all --check
	cargo check
	cargo clippy --all-targets -- -D warnings
	cargo test
	cargo run -- --self-check
	$(MAKE) purity-smoke
	$(MAKE) quality-smoke
	$(MAKE) native-ui-smoke
	$(MAKE) ci-policy-smoke
	$(MAKE) alert-smoke
	$(MAKE) runtime-smoke
	cargo run -- --rpc-health 127.0.0.1:1
	$(MAKE) json-smoke
	$(MAKE) wallet-smoke
	$(MAKE) launch-pack-smoke
	$(MAKE) readiness-smoke
	$(MAKE) metrics-smoke
	$(MAKE) integrity-smoke
	$(MAKE) report-smoke
	$(MAKE) support-smoke
	$(MAKE) event-smoke
	$(MAKE) config-smoke
	$(MAKE) backup-smoke
	cargo build --release
	./target/release/neo-nexus --package-release dist
	./target/release/neo-nexus --verify-release-package dist
	./target/release/neo-nexus --verify-release-package-json dist

clean:
	cargo clean
