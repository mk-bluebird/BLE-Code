#!/bin/bash
# tools/seed_catalog.sh
# Seed the catalog with initial metadata for quick AI orientation

set -euo pipefail

sqlite3 repo_index.db <<EOF
UPDATE files SET description = 'BLE profile schema with RoH invariants' WHERE path = 'schemas/ble-profile.schema.v1.aln';
UPDATE files SET description = 'Per-session SKO schema with KEREW metrics' WHERE path = 'schemas/ble-session.v1.schema.json';
UPDATE files SET description = 'Nordic UART secure scan/connect/subscribe playbook' WHERE path = 'playbooks/nordic-uart-scan-connect-subscribe.ble-playbook.aln';
UPDATE files SET description = 'OpenBCI Cyton over NUS BCI playbook with passthrough' WHERE path = 'playbooks/openbci-cyton-over-nus.ble-playbook.aln';
UPDATE files SET description = 'BCI telemetry data schema' WHERE path = 'schemas/bci-telemetry.v1.schema.json';
UPDATE files SET description = 'Perplexity BLE Guard v1 profile' WHERE path = 'schemas/perplexity-ble-guard-v1.profile.aln';
UPDATE files SET description = 'Workspace root configuration' WHERE path = './Cargo.toml';
UPDATE files SET description = 'Repository governance policy' WHERE path = './repo-governance.aln';
UPDATE files SET description = 'AI contribution policy' WHERE path = './AiContributionPolicy.aln';
EOF

echo "Catalog seeded with initial metadata."
