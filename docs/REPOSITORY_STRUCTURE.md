# BLE-Code Repository Structure

This document describes the machine-readable, AI-chat compatible structure of the BLE-Code repository.

## Overview

BLE-Code is organized as a Rust workspace with clearly typed surfaces and no ad-hoc text configs. Everything important (services, characteristics, policies) is described twice:

1. As Rust types (for code)
2. As ALN schemas (for AI agents and tooling)

This makes the repo "self-describing" and machine-readable end-to-end.

## Directory Structure

```
ble-code/
├── Cargo.toml                    # Workspace definition
├── crates/
│   ├── ble-core/                 # GATT models, UUIDs, ALN bindings
│   ├── ble-security/             # Pairing, bonding, link-level policy abstraction
│   ├── ble-host/                 # Central role logic, scanning, secure connect
│   ├── ble-peripheral/           # Peripheral role implementation
│   ├── ble-model/                # Data types (BleIntent, BleLinkParams, observations)
│   ├── ble-governance/           # Policy objects & RoH invariants
│   ├── ble-guard/                # Non-actuating decision engine
│   ├── ble-env-ingest/           # Scan/environment ingestion
│   ├── ble-adapter-btleplug/     # btleplug ↔ BleIntent mapping
│   ├── ble-android-ffi/          # JSON-over-FFI bridge for Android
│   ├── aln-core/                 # ALN parsing core
│   └── [tooling crates...]       # CI, lint, governance tools
├── schemas/                      # ALN and machine-readable BLE profiles
├── playbooks/                    # Machine-readable BLE sequences
├── docs/                         # Documentation
├── examples/                     # Runnable demos
└── .github/workflows/            # CI with lint, tests
```

## Core Crates

### `ble-core`

GATT model and profiles with typed representations:

- `BleUuid` - Newtype wrapper for 128-bit BLE UUIDs (no stringly-typed UUIDs)
- `BleServiceKind` - Enum for standard and custom services
- `GattService` - Typed service with characteristics
- `GattCharacteristic` - Typed characteristic with properties and security
- `CharProperties` - Bitflags for READ, WRITE, NOTIFY, INDICATE
- `CharSecurity` - Security requirements (encryption, authentication, MITM)
- `BleProfile` - Complete profile with DID and Bostrom address binding

Key features:
- `#![forbid(unsafe_code)]`
- Cybercore security defaults enforced at type level
- Validation methods for security invariants

### `ble-security`

Pairing, bonding, and policy abstraction:

- `BleSecurityLevel` - Enum: NoSecurity, EncryptedNoMitm, EncryptedWithMitm
- `BleSecurityPolicy` - Policy with min_level, bond_on_success, allow_legacy_pairing
- `BleLinkContext` - Runtime link state (peer_id, security_level, bonded)
- `enforce_policy_link()` - Validate link meets policy
- `request_secure_connection()` - Initiate secure connection

Key features:
- No raw pairing modes exposed
- Safe defaults encoded in policy types
- Cybercore default requires EncryptedWithMitm

### `ble-host`

Central role logic with platform abstraction:

- `BleRadio` trait - Platform-agnostic radio interface
- `ScanFilter` - Typed scan filters
- `PeerDescriptor` - Discovered peer information
- `BleHost<R>` - High-level host combining scanning and connections

All operations require `BleSecurityPolicy` parameter.

### `ble-peripheral`

Peripheral role implementation:

- `BlePeripheral` trait - Platform-agnostic peripheral interface
- `PeripheralConfig` - Advertising configuration
- `BlePeripheralServer<P>` - High-level peripheral server

All connections require security policy enforcement.

## Machine-Readable Schemas

Located in `schemas/`:

- `ble-profile.schema.v1.aln` - ALN schema for BLE profiles
- `perplexity-ble-guard-v1.profile.aln` - Example profile instance
- `bci-stream-profile.openbci-cyton.v1.aln` - BCI device profile
- `tooling-event.v1.aln` - Tooling event schema

ALN profiles define:
- Services and characteristics (UUIDs, security requirements)
- Host DID and Bostrom address binding
- Invariants (RoH ceiling, non-negative weights)

## Security Practices

1. **Secure by construction**: Security policy types, no raw APIs
2. **Cybercore defaults**: Any characteristic touching cybercore/neuromotor/bioscale state requires encrypted, authenticated, MITM-protected access
3. **No raw write APIs**: Always require `BleSecurityPolicy` parameter
4. **Key handling abstraction**: BLE stack manages keys; APIs expose only "link meets policy or not"
5. **Formal checks**: `#![forbid(unsafe_code)]`, CI lints, invariant tests

## AI-Agent Usage Pattern

When an AI agent generates BLE code:

1. Import types from `ble-core` and `ble-security`
2. Instantiate `BleSecurityPolicy` with strict settings (e.g., `cybercore_default()`)
3. Use generated `GattService` and `GattCharacteristic` types, not ad-hoc UUID strings
4. Never bypass security checks
5. Ensure sovereignty binding (host_did and bostrom_address match before enabling services)

Example:

```rust
use ble_core::{BleUuid, GattService, CharSecurity};
use ble_security::BleSecurityPolicy;

// Always use cybercore-default security for sensitive operations
let policy = BleSecurityPolicy::cybercore_default();

// Build typed services, not stringly-typed UUIDs
let service = GattService {
    uuid: BleUuid::DEVICE_INFORMATION,
    kind: BleServiceKind::DeviceInformation,
    characteristics: vec![],
    name: Some("secure_service".to_string()),
};

// Validate security before use
service.validate_cybercore_security()?;
```

## Sovereignty Binding

BLE profiles for cybernetic systems are always:
- Bound to host DID and Bostrom address in ALN
- Checked at runtime (host_did and bostrom_address must match before enabling any BLE services that touch bioscale data)

This ensures AI-assistive coding without exposing low-level BLE pitfalls while maintaining sovereign control.
