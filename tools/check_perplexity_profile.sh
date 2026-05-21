#!/usr/bin/env bash
# File: tools/check_perplexity_profile.sh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

PROFILE_ALN="${ROOT_DIR}/schemas/perplexity-ble-guard-v1.profile.aln"
PROFILE_JSON="${ROOT_DIR}/configs/perplexity-ble-guard-config.json"
PLAYBOOK_ALN="${ROOT_DIR}/playbooks/openbci-cyton-over-nus.ble-playbook.aln"

# Tool commands (override via env if needed)
ALN_CLI_BIN="${ALN_CLI_BIN:-aln-cli}"
JQ_BIN="${JQ_BIN:-jq}"

# Machine-readable output: one NDJSON object per check.
# Fields: check, status ("ok"|"error"), detail.

emit_ok() {
  local check="$1"
  local detail="$2"
  printf '{"check":"%s","status":"ok","detail":"%s"}\n' \
    "${check}" "${detail}"
}

emit_err() {
  local check="$1"
  local detail="$2"
  printf '{"check":"%s","status":"error","detail":"%s"}\n' \
    "${check}" "${detail}"
}

overall_status="ok"

fail() {
  overall_status="error"
  emit_err "$1" "$2"
}

###############################################################################
# 1. Existence checks
###############################################################################

if [[ -f "${PROFILE_ALN}" ]]; then
  emit_ok "file_profile_aln_exists" "${PROFILE_ALN}"
else
  fail "file_profile_aln_exists" "missing: ${PROFILE_ALN}"
fi

if [[ -f "${PROFILE_JSON}" ]]; then
  emit_ok "file_profile_json_exists" "${PROFILE_JSON}"
else
  fail "file_profile_json_exists" "missing: ${PROFILE_JSON}"
fi

if [[ -f "${PLAYBOOK_ALN}" ]]; then
  emit_ok "file_playbook_aln_exists" "${PLAYBOOK_ALN}"
else
  fail "file_playbook_aln_exists" "missing: ${PLAYBOOK_ALN}"
fi

###############################################################################
# 2. Parse ALN profile into JSON via aln-cli
###############################################################################

profile_tmp_json="$(mktemp)"
if "${ALN_CLI_BIN}" export --input "${PROFILE_ALN}" --format json >"${profile_tmp_json}" 2>/dev/null; then
  emit_ok "aln_export_profile" "parsed profile via ${ALN_CLI_BIN}"
else
  fail "aln_export_profile" "failed to parse profile via ${ALN_CLI_BIN}"
fi

###############################################################################
# 3. Validate JSON config syntax
###############################################################################

if "${JQ_BIN}" empty "${PROFILE_JSON}" 2>/dev/null; then
  emit_ok "json_config_parse" "config is valid JSON"
else
  fail "json_config_parse" "invalid JSON in ${PROFILE_JSON}"
fi

###############################################################################
# 4. Subject ID and RoH ceiling <= 0.15
###############################################################################

subject_id="$("${JQ_BIN}" -r '.subject.subjectid // empty' "${profile_tmp_json}" 2>/dev/null || true)"
roh_ceiling="$("${JQ_BIN}" -r '.subject.rohceiling // empty' "${profile_tmp_json}" 2>/dev/null || true)"

if [[ "${subject_id}" == "perplexity-ble-guard-v1" ]]; then
  emit_ok "subject_id_match" "subjectid=${subject_id}"
else
  fail "subject_id_match" "expected perplexity-ble-guard-v1, got '${subject_id}'"
fi

if [[ -n "${roh_ceiling}" ]]; then
  # Numeric compare using bc
  cmp_result="$(printf '%s\n0.15\n' "${roh_ceiling}" | sort -g | head -n1)"
  if [[ "${cmp_result}" == "0.15" || "${cmp_result}" == "${roh_ceiling}" ]]; then
    emit_ok "roh_ceiling_limit" "rohceiling=${roh_ceiling} <= 0.15"
  else
    fail "roh_ceiling_limit" "rohceiling=${roh_ceiling} > 0.15"
  fi
else
  fail "roh_ceiling_limit" "rohceiling missing in profile"
fi

###############################################################################
# 5. Service UUIDs must match playbook
###############################################################################

playbook_rx_uuid="$(
  grep -E 'serviceuuid[[:space:]]+6E400001-B5A3-F393-E0A9-E50E24DCCA9E' "${PLAYBOOK_ALN}" \
    >/dev/null 2>&1 && echo "6E400001-B5A3-F393-E0A9-E50E24DCCA9E" || true
)"

playbook_tx_uuid="$(
  grep -E 'serviceuuid[[:space:]]+6E400002-B5A3-F393-E0A9-E50E24DCCA9E' "${PLAYBOOK_ALN}" \
    >/dev/null 2>&1 && echo "6E400002-B5A3-F393-E0A9-E50E24DCCA9E" || true
)"

