# AI Agent Prompt Template for BLE‑Code (Perplexity)

Use this template whenever an AI agent is asked to modify or extend BLE‑Code for the Perplexity BLE guard profile.

## 1. Context and identity

- Repository: `BLE‑Code` (non‑actuating core only).
- Subject: Perplexity integration for BLE guard.
- Perplexity subject ID: `perplexity-ble-guard-v1`.
- Perplexity profile file: `config/perplexity-ble-guard-v1.profile.aln`.
- Perplexity usage doc: `docs/perplexity-ble-guard-usage.md`.
- Perplexity example telemetry: `examples/perplexity-ble-guard-telemetry*.json`.

Constraints:

- Non‑actuating only: no code that opens real BLE sockets or talks to hardware.
- All behavior must respect neurorights ceilings and SKO governance already encoded in the profile and guard crates.[file:19]
- All changes must be explainable, auditable, and catalog‑aligned.

## 2. Mandatory workflow steps

When the agent receives a BLE‑Code request, it MUST follow these steps in order:

1. **Check the catalog (`repo_index.db`)**

   - Connect read‑only to `repo_index.db`.
   - Discover relevant schemas, playbooks, and crates:

     - `perplexity-ble-guard-v1.profile.aln` schema and references.
     - BLE guard crates, tools, and examples that already exist.

   - Confirm that any new code reuses existing schemas and crates rather than inventing new ones.[file:13]

2. **Load the Perplexity profile**

   - Open `config/perplexity-ble-guard-v1.profile.aln`.
   - Extract:
     - Neurorights envelope (RoH ceilings, safety floors).
     - Allowed operation types and parameters.
     - Tag and jurisdiction constraints relevant to Perplexity.

   - Treat these as hard feasibility constraints, not preferences.

3. **Build explicit intents**

   - For the requested change, define a structured list of “intents”:
     - Which SKO classes or BLE operations are affected.
     - What data or state is read or written.
     - Whether any neurorights‑sensitive fields are touched.

   - Intents must be non‑actuating:
     - Parsing, simulation, mock telemetry, decision logic, and policy evaluation are allowed.
     - Real discovery, connect, write, or firmware operations are forbidden.

4. **Call the BLE guard**

   - Use the non‑actuating BLE guard entrypoint (Rust crate or tool) defined in the catalog:
     - Provide mock link parameters and intents.
     - Request a decision vector (allow/deny, RoH, safety, justification).[file:19]

   - Do not circumvent the guard. If the guard rejects an intent, the agent must not propose code that performs it.

5. **Select and follow a playbook**

   - Query `repo_index.db` for playbooks relevant to BLE guard and Perplexity:
     - E.g. `ble-guard-simulate`, `perplexity-ble-guard-simulate`.
   - Choose the closest matching playbook and follow its steps:
     - File locations for new code.
     - Required tests and examples.
     - Required documentation updates.

   - If no perfect playbook exists, adapt the closest one and document any deviations in the PR description.

6. **Generate changes**

   - Only after the guard has approved the intents and a playbook is selected:
     - Propose Rust, ALN, or Markdown changes under the prescribed paths.
     - Ensure all new crates use `[lints] workspace = true`.
     - Add or update catalog entries via existing seeding tools, not by editing `repo_index.db` directly.[file:19]

7. **Self‑check governance**

   - Before finalizing:
     - Re‑run the BLE guard in simulation on the proposed intent set.
     - Ensure all RoH and neurorights ceilings remain satisfied.
     - Ensure catalog integrity scripts and lint inheritance checks would pass.

## 3. Response format for AI agents

The agent should respond in the following structure:

1. **Summary**

   - One paragraph describing what will be added or changed for Perplexity BLE guard.

2. **Catalog inspection**

   - List:
     - Relevant schemas in `repo_index.db`.
     - Relevant playbooks.
     - Target crates and files.

3. **Perplexity profile constraints**

   - Summarize:
     - RoH ceiling for Perplexity.
     - Safety strength minimum, knowledge thresholds, and any special tags.

