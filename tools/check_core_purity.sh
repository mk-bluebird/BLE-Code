#!/bin/bash
# tools/check_core_purity.sh
# Ensure core crates do not contain platform-specific dependencies

set -euo pipefail

CORE_CRATES=("ble-model" "ble-governance" "ble-guard" "ble-env-ingest")
BAD_DEP_REGEX='btleplug|android|jni|tokio'
EXIT=0

for crate in "${CORE_CRATES[@]}"; do
    if grep -Eq "$BAD_DEP_REGEX" "crates/$crate/Cargo.toml" 2>/dev/null; then
        echo "ERROR: core crate '$crate' contains a disallowed dependency"
        EXIT=1
    fi
done

if [ $EXIT -eq 0 ]; then
    echo "OK: All core crates are pure (no platform dependencies)."
fi

exit $EXIT
