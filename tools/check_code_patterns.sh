#!/bin/bash
# tools/check_code_patterns.sh
# Check for forbidden code patterns in core crates

set -euo pipefail

EXIT=0

# Forbid unsafe blocks in core crates (but allow #[forbid(unsafe_code)] declarations)
# We look for 'unsafe {' or 'unsafe impl' or 'unsafe fn' patterns
if grep -rnE 'unsafe\s*\{|unsafe\s+impl|unsafe\s+fn' crates/ble-model crates/ble-governance crates/ble-guard 2>/dev/null; then
    echo "ERROR: 'unsafe' block/impl/fn detected in core crates"
    EXIT=1
fi

# Ban direct BLE actuation calls (example: Peripheral::connect)
if grep -rn '::connect()' crates/ble-adapter-btleplug crates/ble-android-ffi 2>/dev/null; then
    echo "WARNING: potential direct actuation call found (may be behind guard?)"
    # This is a soft check; we can make it fatal later.
fi

# Deny unwrap in core (optional but recommended)
if grep -rn '\.unwrap()' crates/ble-model crates/ble-governance crates/ble-guard 2>/dev/null; then
    echo "ERROR: .unwrap() detected in core crate – use proper error handling"
    EXIT=1
fi

if [ $EXIT -eq 0 ]; then
    echo "OK: No forbidden code patterns detected."
fi

exit $EXIT