4. **Intent plan**

   - A table with:
     - Intent ID.
     - Operation type (parse, simulate, mock_guard_call).
     - SKO / profile scope.
     - Expected guard decision.

5. **Guard simulation**

   - Describe:
     - Guard entrypoints used.
     - Mock link parameters.
     - Observed decisions (allow/deny, RoH, safety).

6. **Playbook alignment**

   - Name the playbook used.
   - Map each step in the playbook to specific edits or tests.

7. **Proposed code and docs**

   - List files to be created or modified:
     - Paths.
     - Short description.
     - Any changes to examples or telemetry.

8. **Governance checklist**

   - Confirm:
     - Lint inheritance (`[lints] workspace = true`) is satisfied.
     - Catalog seeding tools will be used for new assets.
     - Non‑actuating constraint is maintained.
     - BLE guard remains the single authority over allow/deny decisions.

This template ensures that AI agents operate inside the governed, non‑actuating BLE core and respect the Perplexity profile’s neurorights envelope.[file:19]

---

## 25. End‑to‑end integration test script (non‑actuating)

Create: `tests/integration_test.sh`

This script:

- Runs entirely indoors: no hardware, no BLE radios.
- Parses a Perplexity playbook.
- Constructs intents.
- Invokes the guard with mock link parameters.
- Checks that decisions match the neurorights envelope encoded in the Perplexity profile.[file:19]

```bash
#!/usr/bin/env bash
# tests/integration_test.sh
# MIT OR Apache-2.0
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE}")/.." && pwd)"
cd "${ROOT_DIR}"

echo "=== BLE non-actuating end-to-end integration test (Perplexity) ==="

# 1) Ensure required tools exist.
if ! command -v jq >/dev/null 2>&1; then
  echo "ERROR: jq is required for this test." >&2
  exit 1
fi

if [[ ! -x "tools/check_perplexity_profile.sh" ]]; then
  echo "ERROR: tools/check_perplexity_profile.sh not found or not executable." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: cargo is required to run guard simulation binaries." >&2
  exit 1
fi

PROFILE="config/perplexity-ble-guard-v1.profile.aln"
PLAYBOOK="docs/perplexity-ble-guard-playbook.yaml"
MOCK_TELEMETRY="examples/perplexity-ble-guard-telemetry-sample.json"

if [[ ! -f "${PROFILE}" ]]; then
  echo "ERROR: Perplexity profile not found at ${PROFILE}" >&2
  exit 1
fi

if [[ ! -f "${PLAYBOOK}" ]]; then
  echo "ERROR: Perplexity playbook not found at ${PLAYBOOK}" >&2
  exit 1
fi

if [[ ! -f "${MOCK_TELEMETRY}" ]]; then
  echo "ERROR: Mock telemetry not found at ${MOCK_TELEMETRY}" >&2
  exit 1
fi

# 2) Run profile checker as a precondition.
echo "-- Checking Perplexity profile..."
bash tools/check_perplexity_profile.sh

# 3) Parse playbook to construct intents (non-actuating).
#    Assume playbook contains a YAML list of steps with 'intent_id' and 'operation'.
echo "-- Parsing playbook for intents..."
INTENTS_JSON="$(python - <<'PY'
import sys, json, yaml

playbook_path = "docs/perplexity-ble-guard-playbook.yaml"
with open(playbook_path, "r", encoding="utf-8") as f:
    pb = yaml.safe_load(f)

steps = pb.get("steps", [])
intents = []
for step in steps:
    intent = {
        "intent_id": step.get("id"),
        "operation": step.get("operation"),
        "target": step.get("target"),
        "mode": step.get("mode", "simulate"),
    }
    intents.append(intent)

print(json.dumps({"intents": intents}))
PY
)"

echo "Constructed intents:"
echo "${INTENTS_JSON}" | jq '.'

# 4) Invoke non-actuating BLE guard with mock link parameters.
#    Assume a guard simulation binary exists:
#    cargo run -p ble-tools-guard --bin ble_guard_sim -- --profile ... --intents-json ... --mock-link ...
echo "-- Running BLE guard simulation..."
TMP_INTENTS="$(mktemp)"
printf '%s\n' "${INTENTS_JSON}" > "${TMP_INTENTS}"

cargo run -q -p ble-tools-guard --bin ble_guard_sim \
  -- \
  --profile "${PROFILE}" \
  --intents-json "${TMP_INTENTS}" \
  --mock-link '{
    "device_id": "mock-perplexity-device",
    "rssi": -50,
    "channel": 37,
    "mode": "simulation"
  }' \
  --output-json /tmp/ble_guard_perplexity_result.json

RESULT_JSON="/tmp/ble_guard_perplexity_result.json"

if [[ ! -f "${RESULT_JSON}" ]]; then
  echo "ERROR: Guard simulation did not produce result JSON." >&2
  exit 1
fi

echo "Guard decisions:"
cat "${RESULT_JSON}" | jq '.'

# 5) Assert decisions match neurorights envelope.
#    Expect: no decision breaches RoH ceiling or safety floor from profile.
echo "-- Asserting neurorights envelope compliance..."
ROH_MAX="$(grep -E 'roh_max' "${PROFILE}" | head -n1 | sed 's/[^0-9\.]//g' || echo "0.30")"
SAFETY_MIN="$(grep -E 'safety_min' "${PROFILE}" | head -n1 | sed 's/[^0-9\.]//g' || echo "0.75")"

BREACH_COUNT="$(jq \
  --argjson roh_max "${ROH_MAX:-0.30}" \
  --argjson safety_min "${SAFETY_MIN:-0.75}" '
  .decisions
  | map(select(.roh > $roh_max or .safety < $safety_min))
  | length
' "${RESULT_JSON}")"

if [[ "${BREACH_COUNT}" -ne 0 ]]; then
  echo "ERROR: ${BREACH_COUNT} guard decisions breach neurorights envelope (RoH/Safety)." >&2
  exit 1
fi

# 6) Ensure all operations are non-actuating.
#    We assert that all decisions have `"actuation": false`.
NON_ACTUATING_OK="$(jq '
  .decisions
  | all(.actuation == false)
' "${RESULT_JSON}")"

if [[ "${NON_ACTUATING_OK}" != "true" ]]; then
  echo "ERROR: Some decisions are marked as actuating; non-actuating invariant violated." >&2
  exit 1
fi

echo "End-to-end BLE non-actuating integration test: PASS"
```

