#!/bin/bash
# tools/playbook_simulate.sh
# Simulate a playbook, enforce guard/actuation invariants, and print a static view.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLAYBOOK_PATH="${1:-${REPO_ROOT}/playbooks/example-playbook.yaml}"

if [[ -z "${PLAYBOOK_PATH}" ]]; then
  echo "Usage: $0 <playbook.yaml>" >&2
  exit 1
fi

if [[ ! -f "${PLAYBOOK_PATH}" ]]; then
  echo "ERROR: playbook not found at ${PLAYBOOK_PATH}" >&2
  exit 1
fi

if ! command -v yq >/dev/null 2>&1; then
  echo "ERROR: yq is required for playbook_simulate.sh" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "ERROR: jq is required for playbook_simulate.sh" >&2
  exit 1
fi

TMP_JSON="$(mktemp)"
trap 'rm -f "${TMP_JSON}"' EXIT

yq -o=json '.' "${PLAYBOOK_PATH}" > "${TMP_JSON}"

# 1. Check: every step with an intent has a guard containing require_decision "Allowed".

missing_guard_count=$(
  jq '
    .steps
    | map(
        select(has("intent"))
        | select(
            (has("guard") | not)
            or
            (
              .guard
              | tostring
              | contains("require_decision \"Allowed\"") | not
            )
          )
      )
    | length
  ' "${TMP_JSON}"
)

if [[ "${missing_guard_count}" -ne 0 ]]; then
  echo "ERROR: ${missing_guard_count} step(s) with intent are missing a guard with require_decision \"Allowed\"." >&2
  exit 1
fi

# 2. Check: any step with actuation_hint references a prior guarded step.

violations=$(
  jq -r '
    .steps
    | to_entries
    | (reduce .[] as $s (
        {seen_guarded_ids: [], violations: []};
        . as $acc
        | if ($s.value.guard and ($s.value.guard | tostring | contains("require_decision \"Allowed\"")))
          then
            ($acc.seen_guarded_ids + [($s.value.id // ($s.key|tostring))]) as $ids
            | {seen_guarded_ids: $ids, violations: $acc.violations}
          elif ($s.value.actuation_hint)
          then
            ($s.value.actuation_hint.target_id // $s.value.target_id // null) as $target
            | if ($target != null and (($target|tostring) as $tid
                  | ($acc.seen_guarded_ids | index($tid)) != null))
              then $acc
              else
                {
                  seen_guarded_ids: $acc.seen_guarded_ids,
                  violations: ($acc.violations + [($s.value.id // ($s.key|tostring))])
                }
              end
          else
            $acc
          end
      )
    )
    | .violations[]
  ' "${TMP_JSON}"
)

if [[ -n "${violations}" ]]; then
  echo "ERROR: actuation_hint steps without prior guarded reference:" >&2
  echo "${violations}" | while read -r sid; do
    echo "  - step id: ${sid}" >&2
  done
  exit 1
fi

echo "OK: playbook guard invariants satisfied."
echo
echo "=== Static playbook simulation: ${PLAYBOOK_PATH} ==="
echo

grep -E 'step|intent|guard|require_decision|actuation_hint' "${PLAYBOOK_PATH}" || true

echo
echo "=== End of simulation ==="
