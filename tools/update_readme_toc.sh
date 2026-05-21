#!/usr/bin/env bash
# tools/update_readme_toc.sh
# MIT OR Apache-2.0
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

DB="${REPO_INDEX_DB:-${ROOT_DIR}/repo_index.db}"
README="${ROOT_DIR}/README.md"
TOC_START="<!-- CATALOG_TOC_START -->"
TOC_END="<!-- CATALOG_TOC_END -->"

if [[ ! -f "${DB}" ]]; then
  echo "ERROR: repo_index.db not found at ${DB}" >&2
  exit 1
fi

# Generate TOC from catalog.
TOC_CONTENT="$(sqlite3 -markdown "${DB}" '
.mode list
.headers off

-- Schemas
SELECT printf("### Schemas\n") UNION ALL
SELECT printf("- `%s` (%s) – `%s`", name, kind, path)
FROM schemas
ORDER BY name;

-- Separator
SELECT X""; 

-- Playbooks
SELECT printf("### Playbooks\n") UNION ALL
SELECT printf("- `%s` (%s) – `%s`", name, kind, path)
FROM playbooks
ORDER BY name;

-- Separator
SELECT X"";

-- Crates
SELECT printf("### Crates\n") UNION ALL
SELECT printf("- `%s` (%s) – `%s`", name, kind, path)
FROM crates
ORDER BY name;

-- Separator
SELECT X"";

-- Key documents (policies, design)
SELECT printf("### Key Documents\n") UNION ALL
SELECT printf("- `%s` (%s) – `%s`", name, category, path)
FROM documents
WHERE category IN (\"policy\",\"design\",\"spec\")
ORDER BY name;
' | sed '/^$/d')"

# Build new README.
if ! grep -q "${TOC_START}" "${README}"; then
  # Append if markers missing.
  {
    cat "${README}"
    echo
    echo "${TOC_START}"
    echo
    printf "%s\n" "${TOC_CONTENT}"
    echo
    echo "${TOC_END}"
  } >"${README}.tmp"
else
  # Replace between markers.
  awk -v start="${TOC_START}" -v end="${TOC_END}" -v toc="${TOC_CONTENT}" '
    BEGIN { in_toc = 0 }
    {
      if ($0 ~ start) {
        print $0
        print ""
        print toc
        print ""
        in_toc = 1
      } else if ($0 ~ end) {
        in_toc = 0
        print $0
      } else if (!in_toc) {
        print $0
      }
    }
  ' "${README}" >"${README}.tmp"
fi

mv "${README}.tmp" "${README}"
echo "README.md TOC updated from repo_index.db."