This script follows your existing pattern: use profiles and catalog/playbook metadata to drive a fully simulated pipeline, ensuring neurorights and non‑actuating constraints hold end‑to‑end.[file:19]

---

## Embedded progress tracker (21–25)

- Implementable now:
  - Workspace AI‑safe Clippy profile in root `Cargo.toml` and `[lints] workspace = true` in each crate manifest, including `ble-tools-*`.[file:19]
  - `.github/workflows/verify-perplexity-assets.yml` to verify Perplexity assets in `repo_index.db` and run `check_perplexity_profile.sh`.[file:13][file:19]
  - `tools/test_catalog_integrity.sh` for orphan/missing path/type checks on `repo_index.db`.[file:13]
  - `docs/ai-agent-prompt-template.md` capturing the safe BLE‑Code contribution workflow for AI agents, with Perplexity‑specific constraints.[file:19]
  - `tests/integration_test.sh` to simulate a full non‑actuating BLE interaction through the guard with neurorights assertions.[file:19]

- Still missing / to be implemented:
  - The `ble_guard_sim` binary and Perplexity playbook/telemetry files referenced by the integration test, plus any catalog seeding scripts that register them in `repo_index.db`.[file:13][file:19]

- K/E/R / Eco‑wealth for this response:
  - K ≈ 0.95 (scripts and templates directly match your existing governance, catalog, and neurorights design).[file:12][file:13][file:19]
  - E ≈ 0.76 (all work is catalog queries, shell, and non‑actuating simulations; no heavy computation).
  - R ≈ 0.09 (the artifacts tighten governance and non‑actuating guarantees; they don’t add hardware paths).
  - EW ≈ 0.90 (stronger linting, catalog integrity, and guard‑driven workflows increase safe eco‑wealth by reducing mis‑wired behavior).
