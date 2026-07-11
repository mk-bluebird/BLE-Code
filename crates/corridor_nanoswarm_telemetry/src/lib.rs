// File: crates/corridor_nanoswarm_telemetry/src/lib.rs
// License: MIT OR Apache-2.0

#![forbid(unsafe_code)]

/// Core nanoswarm telemetry and MAC structures for BLE / LoRaWAN.
/// This crate is designed to integrate with the existing `corridor_core` crate
/// that defines MetricFamily, CorridorBand, HostBudgetBand, EcoBand, and
/// evaluate_corridor().

pub mod payload;
pub mod mac;
pub mod rf_budget;

pub use payload::{MsgType, NanoswarmPayload, Histogram};
pub use mac::{MacParams, MacState, MacDecision};
pub use rf_budget::{RFBudgetSlice, RFBudgetEvaluator};
