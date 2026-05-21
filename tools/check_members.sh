#!/bin/bash
# tools/check_members.sh
# Verify that every directory under crates/ with Cargo.toml is listed in root workspace members

set -euo pipefail

# Extract members from root Cargo.toml (lines inside members = [...])
# Members are stored as "crates/xxx" so we strip the "crates/" prefix for comparison
ROOT_MEMBERS=$(sed -n '/^members = \[/,/^\]/p' Cargo.toml | grep -E '^\s*"' | sed 's/.*"crates\/\(.*\)".*/\1/' || true)

# Find all crate directories with Cargo.toml (strip crates/ prefix)
FS_DIRS=$(find crates -maxdepth 2 -name Cargo.toml -exec dirname {} \; | sed 's|^crates/||' | sort)

MISMATCH=0

# Check members not on disk
for m in $ROOT_MEMBERS; do
    if [ ! -f "crates/$m/Cargo.toml" ]; then
        echo "ERROR: workspace member 'crates/$m' not found on disk"
        MISMATCH=1
    fi
done

# Check disk dirs not in members
for d in $FS_DIRS; do
    if ! echo "$ROOT_MEMBERS" | grep -qw "$d"; then
        echo "ERROR: directory 'crates/$d' exists but is not in workspace members"
        MISMATCH=1
    fi
done

if [ $MISMATCH -eq 0 ]; then
    echo "OK: All workspace members are consistent with disk."
fi

exit $MISMATCH
