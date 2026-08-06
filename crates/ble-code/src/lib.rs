// crates/ble-code/src/lib.rs
#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// BLE device metadata as seen by scanners.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BleDeviceInfo {
    pub id: Uuid,
    pub address: String,
    pub name: Option<String>,
    pub rssi: i32,
    pub last_seen: DateTime<Utc>,
}

/// BLE scanner abstraction.
/// In production, this would wrap platform-specific BLE APIs.
/// Here, we provide a minimal interface and a dummy implementation.
#[derive(Clone, Debug)]
pub struct BleScanner;

impl BleScanner {
    /// Construct a new BLE scanner.
    pub fn new() -> Self {
        BleScanner
    }

    /// Perform a BLE scan and return discovered devices.
    /// For now, this returns an empty list; real implementations should
    /// integrate with OS/hardware BLE stacks.
    pub fn scan(&self) -> anyhow::Result<Vec<BleDeviceInfo>> {
        Ok(Vec::new())
    }
}
