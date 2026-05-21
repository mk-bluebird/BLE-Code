#![forbid(unsafe_code)]

use ble_model::{BleDeviceObservation, BleEnvironmentSample, BlePhy};
use std::fs::OpenOptions;
use std::io::{self, Write};
use uuid::Uuid;

/// Append an environment sample and its devices to an append-only
/// `.ble-environment.aln` log file.
///
/// Invariants:
/// - Never truncates the file.
/// - Writes a BleEnvironmentSample record followed by one BleDeviceObservation
///   record per device, all sharing the same sample_id.
pub fn append_to_aln(
    path: &std::path::Path,
    sample: &BleEnvironmentSample,
    devices: &[BleDeviceObservation],
) -> io::Result<()> {
       let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    // 1. Environment sample header.
    let env_record = format_environment_record(sample);
    file.write_all(env_record.as_bytes())?;

    // 2. Device observations, each tagged with sample_id.
    for dev in devices {
        let dev_record = format_device_record(sample.sample_id, dev);
        file.write_all(dev_record.as_bytes())?;
    }

    // Ensure data is durably appended before returning.
    file.flush()?;
    Ok(())
}

fn format_environment_record(sample: &BleEnvironmentSample) -> String {
    let mut out = String::new();

    out.push_str("record BleEnvironmentSample\n");
    out.push_str(&format!("  sample_id {}\n", format_uuid(sample.sample_id)));
    out.push_str(&format!("  timestamp_utc \"{}\"\n", sample.timestamp_utc));
    out.push_str(&format!("  device_count {}\n", sample.device_count));
    out.push_str(&format!("  max_rssi_dbm {}\n", sample.max_rssi_dbm));
    if let Some(avg) = sample.avg_rssi_dbm {
        out.push_str(&format!("  avg_rssi_dbm {:.2}\n", avg));
    }
    out.push('\n');

    out
}

fn format_device_record(sample_id: Uuid, dev: &BleDeviceObservation) -> String {
    let mut out = String::new();

    out.push_str("record BleDeviceObservation\n");
    out.push_str(&format!("  sample_id {}\n", format_uuid(sample_id)));
    out.push_str(&format!("  device_id \"{}\"\n", dev.device_id));
    if let Some(name) = &dev.name {
        out.push_str(&format!("  name \"{}\"\n", escape_quotes(name)));
    }
    out.push_str(&format!("  rssi_dbm {}\n", dev.rssi_dbm));

    // service_uuids as simple JSON-like list, which your existing ALN parsers already support.
    if !dev.service_uuids.is_empty() {
        let uuids = dev
            .service_uuids
            .iter()
            .map(|u| format!("\"{}\"", u))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("  service_uuids [{}]\n", uuids));
    }

    if let Some(phy) = dev.phy {
        out.push_str(&format!("  phy \"{}\"\n", format_phy(phy)));
    }

    out.push('\n');
    out
}

fn format_uuid(id: Uuid) -> String {
    // Use canonical lowercase UUID string; ALN side can type this as uuid.
    id.to_string()
}

fn format_phy(phy: BlePhy) -> &'static str {
    match phy {
        BlePhy::Le1M => "LE1M",
        BlePhy::Le2M => "LE2M",
        BlePhy::LeCodedS2 => "LECodedS2",
        BlePhy::LeCodedS8 => "LECodedS8",
    }
}

fn escape_quotes(s: &str) -> String {
    s.replace('"', "\\\"")
}
