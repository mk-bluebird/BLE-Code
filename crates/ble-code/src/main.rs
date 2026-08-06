// crates/ble-code/src/main.rs
#![forbid(unsafe_code)]

use anyhow::Result;
use ble_code::{BleScanner, BleDeviceInfo};

fn main() -> Result<()> {
    env_logger::init();

    let scanner = BleScanner::new();
    let devices: Vec<BleDeviceInfo> = scanner.scan()?;

    if devices.is_empty() {
        println!("BLE-Code: no devices found (dummy scanner).");
    } else {
        for dev in devices {
            println!(
                "BLE device {:?} at {} (RSSI: {}), name: {:?}, last seen: {}",
                dev.id,
                dev.address,
                dev.rssi,
                dev.name,
                dev.last_seen,
            );
        }
    }

    Ok(())
}
