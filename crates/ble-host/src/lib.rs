#![forbid(unsafe_code)]
//! Central role logic, scanning, and secure connect for BLE-Code.
//!
//! This crate provides the `BleRadio` trait that platform adapters implement.
//! Application logic only sees the safe traits and security policies.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub use ble_core::{BleUuid, GattCharacteristic, GattService};
pub use ble_security::{BleLinkContext, BleSecurityPolicy};

/// Filter for BLE scanning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanFilter {
    /// Optional service UUIDs to filter by.
    pub service_uuids: Vec<BleUuid>,
    /// Optional device name prefix filter.
    pub name_prefix: Option<String>,
    /// Minimum RSSI threshold (dBm).
    pub min_rssi: Option<i16>,
}

/// Descriptor of a BLE peer device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerDescriptor {
    /// Device identifier (MAC address or other stable ID).
    pub peer_id: String,
    /// Human-readable device name.
    pub name: Option<String>,
    /// Advertised service UUIDs.
    pub service_uuids: Vec<BleUuid>,
    /// RSSI at time of discovery.
    pub rssi: i16,
}

/// Error types for BLE radio operations.
#[derive(Debug, Error)]
pub enum BleRadioError {
    #[error("Scan failed: {0}")]
    ScanFailed(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Service discovery failed: {0}")]
    ServiceDiscoveryFailed(String),

    #[error("Characteristic operation failed: {0}")]
    CharacteristicOperationFailed(String),

    #[error("Security policy violation: {0}")]
    SecurityPolicyViolation(String),

    #[error("Not connected")]
    NotConnected,

    #[error("Timeout")]
    Timeout,
}

/// Result type for BLE radio operations.
pub type BleRadioResult<T> = Result<T, BleRadioError>;

/// Trait abstracting BLE radio operations for central role.
///
/// Platform adapters (Android, Linux BlueZ, Nordic nRF, etc.) implement this trait.
/// Application or cybernetic logic only sees the safe traits and security policy.
pub trait BleRadio {
    /// Start scanning for BLE devices with optional filters.
    fn start_scan(&mut self, filter: ScanFilter) -> BleRadioResult<()>;

    /// Stop an ongoing scan.
    fn stop_scan(&mut self) -> BleRadioResult<()>;

    /// Connect to a peer with the specified security policy.
    fn connect(
        &mut self,
        peer: &PeerDescriptor,
        policy: &BleSecurityPolicy,
    ) -> BleRadioResult<BleLinkContext>;

    /// Disconnect from a peer.
    fn disconnect(&mut self, ctx: &BleLinkContext) -> BleRadioResult<()>;

    /// Discover services on a connected peer.
    fn discover_services(&mut self, ctx: &BleLinkContext) -> BleRadioResult<Vec<GattService>>;

    /// Read a characteristic value.
    fn read_characteristic(
        &mut self,
        ctx: &BleLinkContext,
        uuid: &BleUuid,
    ) -> BleRadioResult<Vec<u8>>;

    /// Write a characteristic value.
    fn write_characteristic(
        &mut self,
        ctx: &BleLinkContext,
        uuid: &BleUuid,
        value: &[u8],
    ) -> BleRadioResult<()>;

    /// Subscribe to characteristic notifications.
    fn subscribe_characteristic(
        &mut self,
        ctx: &BleLinkContext,
        uuid: &BleUuid,
    ) -> BleRadioResult<()>;

    /// Unsubscribe from characteristic notifications.
    fn unsubscribe_characteristic(
        &mut self,
        ctx: &BleLinkContext,
        uuid: &BleUuid,
    ) -> BleRadioResult<()>;
}

/// High-level host interface combining scanning and connection management.
#[derive(Debug)]
pub struct BleHost<R: BleRadio> {
    radio: R,
    active_connections: Vec<BleLinkContext>,
}

impl<R: BleRadio> BleHost<R> {
    /// Create a new BLE host with the given radio implementation.
    pub fn new(radio: R) -> Self {
        BleHost {
            radio,
            active_connections: Vec::new(),
        }
    }

    /// Get a reference to the underlying radio.
    pub fn radio(&self) -> &R {
        &self.radio
    }

    /// Get a mutable reference to the underlying radio.
    pub fn radio_mut(&mut self) -> &mut R {
        &mut self.radio
    }

    /// Scan for devices and return discovered peers.
    pub fn scan_for_devices(
        &mut self,
        filter: ScanFilter,
        duration_ms: u64,
    ) -> BleRadioResult<Vec<PeerDescriptor>> {
        self.radio.start_scan(filter)?;
        // In a real implementation, we'd collect discoveries during the scan period.
        // For now, this is a placeholder showing the pattern.
        std::thread::sleep(std::time::Duration::from_millis(duration_ms));
        self.radio.stop_scan()?;
        Ok(Vec::new()) // Placeholder
    }

    /// Connect to a device with security policy enforcement.
    pub fn connect_secure(
        &mut self,
        peer: &PeerDescriptor,
        policy: &BleSecurityPolicy,
    ) -> BleRadioResult<BleLinkContext> {
        let ctx = self.radio.connect(peer, policy)?;
        self.active_connections.push(ctx.clone());
        Ok(ctx)
    }

    /// Disconnect from a device.
    pub fn disconnect(&mut self, ctx: &BleLinkContext) -> BleRadioResult<()> {
        self.radio.disconnect(ctx)?;
        self.active_connections.retain(|c| c.peer_id != ctx.peer_id);
        Ok(())
    }

    /// Get all active connections.
    pub fn active_connections(&self) -> &[BleLinkContext] {
        &self.active_connections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock radio for testing.
    #[derive(Default)]
    struct MockRadio {
        scanned: bool,
    }

    impl BleRadio for MockRadio {
        fn start_scan(&mut self, _filter: ScanFilter) -> BleRadioResult<()> {
            self.scanned = true;
            Ok(())
        }

        fn stop_scan(&mut self) -> BleRadioResult<()> {
            self.scanned = false;
            Ok(())
        }

        fn connect(
            &mut self,
            _peer: &PeerDescriptor,
            _policy: &BleSecurityPolicy,
        ) -> BleRadioResult<BleLinkContext> {
            Ok(BleLinkContext {
                peer_id: "mock_peer".to_string(),
                security_level: ble_security::BleSecurityLevel::EncryptedWithMitm,
                bonded: true,
            })
        }

        fn disconnect(&mut self, _ctx: &BleLinkContext) -> BleRadioResult<()> {
            Ok(())
        }

        fn discover_services(&mut self, _ctx: &BleLinkContext) -> BleRadioResult<Vec<GattService>> {
            Ok(vec![])
        }

        fn read_characteristic(
            &mut self,
            _ctx: &BleLinkContext,
            _uuid: &BleUuid,
        ) -> BleRadioResult<Vec<u8>> {
            Ok(vec![])
        }

        fn write_characteristic(
            &mut self,
            _ctx: &BleLinkContext,
            _uuid: &BleUuid,
            _value: &[u8],
        ) -> BleRadioResult<()> {
            Ok(())
        }

        fn subscribe_characteristic(
            &mut self,
            _ctx: &BleLinkContext,
            _uuid: &BleUuid,
        ) -> BleRadioResult<()> {
            Ok(())
        }

        fn unsubscribe_characteristic(
            &mut self,
            _ctx: &BleLinkContext,
            _uuid: &BleUuid,
        ) -> BleRadioResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_ble_host_creation() {
        let radio = MockRadio::default();
        let host = BleHost::new(radio);
        assert!(host.active_connections().is_empty());
    }
}
