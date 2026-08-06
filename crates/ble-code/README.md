# ble-code

`ble-code` is the core BLE scanning and device metadata crate for the **BLE-Code** repository. It provides:

- `BleDeviceInfo` – compact, serializable metadata for BLE devices (ID, address, name, RSSI, last_seen).
- `BleScanner` – abstraction over BLE scanning, with a `new()` constructor and a `scan()` method.

This crate is wired to **CyberFS/Cybercore** via:

```toml
ble-code = { git = "https://github.com/mk-bluebird/BLE-Code", package = "ble-code" }
```

in the `ble_code_core` wrapper crate, ensuring cross-repository / constellation correctness between BLE-Code and CyberFS.

## Crate Layout

- `src/lib.rs`  
  Defines:

  - `BleDeviceInfo` – BLE device representation with `Uuid` and `chrono::DateTime<Utc>` fields.
  - `BleScanner` – BLE scanner abstraction, with a dummy `scan()` implementation for now.

- `src/main.rs`  
  Simple CLI that:

  - Constructs a `BleScanner`.
  - Performs a scan and prints devices (or a message if none are returned).

## Usage in BLE-Code

Inside the BLE-Code repository, `ble-code` can be extended to integrate with:

- Platform-specific BLE stacks (Linux BlueZ, macOS CoreBluetooth, Windows BLE APIs).
- Device filters, authentication, or nanorobotics-specific BLE payloads.

Example:

```rust
use ble_code::{BleScanner, BleDeviceInfo};

fn list_devices() -> anyhow::Result<()> {
    let scanner = BleScanner::new();
    let devices: Vec<BleDeviceInfo> = scanner.scan()?;
    for dev in devices {
        println!(
            "BLE device {:?} at {} (RSSI: {})",
            dev.id, dev.address, dev.rssi,
        );
    }
    Ok(())
}
```

## Usage from CyberFS

In `crates/ble_code_core` within CyberFS:

```rust
use ble_code_core::BleCoreHandle;
use ble_code::BleScanner;

fn example() -> anyhow::Result<()> {
    let scanner = BleScanner::new();
    let handle = BleCoreHandle::new(scanner);

    let devices = handle.scan_devices()?;
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

    Ok(())
}
```

## Building

From the BLE-Code repository root:

```bash
cd crates/ble-code
cargo build
cargo test
```

`ble-code` is intended to be a stable, shared BLE crate for the BLE-Code repository and its constellation (CyberFS, Cybercore, and related mk-bluebird repositories), ensuring consistent wiring for BLE device metadata and scan interfaces.
