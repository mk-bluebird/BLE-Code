#!/usr/bin/env bash
# tools/run_all_checks.sh
# MIT OR Apache-2.0
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${OUT_DIR:-${ROOT_DIR}/output/checks}"
mkdir -p "${OUT_DIR}"

REPORT="${OUT_DIR}/governance-report.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

# Helper: run a command, capture JSON, and status.
run_check() {
  local name="$1"; shift
  local outfile="${TMP_DIR}/${name}.json"
  local status=0
  if "$@" >"${outfile}" 2>"${TMP_DIR}/${name}.stderr"; then
    status=0
  else
    status=$?
  fi

  # If tool did not emit JSON, wrap stderr.
  if ! jq -e . "${outfile}" >/dev/null 2>&1; then
    local err_msg
    err_msg="$(sed 's/"/\\"/g' "${TMP_DIR}/${name}.stderr" || true)"
    cat >"${outfile}" <<EOF
{"tool":"${name}","status":"error","exit_code":${status},"output":null,"error_message":"${err_msg}"}
EOF
  else
    # Ensure tool, status, exit_code fields exist.
    local json
    json="$(jq --arg tool "${name}" --arg status_str "$( [ ${status} -eq 0 ] && echo ok || echo error )" \
      --argjson exit_code "${status}" '
      . + {
        tool: $tool,
        status: $status_str,
        exit_code: $exit_code
      }' "${outfile}")"
    printf '%s\n' "${json}" >"${outfile}"
  fi
}

echo "Running unified governance checks..."

# 1) Rust code pattern checker (item 11).
run_check "code_patterns" \
  cargo run -p ble-tools-code-patterns --quiet --manifest-path "${ROOT_DIR}/Cargo.toml"

# 2) Lint inheritance (item 17) – script below.
run_check "lint_inheritance" \
  bash "${ROOT_DIR}/tools/check_lint_inheritance.sh"

# 3) Playbook simulate checks (item 15).
if [[ -x "${ROOT_DIR}/tools/playbook_simulate.sh" ]]; then
  run_check "playbook_simulate" \
    bash "${ROOT_DIR}/tools/playbook_simulate.sh"
fi

# 4) Environment ingestion sample (item 14), if present.
if [[ -x "${ROOT_DIR}/tools/generate_environment_sample.sh" ]]; then
  run_check "env_ingest" \
    bash "${ROOT_DIR}/tools/generate_environment_sample.sh" "--check-only"
fi

# 5) Guard demo dry-run (item 13).
run_check "ble_guard_demo" \
  cargo run --example ble-guard-demo -- --dry-run --no-color

# Aggregate results.
ALL_JSON="$(jq -s '.' "${TMP_DIR}"/*.json)"

# Compute overall status.
OVERALL_OK="$(printf '%s\n' "${ALL_JSON}" | jq 'all(.[]; .exit_code == 0)' || echo false)"

cat >"${REPORT}" <<EOF
{
  "version": 1,
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "root": "${ROOT_DIR}",
  "results": ${ALL_JSON},
  "overall_ok": ${OVERALL_OK}
}
EOF

echo "Unified governance report written to: ${REPORT}"

if [[ "${OVERALL_OK}" != "true" ]]; then
  exit 1
fi
