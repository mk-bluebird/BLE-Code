# BLE Governance Flow – BLE-Code

```mermaid
flowchart LR
    subgraph Agent["AI / Augmented Citizen"]
        A1["BLE Intent\n(BleIntent)"]
        A2["Consent Profile\n(NeurorightsTag, RoH ceiling)"]
    end

    subgraph BLE_Model["ble-model"]
        M1["BleIntent\n(Scan, Connect, Subscribe, Write)"]
        M2["BleObservation\n(Device, Environment)"]
    end

    subgraph Governance["ble-governance"]
        G1["BleProfileShard\n(subject, deviceclasspolicies, servicepolicies)"]
        G2["GovernanceInvariantError\n(EmptySubjectId, RoH ceiling, weights)"]
        G3["validate_invariants()"]
    end

    subgraph Guard["ble-guard"]
        D1["BleGuard\n(governed adapter)"]
        D2["find_device_class() / find_service()"]
        D3["RoH Accumulator\n(sum service.roh_weight)"]
        D4["Decision\n(Allowed / Denied)"]
    end

    subgraph Adapter["ble-adapter-btleplug / ble-android-ffi"]
        H1["btleplug / Android FFI"]
        H2["System BLE Stack\n(DBus, BlueZ, Android BLE)"]
    end

    Agent -->|Form BleIntent| M1
    M1 -->|Governance check| G3
    G3 -->|Ok| D1
    G3 -->|Err(GovernanceInvariantError)| D4

    D1 -->|Resolve policy| D2
    D2 --> D3
    D3 -->|RoH ≤ subject.roh_ceiling| D4
    D3 -->|RoH > ceiling| D4

    D4 -->|Allowed| H1
    D4 -->|Denied| Agent

    H1 --> H2
    H2 -->|Scan/Notify| M2
    M2 --> Agent
```

This diagram is documentation‑only and does not affect compilation.[file:60]  
It mirrors the existing `BleProfileShard`, `GovernanceInvariantError`, and `BleGuard` roles you already use in `crates/ble-governance` and `crates/ble-guard`.[file:59]

---

## 2. BLE IP/Port + Stack Tables

Destination:

- `docs/tables/ble-ip-stack-tables.md`

Content:

```markdown
# BLE-Code IP / Stack Tables

## Host BLE Stack Ports (Linux CI / Dev)

| Component              | Protocol | Port / Bus     | Notes                                                                 |
|------------------------|----------|----------------|-----------------------------------------------------------------------|
| BlueZ dbus-daemon      | D-Bus    | Unix socket    | `libdbus-1-dev` required in CI; used by `btleplug` on Linux.         |
| BLE HCI (Linux)        | HCI      | /dev/hciX      | Kernel-managed; accessed via BlueZ, not directly by BLE-Code.        |
| BLE Adapter UDev       | udev     | Netlink        | `libudev-dev` required in CI for adapter enumeration.                |

## GitHub CI Network / System Dependencies

| Job             | Package                 | Purpose                                |
|-----------------|-------------------------|----------------------------------------|
| workspace-hygiene | libdbus-1-dev, pkg-config, libudev-dev | Needed by `btleplug` build scripts in `ble-adapter-btleplug`. |
| clippy / test   | libdbus-1-dev, pkg-config, libudev-dev | Same as above; enforced in `.github/workflows/ci.yml`.        |

## OpenBCI NUS Service UUIDs

| Name             | UUID                                   | Usage                                     |
|------------------|----------------------------------------|-------------------------------------------|
| NUS RX (data)    | 6E400001-B5A3-F393-E0A9-E50E24DCCA9E   | OpenBCI Cyton EEG stream (host reads).    |
| NUS TX (control) | 6E400002-B5A3-F393-E0A9-E50E24DCCA9E   | OpenBCI Cyton commands (host writes).     |
```

These tables are consistent with your existing CI script that installs `libdbus-1-dev`, `pkg-config`, and `libudev-dev` and with the OpenBCI NUS UUIDs used in your Perplexity profile validator.[file:59]

---

## 3. Homomorphic Mapping Index: Intent ↔ Observation ↔ Hex

Destination:

- `docs/indexes/ble-intent-observation-index.md`

Content:

