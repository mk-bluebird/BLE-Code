#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// High-level BLE action an AI or host wants to perform.
/// Purely descriptive, non-actuating.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BleIntent {
    /// Request a scan over the RF environment for devices matching a class/profile.
    Scan {
        /// Optional logical device class identifier (e.g., "openbci-cyton-nus").
        class_id: Option<String>,
    },

    /// Attempt to connect to a discovered device.
    Connect {
        /// Logical device class identifier, stable across sessions.
        class_id: String,
        /// Device identifier (MAC or opaque ID from adapter).
        device_id: String,
    },

    /// Subscribe to notifications/indications on a given characteristic.
    SubscribeCharacteristic {
        class_id: String,
        device_id: String,
        /// 128-bit service UUID in canonical string form.
        service_uuid: String,
        /// 128-bit characteristic UUID in canonical string form.
        characteristic_uuid: String,
    },

    /// Write some payload length (bytes) to a characteristic.
    /// Payload contents are handled at higher layers / adapters.
    WriteCharacteristic {
        class_id: String,
        device_id: String,
        service_uuid: String,
        characteristic_uuid: String,
        /// Intended payload length in bytes (actual payload may be carried separately).
        payload_len: usize,
    },
}

/// PHY values supported by the model. These are stringly-typed in ALN shards,
/// but in Rust we keep them as a closed enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlePhy {
    Le1M,
    Le2M,
    LeCodedS2,
    LeCodedS8,
}

/// Negotiated / desired link parameters for a BLE connection.
/// This is purely descriptive; enforcement lives in `ble-governance`/`ble-guard`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BleLinkParams {
    /// PHY to use for the link.
    pub phy: BlePhy,
    /// Whether the link is encrypted at the BLE layer.
    pub encrypted: bool,
    /// Whether MIC protection is enabled.
    pub mic_present: bool,
    /// Whether the link is bonded (long-term keys).
    pub bonded: bool,
    /// Connection interval in milliseconds.
    pub conn_interval_ms: u32,
    /// Max PDU size in bytes.
    pub max_pdu_bytes: u16,
    /// Whether Constant Tone Extension (direction finding) is used.
    pub cte_present: bool,
}

/// Summary of a device observed during a scan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BleDeviceObservation {
    /// Opaque device identifier (e.g., MAC address or platform-specific handle).
    pub device_id: String,
    /// Human-readable name if available.
    pub name: Option<String>,
    /// Current RSSI in dBm.
    pub rssi_dbm: i16,
    /// List of advertised service UUIDs (string form).
    pub service_uuids: Vec<String>,
    /// PHY used during this observation, if known.
    pub phy: Option<BlePhy>,
}

/// A higher-level observation event, which can represent a single device,
/// or a batch snapshot when used with `ble-env-ingest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BleObservation {
    Device {
        observation_id: Uuid,
        timestamp_utc: String,
        device: BleDeviceObservation,
    },
    /// Reserved for future environment-level observations.
    /// (e.g., aggregated density, noise floor).
    EnvironmentSampleRef {
        sample_id: Uuid,
    },
}

impl BleLinkParams {
    /// Minimal non-policy invariants:
    /// - conn_interval_ms > 0
    /// - max_pdu_bytes > 0
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
    /// Cheap sanity checks that do NOT encode neurorights or RoH policy.
    /// They only ensure identifiers are non-empty and UUIDs are parsable.
    pub fn validate_invariants(&self) -> Result<(), String> {
        match self {
            BleIntent::Scan { .. } => Ok(()),
            BleIntent::Connect { class_id, device_id } => {
                if class_id.is_empty() {
                    return Err("class_id must not be empty".into());
                }
                if device_id.is_empty() {
                    return Err("device_id must not be empty".into());
                }
                Ok(())
            }
            BleIntent::SubscribeCharacteristic {
                class_id,
                device_id,
                service_uuid,
                characteristic_uuid,
            }
            | BleIntent::WriteCharacteristic {
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
                // UUID parsing is a structural check only.
                let _ = uuid::Uuid::parse_str(service_uuid).map_err(|e| {
                    format!("service_uuid must be a valid UUID: {e}")
                })?;
                let _ = uuid::Uuid::parse_str(characteristic_uuid).map_err(|e| {
                    format!("characteristic_uuid must be a valid UUID: {e}")
                })?;
                Ok(())
            }
        }
    }
}
