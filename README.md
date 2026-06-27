# BLE‑Code — Governance‑First Bluetooth Low Energy for AI‑Safe Augmented Citizens

**BLE‑Code** is a public, AI‑friendly Rust workspace that provides a non‑actuating policy engine, data models, adapter patterns, and governance tooling for neurorights‑compliant BLE interactions. It is designed to be the canonical, secure interface that any AI chat, agent, or tool can use when building or reasoning about BLE connectivity for augmented‑citizen endpoints.

## Why this repo exists

- **Neurorights & Rights of Humanity (RoH) first:** every BLE intent (scan, connect, subscribe, write) must pass through a guard that enforces subject‑specific RoH ceilings and service‑level policies.
- **AI‑safe by construction:** core crates are non‑actuating; adapters only translate events but never call OS BLE APIs directly. The guard is always consulted before any platform‑specific action.
- **Portable & platform‑agnostic:** the core model and guard have zero OS dependencies. Platform adapters (btleplug, Android FFI) are feature‑gated and live in separate crates.
- **Governance as code:** CI workflows and custom tools (`workspace‑lint`, `ci‑lint‑github`) enforce structural, security, and rollback‑prevention rules automatically.

## Repository structure

```
BLE-Code/
├── Cargo.toml               # workspace root
├── rust-toolchain.toml      # pinned toolchain
├── README.md
├── .gitignore
├── LICENSE
├── crates/
│   ├── ble-core/            # GATT models, UUIDs, ALN bindings (typed surfaces)
│   ├── ble-security/        # Pairing, bonding, link-level policy abstraction
│   ├── ble-host/            # Central role logic, scanning, secure connect
│   ├── ble-peripheral/      # Peripheral role implementation
│   ├── ble-model/           # data types (BleIntent, BleLinkParams, …)
│   ├── ble-governance/      # policy objects & RoH invariants
│   ├── ble-guard/           # non‑actuating decision engine
│   ├── ble-env-ingest/      # scan/environment ingestion (Termux/CLI)
│   ├── ble-adapter-btleplug/# btleplug ↔ BleIntent mapping
│   ├── ble-android-ffi/     # JSON‑over‑FFI bridge for Android
│   ├── aln-core/            # ALN parsing core
│   ├── ble-tools-workspace-lint/   # Cargo.toml hygiene checker
│   └── ble-tools-ci-lint-github/   # GitHub workflow governance linter
├── schemas/                 # ALN & JSON schemas for profiles, sessions, telemetry
├── playbooks/               # Machine‑readable BLE sequences (NUS, OpenBCI)
├── docs/                    # AI usage guides, Android/Kotlin reference, REPOSITORY_STRUCTURE.md
├── configs/                 # Example guard configurations
├── examples/                # Runnable demos & sample telemetry
└── .github/workflows/       # CI (Rust + Android)
```

## Quick start for AI agents

When you (an AI) are asked to generate BLE code:

1. **Import the typed surfaces** from `ble-core` (`BleUuid`, `GattService`, `GattCharacteristic`, `CharSecurity`) and `ble-security` (`BleSecurityPolicy`, `BleLinkContext`).
2. **Build typed GATT structures**—never use stringly-typed UUIDs. Use `BleUuid::from_uuid()` or well-known constants like `BleUuid::DEVICE_INFORMATION`.
3. **Enforce security at type level**: Use `CharSecurity::cybercore_default()` for any characteristic that touches cybercore/neuromotor/bioscale state.
4. **Use the `BleRadio` trait** (from `ble-host`) or `BlePeripheral` trait (from `ble-peripheral`) for platform-agnostic operations—both require `BleSecurityPolicy` on all connect/write operations.
5. **Validate profiles** using `BleProfile::validate_all_services()` before deployment.
6. **Bind sovereignty**: Ensure `host_did` and `bostrom_addr` in your ALN profile match runtime values before enabling services.

See `docs/REPOSITORY_STRUCTURE.md` for the full crate overview and `docs/ai-usage-perplexity-ble-guard-v1.md` for a concrete, low‑energy agent contract.

## Getting started (local development)

```bash
# Clone the repo
git clone https://github.com/mk-bluebird/BLE-Code.git
cd BLE-Code

# Ensure the correct Rust toolchain is installed (1.85.0)
rustup show

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Lint
cargo clippy --all-targets --all-features -- -D warnings
```

## Governance

This repository is governed by `repo-governance.aln` and `AiContributionPolicy.aln`. All changes, especially those from AI, must satisfy:

- CI must be green (workspace‑hygiene, fmt, clippy, test, governance‑lint).
- No removal of required safety jobs.
- No widening of RoH ceilings beyond 0.3.
- AI‑authored crates must declare the `ai-authored` marker and use the workspace Clippy profile.

## License

Licensed under either of

- MIT license ([LICENSE](LICENSE))
- Apache License, Version 2.0 ([LICENSE](LICENSE))

at your option.

## Contact

This is a sovereign‑augmented‑citizen project. For policy questions, refer to the `Data_Lake` repository (private). For public technical discussion, open an issue on this repo.
