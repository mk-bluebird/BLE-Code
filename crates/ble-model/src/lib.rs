// File: crates/ble-model/src/lib.rs

#![forbid(unsafe_code)]
//! BLE intent and observation models for neurorights-aware BLE workflows.
//!
//! This crate provides pure data models for BLE intents, link parameters,
//! and observations. All types are non-actuating and safe to serialize.
#![allow(missing_docs)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::doc_markdown)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── BLE Intent ────────────────────────────────────────────────────────────────

/// High-level BLE action an AI or host wants to perform.
/// Purely descriptive, non-actuating.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BleIntent {
    /// Scan for BLE devices, optionally filtered by device class.
    Scan {
        class_id: Option<String>,
    },
    /// Connect to a specific BLE device.
    Connect {
        class_id: String,
        device_id: String,
    },
    /// Subscribe to notifications from a BLE characteristic.
    SubscribeCharacteristic {
        class_id: String,
        device_id: String,
        service_uuid: String,
        characteristic_uuid: String,
    },
    /// Write data to a BLE characteristic.
    WriteCharacteristic {
        class_id: String,
        device_id: String,
        service_uuid: String,
        characteristic_uuid: String,
        payload_len: usize,
    },
}

// ── PHY & Link Parameters ─────────────────────────────────────────────────────

/// Bluetooth Low Energy PHY (physical layer) mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlePhy {
    /// 1 Mbps uncoded PHY (Bluetooth 4.x baseline).
    Le1M,
    /// 2 Mbps uncoded PHY (Bluetooth 5.0+).
    Le2M,
    /// Coded PHY with S=2 (longer range, Bluetooth 5.0+).
    LeCodedS2,
    /// Coded PHY with S=8 (maximum range, Bluetooth 5.0+).
    LeCodedS8,
}

/// BLE connection link parameters observed or required.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BleLinkParams {
    /// Physical layer mode.
    pub phy: BlePhy,
    /// Whether the link is encrypted (LE Privacy).
    pub encrypted: bool,
    /// Whether Message Integrity Check is enabled.
    pub mic_present: bool,
    /// Whether devices are bonded (persistent pairing).
    pub bonded: bool,
    /// Connection interval in milliseconds.
    pub conn_interval_ms: u32,
    /// Maximum Protocol Data Unit size in bytes.
    pub max_pdu_bytes: u16,
    /// Whether Constant Tone Extension is present (direction finding).
    pub cte_present: bool,
}

// ── Observations ───────────────────────────────────────────────────────────────

/// Observation of a single BLE device during a scan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BleDeviceObservation {
    /// Device identifier (MAC address or other stable ID).
    pub device_id: String,
    /// Human-readable device name (from advertising data).
    pub name: Option<String>,
    /// Received Signal Strength Indicator in dBm.
    pub rssi_dbm: i16,
    /// List of advertised service UUIDs.
    pub service_uuids: Vec<String>,
    /// PHY used for advertising (if known).
    pub phy: Option<BlePhy>,
}

/// Top-level BLE observation event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BleObservation {
    /// Single device observation.
    Device {
        observation_id: Uuid,
        timestamp_utc: String,
        device: BleDeviceObservation,
    },
    /// Reference to an environment sample (aggregated scan result).
    EnvironmentSampleRef {
        sample_id: Uuid,
    },
}

// ── Environment Snapshot ───────────────────────────────────────────────────────

/// Aggregated BLE environment snapshot (multiple devices).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BleEnvironmentSample {
    /// Unique sample identifier.
    pub sample_id: Uuid,
    /// ISO 8601 timestamp of sample capture.
    pub timestamp_utc: String,
    /// Number of unique devices observed.
    pub device_count: u32,
    /// Maximum RSSI observed across all devices.
    pub max_rssi_dbm: i16,
    /// Average RSSI across all devices (if computed).
    pub avg_rssi_dbm: Option<i16>,
}

// ── Structural Invariants (no policy) ──────────────────────────────────────────

impl BleLinkParams {
    /// Validates basic structural invariants (not policy).
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

impl BleEnvironmentSample {
    /// Validates basic structural invariants.
    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.device_count == 0 && self.avg_rssi_dbm.is_some() {
            return Err("avg_rssi_dbm must be None when device_count is 0".into());
        }
        Ok(())
    }
}

impl BleIntent {
    /// Validates basic structural invariants (not policy).
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
                Uuid::parse_str(service_uuid)
                    .map_err(|e| format!("service_uuid must be a valid UUID: {e}"))?;
                Uuid::parse_str(characteristic_uuid)
                    .map_err(|e| format!("characteristic_uuid must be a valid UUID: {e}"))?;
                Ok(())
            }
        }
    }
}
