#![forbid(unsafe_code)]
//! Peripheral role implementation for BLE-Code.
//!
//! This crate provides the `BlePeripheral` trait that platform adapters implement
//! for peripheral (advertiser/server) role operations.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use ble_core::{BleUuid, GattCharacteristic, GattService};
pub use ble_security::{BleLinkContext, BleSecurityPolicy};

/// Error types for BLE peripheral operations.
#[derive(Debug, Error)]
pub enum BlePeripheralError {
    #[error("Advertising failed: {0}")]
    AdvertisingFailed(String),

    #[error("Service registration failed: {0}")]
    ServiceRegistrationFailed(String),

    #[error("Characteristic operation failed: {0}")]
    CharacteristicOperationFailed(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Not advertising")]
    NotAdvertising,
}

/// Result type for BLE peripheral operations.
pub type BlePeripheralResult<T> = Result<T, BlePeripheralError>;

/// Configuration for a peripheral device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeripheralConfig {
    /// Device name to advertise.
    pub device_name: String,
    /// Service UUIDs to include in advertising data.
    pub service_uuids: Vec<BleUuid>,
    /// Whether to use connectable advertising.
    pub connectable: bool,
    /// Minimum advertising interval in milliseconds.
    pub adv_interval_min_ms: u16,
    /// Maximum advertising interval in milliseconds.
    pub adv_interval_max_ms: u16,
}

impl Default for PeripheralConfig {
    fn default() -> Self {
        PeripheralConfig {
            device_name: "BLE-Code Peripheral".to_string(),
            service_uuids: Vec::new(),
            connectable: true,
            adv_interval_min_ms: 100,
            adv_interval_max_ms: 200,
        }
    }
}

/// Trait abstracting BLE peripheral operations.
///
/// Platform adapters implement this trait to provide peripheral functionality.
pub trait BlePeripheral {
    /// Register a GATT service with the peripheral stack.
    fn register_service(&mut self, service: &GattService) -> BlePeripheralResult<()>;

    /// Start advertising with the given configuration.
    fn start_advertising(&mut self, config: &PeripheralConfig) -> BlePeripheralResult<()>;

    /// Stop advertising.
    fn stop_advertising(&mut self) -> BlePeripheralResult<()>;

    /// Wait for and accept an incoming connection with security policy.
    fn accept_connection(&mut self, policy: &BleSecurityPolicy) -> BlePeripheralResult<BleLinkContext>;

    /// Handle characteristic read request.
    fn on_characteristic_read(
        &mut self,
        ctx: &BleLinkContext,
        uuid: &BleUuid,
    ) -> BlePeripheralResult<Vec<u8>>;

    /// Handle characteristic write request.
    fn on_characteristic_write(
        &mut self,
        ctx: &BleLinkContext,
        uuid: &BleUuid,
        value: &[u8],
    ) -> BlePeripheralResult<()>;

    /// Send a notification to a connected peer.
    fn send_notification(
        &mut self,
        ctx: &BleLinkContext,
        uuid: &BleUuid,
        value: &[u8],
    ) -> BlePeripheralResult<()>;
}

/// High-level peripheral server combining services and advertising.
#[derive(Debug)]
pub struct BlePeripheralServer<P: BlePeripheral> {
    peripheral: P,
    registered_services: Vec<GattService>,
    is_advertising: bool,
}

impl<P: BlePeripheral> BlePeripheralServer<P> {
    /// Create a new peripheral server.
    pub fn new(peripheral: P) -> Self {
        BlePeripheralServer {
            peripheral,
            registered_services: Vec::new(),
            is_advertising: false,
        }
    }

    /// Add a service to be advertised.
    pub fn add_service(&mut self, service: &GattService) -> BlePeripheralResult<()> {
        self.peripheral.register_service(service)?;
        self.registered_services.push(service.clone());
        Ok(())
    }

    /// Start advertising all registered services.
    pub fn start_advertising(&mut self, config: &PeripheralConfig) -> BlePeripheralResult<()> {
        self.peripheral.start_advertising(config)?;
        self.is_advertising = true;
        Ok(())
    }

    /// Stop advertising.
    pub fn stop_advertising(&mut self) -> BlePeripheralResult<()> {
        self.peripheral.stop_advertising()?;
        self.is_advertising = false;
        Ok(())
    }

    /// Check if currently advertising.
    pub fn is_advertising(&self) -> bool {
        self.is_advertising
    }

    /// Get registered services.
    pub fn registered_services(&self) -> &[GattService] {
        &self.registered_services
    }

    /// Accept an incoming connection with security policy enforcement.
    pub fn accept_secure_connection(
        &mut self,
        policy: &BleSecurityPolicy,
    ) -> BlePeripheralResult<BleLinkContext> {
        self.peripheral.accept_connection(policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock peripheral for testing.
    #[derive(Default)]
    struct MockPeripheral {
        advertising: bool,
    }

    impl BlePeripheral for MockPeripheral {
        fn register_service(&mut self, _service: &GattService) -> BlePeripheralResult<()> {
            Ok(())
        }

        fn start_advertising(&mut self, _config: &PeripheralConfig) -> BlePeripheralResult<()> {
            self.advertising = true;
            Ok(())
        }

        fn stop_advertising(&mut self) -> BlePeripheralResult<()> {
            self.advertising = false;
            Ok(())
        }

        fn accept_connection(
            &mut self,
            _policy: &BleSecurityPolicy,
        ) -> BlePeripheralResult<BleLinkContext> {
            Ok(BleLinkContext {
                peer_id: "mock_central".to_string(),
                security_level: ble_security::BleSecurityLevel::EncryptedWithMitm,
                bonded: true,
            })
        }

        fn on_characteristic_read(
            &mut self,
            _ctx: &BleLinkContext,
            _uuid: &BleUuid,
        ) -> BlePeripheralResult<Vec<u8>> {
            Ok(vec![])
        }

        fn on_characteristic_write(
            &mut self,
            _ctx: &BleLinkContext,
            _uuid: &BleUuid,
            _value: &[u8],
        ) -> BlePeripheralResult<()> {
            Ok(())
        }

        fn send_notification(
            &mut self,
            _ctx: &BleLinkContext,
            _uuid: &BleUuid,
            _value: &[u8],
        ) -> BlePeripheralResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_peripheral_server_creation() {
        let peripheral = MockPeripheral::default();
        let server = BlePeripheralServer::new(peripheral);
        assert!(server.registered_services().is_empty());
        assert!(!server.is_advertising());
    }

    #[test]
    fn test_peripheral_advertising_lifecycle() {
        let peripheral = MockPeripheral::default();
        let mut server = BlePeripheralServer::new(peripheral);

        let config = PeripheralConfig::default();
        assert!(server.start_advertising(&config).is_ok());
        assert!(server.is_advertising());

        assert!(server.stop_advertising().is_ok());
        assert!(!server.is_advertising());
    }
}
