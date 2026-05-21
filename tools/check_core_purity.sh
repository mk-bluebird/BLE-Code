#!/bin/bash
# tools/check_core_purity.sh
# Ensure core crates do not contain platform-specific / non-pure dependencies.
#
# Current semantics:
#   - For each core crate, grep its Cargo.toml for disallowed dependencies:
#       btleplug, android, jni, tokio.
#   - Fail (non-zero exit) if any are present, otherwise print OK.
#
# Forward-compatible hook:
#   - Optionally call the Rust-based ble-tools-core-purity checker when present,
#     using a config file that encodes the same crate list and bad-dependency regex.

set -euo pipefail

# List of core crates (relative to crates/).
CORE_CRATES=("ble-model" "ble-governance" "ble-guard" "ble-env-ingest")

# Regex of disallowed dependencies in Cargo.toml for core crates.
BAD_DEP_REGEX='btleplug|android|jni|tokio'

EXIT=0

# 1. Current behavior: grep Cargo.toml files for forbidden dependencies.
for crate in "${CORE_CRATES[@]}"; do
    cargo_toml="crates/$crate/Cargo.toml"
    if [[ -f "$cargo_toml" ]]; then
        if grep -Eq "$BAD_DEP_REGEX" "$cargo_toml"; then
            echo "ERROR: core crate '$crate' contains a disallowed dependency (matched: $BAD_DEP_REGEX) in $cargo_toml"
            EXIT=1
        fi
    else
        echo "WARN: expected Cargo.toml not found for core crate '$crate' at $cargo_toml" >&2
    fi
done

# 2. Optional: call Rust-based core purity checker if available.
#    This is strictly additive and will NOT change behavior unless the binary exists.
#    You can remove this block once you fully migrate to the Rust tool.
if command -v cargo >/dev/null 2>&1; then
    if grep -q '\[package\]' "crates/ble-tools-core-purity/Cargo.toml" 2>/dev/null; then
        # Allow override of config path; default to tools/core_purity.toml
        : "${BLE_CORE_PURITY_CONFIG:=tools/core_purity.toml}"

        echo "INFO: Running ble-tools-core-purity (config: ${BLE_CORE_PURITY_CONFIG})..."
        if ! BLE_CORE_PURITY_CONFIG="${BLE_CORE_PURITY_CONFIG}" \
             cargo run -p ble-tools-core-purity --quiet; then
            echo "ERROR: ble-tools-core-purity reported violations."
            EXIT=1
        fi
    fi
fi

if [ "$EXIT" -eq 0 ]; then
    echo "OK: All core crates are pure (no disallowed platform dependencies)."
fi

exit "$EXIT"