```markdown
# BLE Intent / Observation Mapping Index

This index aligns `BleIntent`, `BleDeviceObservation`, and hex-level payload patterns for NUS / generic BLE services.

## BLE Intents

| Intent Variant           | Fields                                              | Typical Usage                           |
|--------------------------|-----------------------------------------------------|-----------------------------------------|
| `Scan`                   | `classid: Option<String>`                          | Discover devices under a profile.       |
| `Connect`                | `classid: String`, `deviceid: String`              | Establish governed connection.          |
| `SubscribeCharacteristic` | `classid`, `deviceid`, `serviceuuid`, `characteristicuuid` | Begin notifications on a stream. |
| `WriteCharacteristic`    | `classid`, `deviceid`, `serviceuuid`, `characteristicuuid`, `payload_len: usize` | Send control / config commands. |

## BLE Observations

| Struct                  | Key Fields                                             | Notes                                      |
|-------------------------|--------------------------------------------------------|--------------------------------------------|
| `BleDeviceObservation`  | `deviceid: String`, `name: Option<String>`, `rssidbm: i16`, `serviceuuids: Vec<String>`, `phy: Option<BlePhy>` | Single scan result.        |
| `BleEnvironmentSample`  | `sampleid: Uuid`, `devicecount: u32`, `maxrssidbm: i16`, `avgrssidbm: Option<f32>` | Aggregated environment snapshot. |

## Homomorphic Mapping Table (OpenBCI NUS)

| Layer        | Symbol / Field                      | Hex / Wire Representation                        | Schema / Contract                        |
|-------------|--------------------------------------|--------------------------------------------------|------------------------------------------|
| Intent      | `BleIntent::SubscribeCharacteristic` | GATT Subscribe on NUS RX characteristic          | `schemas/bci-stream-profile.openbci-cyton.v1.aln` frame profile. |
| Intent      | `BleIntent::WriteCharacteristic`     | NUS TX ASCII command byte (e.g., `b`, `s`, `v`)  | `OpenBciCommandProfileV1.cmd` + `rohweight`. |
| Observation | `BleDeviceObservation.rssidbm`       | RSSI in dBm encoded as signed 8/16‑bit integer   | Canon type `RssiDbm` (`i16`) in device‑type canon. |
| Observation | `BleDeviceObservation.serviceuuids`  | List of 128‑bit UUIDs, LE byte order             | Must include NUS RX/TX UUIDs for OpenBCI. |
```

This doc is aligned with your existing `BleIntent`, `BleDeviceObservation`, and the `BciStreamProfileOpenBciCytonV1` schema.[file:59][file:58]

---

## 4. Example Hex Tables for OpenBCI NUS

Destination:

- `docs/tables/openbci-nus-hex-table.md`

Content:

```markdown
# OpenBCI Cyton over NUS – Hex Tables

These examples are illustrative and must still respect the RoH ceilings in `BciStreamProfileOpenBciCytonV1`.

## ASCII Command Bytes (TX Characteristic)

| Command               | Byte (char) | Hex  | Description                              | Neurorights Tag      |
|-----------------------|------------|------|------------------------------------------|----------------------|
| StartStream           | `b`        | 0x62 | Begin streaming EEG frames.             | `BciRawSignal`       |
| StopStream            | `s`        | 0x73 | Stop EEG streaming.                     | `BciRawSignal`       |
| SoftReset             | `v`        | 0x76 | Reset board state.                      | `BciBasicTelemetry`  |
| ImpedanceStart        | `z`        | 0x7A | Start impedance test.                   | `BciRawSignal`       |
| ImpedanceStop         | `Z`        | 0x5A | Stop impedance test.                    | `BciRawSignal`       |
| SetSampleRate250      | `F`        | 0x46 | Set 250 Hz sampling.                    | `BciRawSignal`       |
| SetSampleRate500      | `G`        | 0x47 | Set 500 Hz sampling.                    | `BciRawSignal`       |
| SetSampleRate1000     | `H`        | 0x48 | Set 1000 Hz sampling.                   | `BciRawSignal`       |

## Example ASCII Data Frame Prefix (RX Characteristic)

| Pattern         | Hex Sequence          | Meaning                          |
|----------------|-----------------------|----------------------------------|
| EEG data frame | `0x24 0x44` (`$D`)    | ASCII frame header for samples. |
| End of frame   | `0xA` (`\\n`)         | Line terminator.                 |

## Sample RSSI Encoding

| Field                   | Value      | Wire Hex | Notes                         |
|-------------------------|-----------:|---------:|-------------------------------|
| `BleDeviceObservation.rssidbm` | `-65` dBm | `0xBF`  | Fits in signed 8‑bit range.   |
```

These tables are compatible with your Cyton stream profile schema (`OpenBciCommandProfileV1`, `BciNeurorightsTagV1`).[file:60]

---

## 5. Mapping Index File for BLE Types (Rust‑side)

Destination:

- `crates/ble-model/src/ble_type_index.rs`

