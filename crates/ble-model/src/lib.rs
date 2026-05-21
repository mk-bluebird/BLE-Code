// File: crates/ble-model/src/lib.rs

#![forbid(unsafe_code)]
// Temporary allows until documentation is added
#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::struct_excessive_bools)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── BLE Intent ────────────────────────────────────────────────────────────────

/// High-level BLE action an AI or host wants to perform.
/// Purely descriptive, non-actuating.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BleIntent {
    Scan {
        class_id: Option<String>,
    },
    Connect {
        class_id: String,
        device_id: String,
    },
    SubscribeCharacteristic {
        class_id: String,
        device_id: String,
        service_uuid: String,
        characteristic_uuid: String,
    },
    WriteCharacteristic {
        class_id: String,
        device_id: String,
        service_uuid: String,
        characteristic_uuid: String,
        payload_len: usize,
    },
}

// ── PHY & Link Parameters ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlePhy {
    Le1M,
    Le2M,
    LeCodedS2,
    LeCodedS8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BleLinkParams {
    pub phy: BlePhy,
    pub encrypted: bool,
    pub mic_present: bool,
    pub bonded: bool,
    pub conn_interval_ms: u32,
    pub max_pdu_bytes: u16,
    pub cte_present: bool,
}

// ── Observations ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BleDeviceObservation {
    pub device_id: String,
    pub name: Option<String>,
    pub rssi_dbm: i16,
    pub service_uuids: Vec<String>,
    pub phy: Option<BlePhy>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BleObservation {
    Device {
        observation_id: Uuid,
        timestamp_utc: String,
        device: BleDeviceObservation,
    },
    EnvironmentSampleRef {
        sample_id: Uuid,
    },
}

// ── Environment Snapshot ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BleEnvironmentSample {
    pub sample_id: Uuid,
    pub timestamp_utc: String,
    pub device_count: u32,
    pub max_rssi_dbm: i16,
    pub avg_rssi_dbm: Option<f32>,
}

// ── Structural Invariants (no policy) ──────────────────────────────────────────

impl BleLinkParams {
    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.conn_interval_ms == 0 {
            return Err("conn_interval_ms must be > 0".into());
        }
        if self.max_pdu_bytes == 0 {
            return Err("max_pdu_bytes must be > 0".into());
        }
        Ok(())
    }
}

impl BleIntent {
    pub fn validate_invariants(&self) -> Result<(), String> {
        match self {
            Self::Scan { .. } => Ok(()),
            Self::Connect {
                class_id,
                device_id,
            } => {
                if class_id.is_empty() {
                    return Err("class_id must not be empty".into());
                }
                if device_id.is_empty() {
                    return Err("device_id must not be empty".into());
                }
                Ok(())
            }
            Self::SubscribeCharacteristic {
                class_id,
                device_id,
                service_uuid,
                characteristic_uuid,
            }
            | Self::WriteCharacteristic {
                class_id,
                device_id,
                service_uuid,
                characteristic_uuid,
                ..
            } => {
                if class_id.is_empty() {
                    return Err("class_id must not be empty".into());
                }
                if device_id.is_empty() {
                    return Err("device_id must not be empty".into());
                }
                let _ = uuid::Uuid::parse_str(service_uuid)
                    .map_err(|e| format!("service_uuid must be a valid UUID: {e}"))?;
                let _ = uuid::Uuid::parse_str(characteristic_uuid)
                    .map_err(|e| format!("characteristic_uuid must be a valid UUID: {e}"))?;
                Ok(())
            }
        }
    }
}
