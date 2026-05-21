#![forbid(unsafe_code)]

//! btleplug → BleIntent/BleLinkParams adapter.
//!
//! This crate never calls OS Bluetooth APIs directly. It only:
//! - Translates btleplug discovery/connect events into BleIntent.
//! - Builds BleLinkParams from btleplug link properties.

#[cfg(feature = "btleplug")]
pub mod mapping;

#[cfg(feature = "btleplug")]
pub use mapping::*;
