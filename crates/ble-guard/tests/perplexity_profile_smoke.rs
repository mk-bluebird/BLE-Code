// File: crates/ble-guard/tests/perplexity_profile_smoke.rs
#![forbid(unsafe_code)]

use ble_governance::BleProfileShard;
use ble_guard::BleGuard;
use std::fs;

#[test]
fn perplexity_profile_loads_into_guard() {
    let path = "schemas/perplexity-ble-guard-v1.profile.json";
    // Either maintain a JSON-exported copy in the repo or generate it in CI.
    let raw =
        fs::read_to_string(path).expect("perplexity profile JSON should exist for smoke test");
    let shard: BleProfileShard =
        serde_json::from_str(&raw).expect("perplexity profile JSON must parse");
    shard
        .validate_invariants()
        .expect("perplexity profile invariants must hold");
    let _guard = BleGuard::new(shard).expect("BleGuard should construct from Perplexity profile");
}
