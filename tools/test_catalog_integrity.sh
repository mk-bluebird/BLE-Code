#!/usr/bin/env bash
# tools/test_catalog_integrity.sh
# MIT OR Apache-2.0
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

DB="${REPO_INDEX_DB:-${ROOT_DIR}/repo_index.db}"

if [[ ! -f "${DB}" ]]; then
  echo "ERROR: repo_index.db not found at ${DB}" >&2
  exit 1
fi

fail=0

echo "== Catalog integrity checks on ${DB} =="

# 1) Orphaned schema_references.
echo "-- Checking orphaned schema_references..."
orph_schemas=$(
  sqlite3 "${DB}" <<'SQL'
  SELECT COUNT(*)
  FROM schema_references r
  LEFT JOIN schemas s_from ON s_from.id = r.from_schema_id
  LEFT JOIN schemas s_to   ON s_to.id   = r.to_schema_id
  WHERE s_from.id IS NULL OR s_to.id IS NULL;
SQL
)
if [[ "${orph_schemas}" -ne 0 ]]; then
  echo "ERROR: ${orph_schemas} orphaned schema_references rows detected."
  fail=1
fi

# 2) Orphaned playbook_dependencies.
echo "-- Checking orphaned playbook_dependencies..."
orph_playbooks=$(
  sqlite3 "${DB}" <<'SQL'
  SELECT COUNT(*)
  FROM playbook_dependencies d
  LEFT JOIN playbooks p ON p.id = d.playbook_id
  WHERE p.id IS NULL;
SQL
)
if [[ "${orph_playbooks}" -ne 0 ]]; then
  echo "ERROR: ${orph_playbooks} orphaned playbook_dependencies rows detected."
  fail=1
fi

# 3) Orphaned crate_artifacts.
echo "-- Checking orphaned crate_artifacts..."
orph_crates=$(
  sqlite3 "${DB}" <<'SQL'
  SELECT COUNT(*)
  FROM crate_artifacts a
  LEFT JOIN crates c ON c.id = a.crate_id
  WHERE c.id IS NULL;
SQL
)
if [[ "${orph_crates}" -ne 0 ]]; then
  echo "ERROR: ${orph_crates} orphaned crate_artifacts rows detected."
  fail=1
fi

# 4) Missing files for catalog entries (best-effort).
#    This assumes paths are relative to repo root.
echo "-- Checking that catalog file paths exist on disk..."
missing_files=0

check_paths() {
  local sql="$1"
  while IFS= read -r rel; do
    [[ -z "${rel}" ]] && continue
    if [[ ! -e "${ROOT_DIR}/${rel}" ]]; then
      echo "ERROR: Catalog path not found on disk: ${rel}"
      missing_files=$((missing_files + 1))
    fi
  done < <(sqlite3 "${DB}" "${sql}")
}

# Schemas, playbooks, crates.manifest_path, documents.
check_paths "SELECT path FROM schemas;"
check_paths "SELECT path FROM playbooks;"
check_paths "SELECT manifest_path FROM crates;"
check_paths "SELECT path FROM documents;"

if [[ "${missing_files}" -ne 0 ]]; then
  echo "ERROR: ${missing_files} catalog paths are missing on disk."
  fail=1
fi

# 5) Basic type consistency checks (non-empty names and paths).
echo "-- Checking non-empty names and paths..."
empty_fields=$(
  sqlite3 "${DB}" <<'SQL'
  SELECT
    (SELECT COUNT(*) FROM schemas   WHERE name = '' OR path = '') +
    (SELECT COUNT(*) FROM playbooks WHERE name = '' OR path = '') +
    (SELECT COUNT(*) FROM crates    WHERE name = '' OR path = '') +
    (SELECT COUNT(*) FROM documents WHERE name = '' OR path = '');
SQL
)
if [[ "${empty_fields}" -ne 0 ]]; then
  echo "ERROR: ${empty_fields} catalog rows have empty name or path."
  fail=1
fi

if [[ "${fail}" -ne 0 ]]; then
  echo "Catalog integrity: FAIL"
  exit 1
fi

echo "Catalog integrity: CLEAN"