This is a pure‑Rust, no‑unsafe, no‑unwrap helper that gives a “type index” you can reuse when you add a device‑type canon schema.[file:58][file:59]

```rust
//! BLE type index for documentation and test datasets.
//!
//! This module does not perform I/O. It provides a small, in-memory
//! mapping between high-level BLE concepts and their canonical Rust types.

#![forbid(unsafe_code)]
#![allow(missing_docs)]

use std::collections::BTreeMap;

/// Canonical signal / field kinds used throughout BLE-Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum BleFieldKind {
    RssiDbm,
    ServiceUuid,
    DeviceId,
    SubjectId,
}

/// Canonical Rust type information for a given `BleFieldKind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BleFieldTypeInfo {
    pub rust_type: &'static str,
    pub signed: bool,
    pub bits: u16,
    pub zero_is_valid: bool,
    pub description: &'static str,
}

/// Build a static mapping from `BleFieldKind` to `BleFieldTypeInfo`.
pub fn build_ble_field_type_index() -> BTreeMap<BleFieldKind, BleFieldTypeInfo> {
    use BleFieldKind::*;

    let mut map = BTreeMap::new();

    map.insert(
        RssiDbm,
        BleFieldTypeInfo {
            rust_type: "i16",
            signed: true,
            bits: 16,
            zero_is_valid: false,
            description: "Received Signal Strength Indicator in dBm.",
        },
    );

    map.insert(
        ServiceUuid,
        BleFieldTypeInfo {
            rust_type: "String",
            signed: false,
            bits: 128,
            zero_is_valid: false,
            description: "128-bit service UUID in canonical string form.",
        },
    );

    map.insert(
        DeviceId,
        BleFieldTypeInfo {
            rust_type: "String",
            signed: false,
            bits: 0,
            zero_is_valid: false,
            description: "Opaque device identifier derived from BLE address.",
        },
    );

    map.insert(
        SubjectId,
        BleFieldTypeInfo {
            rust_type: "String",
            signed: false,
            bits: 0,
            zero_is_valid: false,
            description: "Logical subject identifier in BLE governance profiles.",
        },
    );

    map
}
```

You can wire this into tests or future ALN‑backed type‑canon tooling without changing public APIs.[file:58]

---

## 6. Example Dataset for BLE Environment Samples

Destination:

- `examples/datasets/ble-environment-sample.ndjson`

Content (aligned with your `BleEnvironmentSample` struct and invariants).[file:59]

```ndjson
{"sampleid":"00000000-0000-0000-0000-000000000001","timestamputc":"2026-05-26T20:10:00Z","devicecount":0,"maxrssidbm":-128,"avgrssidbm":null}
{"sampleid":"00000000-0000-0000-0000-000000000002","timestamputc":"2026-05-26T20:10:05Z","devicecount":2,"maxrssidbm":-45,"avgrssidbm":-52.5}
{"sampleid":"00000000-0000-0000-0000-000000000003","timestamputc":"2026-05-26T20:10:10Z","devicecount":1,"maxrssidbm":-60,"avgrssidbm":-60.0}
```

These rows respect your invariant that `avgrssidbm` must be `null` when `devicecount == 0`.[file:59]

---

## 7. IP‑Table‑Style View for ALN / CI Assets

Destination:

- `docs/tables/ble-code-asset-ports.md`

Content:

```markdown
# BLE-Code “Asset Port” Table

Logical mapping from repo assets to their “ports” in CI / governance.

| Asset                                  | Type      | “Port” / Entry Point                        | Used By                            |
|----------------------------------------|-----------|---------------------------------------------|------------------------------------|
| `.github/workflows/ci.yml`             | CI YAML   | GitHub Actions `ci` workflow                | Rustfmt, Clippy, tests, gov-lints. |
| `.github/workflows/perplexity-ci.yml`  | CI YAML   | `perplexity-ci` workflow trigger on profile | Perplexity BLE guard assets.       |
| `schemas/bci-stream-profile.openbci-cyton.v1.aln` | ALN schema | `openbci-cyton-over-nus.ble-playbook.aln`   | BCI stream constraints for NUS.    |
| `tools/check-perplexity-profile.sh`    | Shell     | Perplexity profile validation job           | Ensures RoH, UUIDs, security flags.|
| `crates/ble-governance/src/lib.rs`     | Rust lib  | BleProfileShard invariants, RoH envelope    | All BLE governance decisions.      |
| `crates/ble-guard/src/lib.rs`          | Rust lib  | BleGuard decision engine                     | Runtime enforcement before BLE I/O.|
