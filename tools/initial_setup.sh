#!/usr/bin/env bash
# tools/initial_setup.sh
# MIT OR Apache-2.0
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

DB="${REPO_INDEX_DB:-${ROOT_DIR}/repo_index.db}"

echo "=== Initial repository setup ==="

# 1) Check sqlite3.
if ! command -v sqlite3 >/dev/null 2>&1; then
  echo "ERROR: sqlite3 not found on PATH." >&2
  echo "Please install sqlite3 via your package manager and re-run:" >&2
  echo "  - Ubuntu: sudo apt-get install sqlite3" >&2
  echo "  - macOS:  brew install sqlite" >&2
  exit 1
fi

# 2) Create or reset repo_index.db schema.
if [[ -f "${DB}" ]]; then
  echo "Existing repo_index.db found at ${DB} – leaving in place."
else
  echo "Creating new repo_index.db at ${DB}..."
  sqlite3 "${DB}" <<'SQL'
PRAGMA foreign_keys = ON;

CREATE TABLE schemas (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  path TEXT NOT NULL,
  version TEXT NOT NULL,
  tags TEXT
);

CREATE TABLE playbooks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  kind TEXT NOT NULL,
  description TEXT
);

CREATE TABLE crates (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  kind TEXT NOT NULL,
  manifest_path TEXT NOT NULL
);

CREATE TABLE documents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  category TEXT NOT NULL
);

CREATE TABLE schema_references (
  from_schema_id INTEGER NOT NULL,
  to_schema_id INTEGER NOT NULL,
  relation TEXT NOT NULL,
  FOREIGN KEY(from_schema_id) REFERENCES schemas(id),
  FOREIGN KEY(to_schema_id) REFERENCES schemas(id)
);

CREATE TABLE playbook_dependencies (
  playbook_id INTEGER NOT NULL,
  target_type TEXT NOT NULL,
  target_id INTEGER NOT NULL,
  relation TEXT NOT NULL,
  FOREIGN KEY(playbook_id) REFERENCES playbooks(id)
);

CREATE TABLE crate_artifacts (
  crate_id INTEGER NOT NULL,
  artifact_type TEXT NOT NULL,
  artifact_path TEXT NOT NULL,
  FOREIGN KEY(crate_id) REFERENCES crates(id)
);
SQL
fi

# 3) Run seeding and linking tools, if present.
if [[ -x "tools/seed_catalog_from_fs.sh" ]]; then
  echo "Seeding catalog from filesystem..."
  bash tools/seed_catalog_from_fs.sh
else
  echo "WARNING: tools/seed_catalog_from_fs.sh not found; catalog may be empty."
fi

if [[ -x "tools/link_catalog_relations.sh" ]]; then
  echo "Linking catalog relations..."
  bash tools/link_catalog_relations.sh
else
  echo "WARNING: tools/link_catalog_relations.sh not found; relationships may be incomplete."
fi

# 4) Update README TOC from catalog.
if [[ -x "tools/update_readme_toc.sh" ]]; then
  echo "Updating README.md TOC..."
  bash tools/update_readme_toc.sh
else
  echo "WARNING: tools/update_readme_toc.sh not found; README TOC not updated."
fi

# 5) Run unified governance checks.
if [[ -x "tools/run_all_checks.sh" ]]; then
  echo "Running unified governance checks..."
  bash tools/run_all_checks.sh
else
  echo "WARNING: tools/run_all_checks.sh not found; governance checks not run."
fi

echo "Initial setup complete. Repository is ready for development."
