#!/usr/bin/env bash
# tools/check_lint_inheritance.sh
# MIT OR Apache-2.0
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

FAIL=0
RESULTS=()

while IFS= read -r -d '' toml; do
  # Skip workspace root if desired; assume root Cargo.toml has [workspace].
  if grep -q '^\[workspace\]' "${toml}"; then
    continue
  fi

  CrateDir="$(dirname "${toml}")"
  CrateName="$(basename "${CrateDir}")"

  if grep -q '^\[lints\]' "${toml}"; then
    if grep -Eq '^\s*workspace\s*=\s*true' "${toml}"; then
      RESULTS+=("{\"crate\":\"${CrateName}\",\"path\":\"${toml}\",\"lint_inheritance\":true}")
    else
      echo "ERROR: crate '${CrateName}' (${toml}) defines [lints] but does not set workspace = true" >&2
      RESULTS+=("{\"crate\":\"${CrateName}\",\"path\":\"${toml}\",\"lint_inheritance\":false}")
      FAIL=1
    fi
  else
    echo "ERROR: crate '${CrateName}' (${toml}) is missing [lints] workspace = true" >&2
    RESULTS+=("{\"crate\":\"${CrateName}\",\"path\":\"${toml}\",\"lint_inheritance\":false}")
    FAIL=1
  fi
done < <(find . -name Cargo.toml -print0)

# Emit JSON report to stdout for the unified checker.
printf '{\n'
printf '  "version": 1,\n'
printf '  "tool": "check_lint_inheritance",\n'
printf '  "results": [\n'
len="${#RESULTS[@]}"
for i in "${!RESULTS[@]}"; do
  printf '    %s%s\n' "${RESULTS[$i]}" "$([[ "$i" -lt "$((len-1))" ]] && echo ',' || echo '')"
done
printf '  ]\n'
printf '}\n'

exit "${FAIL}"
