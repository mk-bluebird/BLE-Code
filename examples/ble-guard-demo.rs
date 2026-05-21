#![forbid(unsafe_code)]

// Minimal demo of the BLE guard – no platform BLE calls.
// Shows the sequence: load profile -> build intent -> ask guard -> react to decision.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use ble_governance::aln_loader::load_ble_profile_shard;
use ble_guard::{BleGuard, BleGuardDecision, GuardEngine};
use ble_model::{BleIntent, BleLinkParams, BlePhy};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PerplexityConfig {
    api_endpoint: String,
    api_key: String,
    guard_profile: String,
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut dry_run = false;

    for arg in &args[1..] {
        if arg == "--dry-run" {
            dry_run = true;
        }
    }

    // Load Perplexity guard config used by the GuardEngine demo.
    let perplexity_config = load_perplexity_config()?;
    println!(
        "Loaded Perplexity config for profile '{}'",
        perplexity_config.guard_profile
    );

    let engine = GuardEngine::from_profile(&perplexity_config.guard_profile)?;
    let engine_input = "Example request: read-only access to BLE session logs.";
    let engine_decision = engine.evaluate(engine_input)?;
    println!("GuardEngine decision for input: {engine_decision}");

    if dry_run {
        println!("Dry-run mode: no BLE actuation performed.");
        return Ok(());
    }

    // Load BLE profile shard for BLE guard demo.
    // Prefer ALN if present; fall back to JSON config.
    let aln_path = Path::new("configs/perplexity-ble-profile.aln");
    let json_path = Path::new("configs/perplexity-ble-guard-config.json");

    let shard = if aln_path.exists() {
        let aln_text = fs::read_to_string(aln_path)?;
        load_ble_profile_shard(&aln_text)?
    } else {
        let profile_json = fs::read_to_string(json_path)?;
        let shard_json: ble_governance::BleProfileShard = serde_json::from_str(&profile_json)?;
        shard_json
    };

    shard.validate_invariants()?;

    let mut guard = BleGuard::new(shard);

    // 1. Attempt to scan.
    let scan_intent = BleIntent::Scan {
        class_id: Some("openbci-cyton-nus".into()),
    };
    match guard.guard_scan(&scan_intent) {
        BleGuardDecision::Allowed => println!("Scan allowed."),
        BleGuardDecision::Rejected { reason } => println!("Scan rejected: {reason}"),
    }

    // 2. Attempt to connect with too weak link params (should be rejected).
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
        BleGuardDecision::Allowed => {
            println!("Connect allowed with weak link (unexpected).");
        }
        BleGuardDecision::Rejected { reason } => {
            println!("Connect correctly rejected: {reason}");
        }
    }

    // 3. Correct link parameters matching the profile.
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

    // 4. Try to subscribe to BCI stream (should update RoH).
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

    let state = guard.state();
    println!(
        "Accumulated RoH (BLE): {:.3}",
        state.accumulated_roh_ble
    );

    Ok(())
}

fn load_perplexity_config() -> anyhow::Result<PerplexityConfig> {
    let mut path = PathBuf::from("configs");
    path.push("perplexity-ble-guard-config.json");

    let data = fs::read_to_string(&path)?;
    let cfg: PerplexityConfig = serde_json::from_str(&data)?;
    Ok(cfg)
}
