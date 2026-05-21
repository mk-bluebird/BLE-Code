#![forbid(unsafe_code)]

use ble_model::{BleDeviceObservation, BleEnvironmentSample, BlePhy};
use serde::Deserialize;
use uuid::Uuid;

pub mod aln_append;

pub use aln_append::append_to_aln;

/// Shape of Termux/CLI JSON scan results.
/// This should match the actual JSON from your Termux wrapper.
#[derive(Debug, Deserialize)]
struct RawScanDevice {
    address: String,
    name: Option<String>,
    rssi: i16,
    #[serde(default)]
    service_uuids: Vec<String>,
    #[serde(default)]
    phy: Option<String>,
}

/// Parse a JSON array of scan devices into BleDeviceObservation objects.
///
/// Example input (Termux):
/// `[{"address":"AA:BB:CC:DD:EE:FF","name":"Device","rssi":-60,"service_uuids":["..."]}]`
pub fn parse_scan_json(raw: &str) -> Result<Vec<BleDeviceObservation>, serde_json::Error> {
    let devices: Vec<RawScanDevice> = serde_json::from_str(raw)?;
    let mapped = devices
        .into_iter()
        .map(|d| BleDeviceObservation {
            device_id: d.address,
            name: d.name,
            rssi_dbm: d.rssi,
            service_uuids: d.service_uuids,
            phy: d.phy.and_then(parse_phy),
        })
        .collect();
    Ok(mapped)
}

/// Summarize a set of observations into a BleEnvironmentSample.
pub fn summarize_sample(
    observations: &[BleDeviceObservation],
    timestamp_utc: String,
) -> BleEnvironmentSample {
    let sample_id = Uuid::new_v4();
    let device_count = observations.len() as u32;

    let mut max_rssi: i16 = i16::MIN;
    let mut sum_rssi: i64 = 0;

    for d in observations {
        if d.rssi_dbm > max_rssi {
            max_rssi = d.rssi_dbm;
        }
        sum_rssi += d.rssi_dbm as i64;
    }

    let avg_rssi_dbm = if device_count == 0 {
        None
    } else {
        Some(sum_rssi as f32 / device_count as f32)
    };

    BleEnvironmentSample {
        sample_id,
        timestamp_utc,
        device_count,
        max_rssi_dbm: if device_count == 0 { 0 } else { max_rssi },
        avg_rssi_dbm,
    }
}

fn parse_phy(s: String) -> Option<BlePhy> {
    match s.as_str() {
        "LE1M" => Some(BlePhy::Le1M),
        "LE2M" => Some(BlePhy::Le2M),
        "LECodedS2" | "LECODEDS2" => Some(BlePhy::LeCodedS2),
        "LECodedS8" | "LECODEDS8" => Some(BlePhy::LeCodedS8),
        _ => None,
    }
}
