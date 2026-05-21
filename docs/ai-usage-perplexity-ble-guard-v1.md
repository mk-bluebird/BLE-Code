# Perplexity BLE Guard Profile v1

This document defines how the `perplexity-ble-guard-v1` agent must interact with BLE-Code.

## Core principles

- Always construct `BleIntent` and `BleLinkParams` instead of direct OS BLE calls.
- Always call `BleGuard` (or its FFI/HTTP façade) before any `connect`, `subscribe`, or `write`.
- Never widen RoH ceilings or security requirements in generated code.

## Subject profile

- Subject id: `perplexity-ble-guard-v1`.
- RoH ceiling: `0.15` (lower than the human default `0.3`).
- Max parallel BLE devices: `1`.
- Allowed BLE class: `openbci-cyton-nus` under strict security.

## Allowed BLE geometry

- PHY: `LE1M`–`LE2M`.
- Security: `encrypted = true`, `bonded = true`, `mic_present = true`.
- Connection interval: `<= 50 ms`.
- Max PDU: `<= 64 bytes`.
- CTE: not allowed.

## BCI passthrough services

- RX notify (sensor):
  - Service UUID: `6E400001-B5A3-F393-E0A9-E50E24DCCA9E`.
  - Notifications allowed, writes forbidden.
  - `neurorights_tag = BciIntent`, `roh_weight = 0.10`.
  - `passthrough = true` (frames are validated by a higher-level BCI profile).

- TX write (control):
  - Service UUID: `6E400002-B5A3-F393-E0A9-E50E24DCCA9E`.
  - Writes allowed, notifications forbidden.
  - `neurorights_tag = BciIntent`, `roh_weight = 0.05`.
  - `passthrough = true`.

## Low-energy guidance

- Prefer short sessions: target <= 5 minutes for example interactions.
- Encourage negotiated connection interval in the 30–50 ms range.
- Keep MTU modest (<= 64 bytes) unless BCI profile explicitly requires more.

## Code-generation rules for Perplexity

- When asked for BLE code, emit:
  - Construction of `BleIntent` and `BleLinkParams`.
  - A call to `BleGuard` (or `evaluate_ble_guard_json`) before any BLE actuation.
  - Usage of existing playbooks such as:
    - `openbci-cyton-over-nus.ble-playbook.aln`.
- Do not:
  - Invent new BLE flows without a playbook.
  - Change `subject_id`, `roh_ceiling`, or security flags for this profile.
