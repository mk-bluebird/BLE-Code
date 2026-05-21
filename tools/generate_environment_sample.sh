#!/bin/bash
# tools/generate_environment_sample.sh
# Generate a .ble-environment.aln entry from a mock scan JSON,
# append to the environment log, and validate against ble-session.v1.schema.json.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

MOCK_SCAN_JSON="${1:-${REPO_ROOT}/tools/mock-environment-scan.json}"
ENV_LOG="${2:-${REPO_ROOT}/logs/ble-environment.log.aln}"
SCHEMA_JSON="${3:-${REPO_ROOT}/schemas/ble-session.v1.schema.json}"

BLE_ENV_INGEST_BIN="${BLE_ENV_INGEST_BIN:-ble-env-ingest}"
JSON_VALIDATOR_BIN="${JSON_VALIDATOR_BIN:-validate-json-schema}"

if [[ ! -f "${MOCK_SCAN_JSON}" ]]; then
  echo "ERROR: mock scan JSON not found at ${MOCK_SCAN_JSON}" >&2
  exit 1
fi

if [[ ! -f "${SCHEMA_JSON}" ]]; then
  echo "ERROR: schema JSON not found at ${SCHEMA_JSON}" >&2
  exit 1
fi

mkdir -p "$(dirname "${ENV_LOG}")"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

ALN_OUT="${TMP_DIR}/environment.ble-environment.aln"
JSON_OUT="${TMP_DIR}/environment.ble-session.json"

echo "INFO: Running ble-env-ingest..."
# Assumed CLI: ble-env-ingest --input scan.json --aln-out x --json-out y
"${BLE_ENV_INGEST_BIN}" \
  --input "${MOCK_SCAN_JSON}" \
  --aln-out "${ALN_OUT}" \
  --json-out "${JSON_OUT}"

if [[ ! -f "${ALN_OUT}" || ! -f "${JSON_OUT}" ]]; then
  echo "ERROR: ble-env-ingest did not produce expected outputs." >&2
  exit 1
fi

echo "INFO: Validating JSON output against schema..."
"${JSON_VALIDATOR_BIN}" "${SCHEMA_JSON}" "${JSON_OUT}"

echo "INFO: Appending ALN record to ${ENV_LOG}..."
{
  echo "# --- environment sample $(date -u +%Y-%m-%dT%H:%M:%SZ) ---"
  cat "${ALN_OUT}"
  echo
} >> "${ENV_LOG}"

echo "OK: environment sample generated and logged."
