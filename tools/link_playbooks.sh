#!/bin/bash
# tools/link_playbooks.sh
# Extract playbook ↔ schema references into the database

set -euo pipefail

DB="repo_index.db"

for playbook in playbooks/*.aln; do
    [ -f "$playbook" ] || continue
    while IFS= read -r ref_line; do
        # Extract ref_type (profile_ref or bci_stream_profile_ref)
        ref_type=$(echo "$ref_line" | sed -n 's/.*\(profile_ref\|bci_stream_profile_ref\)\s*"\([^"]*\)".*/\1/p')
        # Extract schema path
        schema=$(echo "$ref_line" | sed -n 's/.*"\([^"]*\)".*/\1/p')
        if [ -n "$ref_type" ] && [ -n "$schema" ]; then
            sqlite3 "$DB" "INSERT OR IGNORE INTO playbook_schema_refs(playbook_path, ref_type, schema_path) VALUES('$playbook', '$ref_type', '$schema');"
        fi
    done < <(grep -E 'profile_ref|bci_stream_profile_ref' "$playbook" 2>/dev/null || true)
done

echo "Playbook-schema links extracted successfully."
