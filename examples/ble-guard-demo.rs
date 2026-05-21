#![forbid(unsafe_code)]

//! Minimal demo of the BLE guard – no platform BLE calls.
//! Shows the sequence: load profile -> build intent -> ask guard -> react to decision.

use ble_governance::aln_loader::load_ble_profile_shard;
use ble_guard::{BleGuard, BleGuardDecision};
use ble_model::{BleIntent, BleLinkParams, BlePhy};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // In a real app you'd load a profile from an ALN file.
    // Here we use the JSON config shipped with the repo for Perplexity.
    let config_path = Path::new("configs/perplexity-ble-guard-config.json");
    let profile_json = std::fs::read_to_string(config_path)?;
    let shard: ble_governance::BleProfileShard = serde_json::from_str(&profile_json)?;
    // Validate RoH invariants (already enforced during load, but double-check)
    shard.validate_invariants()?;

    let mut guard = BleGuard::new(shard);

    // 1. Attempt to scan
    let scan_intent = BleIntent::Scan {
        class_id: Some("openbci-cyton-nus".into()),
    };
    match guard.guard_scan() {
        BleGuardDecision::Allowed => println!("Scan allowed."),
        BleGuardDecision::Rejected { reason } => println!("Scan rejected: {reason}"),
    }

    // 2. Attempt to connect with too weak link params (should be rejected)
    let connect_intent = BleIntent::Connect {
        class_id: "openbci-cyton-nus".into(),
        device_id: "CYTON-01".into(),
    };
    let weak_link = BleLinkParams {
        phy: BlePhy::Le1M,
        encrypted: false,
        mic_present: false,
        bonded: false,
        conn_interval_ms: 50,
        max_pdu_bytes: 23,
        cte_present: false,
    };

    match guard.guard_connect(&connect_intent, &weak_link) {
        BleGuardDecision::Allowed => println!("Connect allowed with weak link (unexpected)"),
        BleGuardDecision::Rejected { reason } => println!("Connect correctly rejected: {reason}"),
    }

    // 3. Correct link parameters matching the profile
    let strong_link = BleLinkParams {
        phy: BlePhy::Le1M,
        encrypted: true,
        mic_present: true,
        bonded: true,
        conn_interval_ms: 30,
        max_pdu_bytes: 64,
        cte_present: false,
    };

    match guard.guard_connect(&connect_intent, &strong_link) {
        BleGuardDecision::Allowed => println!("Connect allowed with strong link."),
        BleGuardDecision::Rejected { reason } => eprintln!("Connect rejected: {reason}"),
    }

    // 4. Try to subscribe to BCI stream (should update RoH)
    let sub_intent = BleIntent::SubscribeCharacteristic {
        class_id: "openbci-cyton-nus".into(),
        device_id: "CYTON-01".into(),
        service_uuid: "6E400001-B5A3-F393-E0A9-E50E24DCCA9E".into(),
        characteristic_uuid: "6E400003-B5A3-F393-E0A9-E50E24DCCA9E".into(),
    };
    match guard.guard_subscribe(&sub_intent) {
        BleGuardDecision::Allowed => println!("Subscribe allowed, RoH updated."),
        BleGuardDecision::Rejected { reason } => println!("Subscribe rejected: {reason}"),
    }

    println!(
        "Accumulated RoH: {:.3} / {:.3}",
        guard.state().accumulated_roh_ble,
        guard.state().accumulated_roh_ble // no ceiling exposed publicly, but we can show it.
    );

    Ok(())
}