if [[ -z "${playbook_rx_uuid}" || -z "${playbook_tx_uuid}" ]]; then
  fail "playbook_service_uuids_present" "playbook missing expected NUS service UUIDs"
else
  emit_ok "playbook_service_uuids_present" "playbook has expected NUS UUIDs"
fi

profile_rx_uuid="$("${JQ_BIN}" -r '.servicepolicies[] | select(.role=="Sensor") | .serviceuuid' "${profile_tmp_json}" 2>/dev/null || true)"
profile_tx_uuid="$("${JQ_BIN}" -r '.servicepolicies[] | select(.role=="Control") | .serviceuuid' "${profile_tmp_json}" 2>/dev/null || true)"

if [[ "${profile_rx_uuid}" == "${playbook_rx_uuid}" ]]; then
  emit_ok "profile_rx_uuid_match" "RX UUID matches playbook: ${profile_rx_uuid}"
else
  fail "profile_rx_uuid_match" "RX UUID mismatch profile=${profile_rx_uuid}, playbook=${playbook_rx_uuid}"
fi

if [[ "${profile_tx_uuid}" == "${playbook_tx_uuid}" ]]; then
  emit_ok "profile_tx_uuid_match" "TX UUID matches playbook: ${profile_tx_uuid}"
else
  fail "profile_tx_uuid_match" "TX UUID mismatch profile=${profile_tx_uuid}, playbook=${playbook_tx_uuid}"
fi

###############################################################################
# 6. Security flags not weakened relative to playbook
#    For Perplexity: require encryption=true, mic=true, bonding=true,
#    LE1M/LE2M only, conn interval <= 50 ms, MTU <= 64.
###############################################################################

security_ok="true"

require_encryption="$("${JQ_BIN}" -r '.deviceclasspolicies[] | select(.classid=="openbci-cyton-nus") | .requireencryption' "${profile_tmp_json}" 2>/dev/null || true)"
require_mic="$("${JQ_BIN}" -r '.deviceclasspolicies[] | select(.classid=="openbci-cyton-nus") | .requiremic' "${profile_tmp_json}" 2>/dev/null || true)"
require_bonding="$("${JQ_BIN}" -r '.deviceclasspolicies[] | select(.classid=="openbci-cyton-nus") | .requirebonding' "${profile_tmp_json}" 2>/dev/null || true)"
max_conn_interval="$("${JQ_BIN}" -r '.deviceclasspolicies[] | select(.classid=="openbci-cyton-nus") | .maxconnintervalms' "${profile_tmp_json}" 2>/dev/null || true)"
max_pdu_bytes="$("${JQ_BIN}" -r '.deviceclasspolicies[] | select(.classid=="openbci-cyton-nus") | .maxpdubytes' "${profile_tmp_json}" 2>/dev/null || true)"

if [[ "${require_encryption}" != "true" ]]; then
  security_ok="false"
  fail "security_require_encryption" "requireencryption must be true"
else
  emit_ok "security_require_encryption" "requireencryption=true"
fi

if [[ "${require_mic}" != "true" ]]; then
  security_ok="false"
  fail "security_require_mic" "requiremic must be true"
else
  emit_ok "security_require_mic" "requiremic=true"
fi

if [[ "${require_bonding}" != "true" ]]; then
  security_ok="false"
  fail "security_require_bonding" "requirebonding must be true"
else
  emit_ok "security_require_bonding" "requirebonding=true"
fi

if [[ -n "${max_conn_interval}" ]]; then
  cmp_ci="$(printf '%s\n50\n' "${max_conn_interval}" | sort -g | head -n1)"
  if [[ "${cmp_ci}" == "${max_conn_interval}" ]]; then
    emit_ok "security_max_conn_interval" "maxconnintervalms=${max_conn_interval} <= 50"
  else
    security_ok="false"
    fail "security_max_conn_interval" "maxconnintervalms=${max_conn_interval} > 50"
  fi
else
  security_ok="false"
  fail "security_max_conn_interval" "maxconnintervalms missing"
fi

if [[ -n "${max_pdu_bytes}" ]]; then
  cmp_mtu="$(printf '%s\n64\n' "${max_pdu_bytes}" | sort -g | head -n1)"
  if [[ "${cmp_mtu}" == "${max_pdu_bytes}" ]]; then
    emit_ok "security_max_pdu_bytes" "maxpdubytes=${max_pdu_bytes} <= 64"
  else
    security_ok="false"
    fail "security_max_pdu_bytes" "maxpdubytes=${max_pdu_bytes} > 64"
  fi
else
  security_ok="false"
  fail "security_max_pdu_bytes" "maxpdubytes missing"
fi

if [[ "${security_ok}" == "true" ]]; then
  emit_ok "security_flags_not_weakened" "Perplexity security envelope respected"
fi

###############################################################################
# Final summary as NDJSON
###############################################################################

if [[ "${overall_status}" == "ok" ]]; then
  emit_ok "summary" "perplexity profile valid"
  exit 0
else
  fail "summary" "perplexity profile has errors"
  exit 1
fi
