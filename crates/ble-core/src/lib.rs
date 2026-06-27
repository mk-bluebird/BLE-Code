#![forbid(unsafe_code)]
//! GATT models, UUIDs, and ALN bindings for BLE-Code.
//!
//! This crate provides typed representations of BLE services, characteristics,
//! and UUIDs. No stringly-typed UUIDs; use newtypes and enums.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

bitflags::bitflags! {
    /// GATT characteristic properties.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CharProperties: u8 {
        const READ = 0b0000_0001;
        const WRITE = 0b0000_0010;
        const WRITE_NO_RESPONSE = 0b0000_0100;
        const NOTIFY = 0b0000_1000;
        const INDICATE = 0b0001_0000;
        const SIGNED_WRITE = 0b0010_0000;
        const EXTENDED_PROPERTIES = 0b0100_0000;
    }
}

/// A 128-bit BLE UUID as a newtype wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BleUuid(pub [u8; 16]);

impl BleUuid {
    /// Create a BleUuid from a standard Uuid.
    pub fn from_uuid(uuid: Uuid) -> Self {
        BleUuid(uuid.as_bytes().to_owned())
    }

    /// Convert to a standard Uuid.
    pub fn to_uuid(&self) -> Uuid {
        Uuid::from_bytes(self.0)
    }

    /// Well-known Device Information Service UUID.
    pub const DEVICE_INFORMATION: Self = BleUuid([
        0x00, 0x00, 0x18, 0x0A, 0x00, 0x00, 0x10, 0x00,
        0x80, 0x00, 0x00, 0x80, 0x5F, 0x9B, 0x34, 0xFB,
    ]);

    /// Well-known Battery Service UUID.
    pub const BATTERY_SERVICE: Self = BleUuid([
        0x00, 0x00, 0x18, 0x0F, 0x00, 0x00, 0x10, 0x00,
        0x80, 0x00, 0x00, 0x80, 0x5F, 0x9B, 0x34, 0xFB,
    ]);
}

/// Kind of BLE service (standard or custom).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BleServiceKind {
    DeviceInformation,
    Battery,
    Custom { name: String },
}

/// Security requirements for a characteristic.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharSecurity {
    /// Requires encrypted link.
    pub requires_encryption: bool,
    /// Requires authenticated (bonded) peer.
    pub requires_authentication: bool,
    /// Requires MITM (Man-in-the-Middle) protection.
    pub requires_mitm_protection: bool,
}

impl CharSecurity {
    /// Default security for cybercore/neuromotor/bioscale operations.
    /// All three flags are true for maximum safety.
    pub const fn cybercore_default() -> Self {
        CharSecurity {
            requires_encryption: true,
            requires_authentication: true,
            requires_mitm_protection: true,
        }
    }

    /// No security (only for non-sensitive telemetry).
    pub const fn none() -> Self {
        CharSecurity {
            requires_encryption: false,
            requires_authentication: false,
            requires_mitm_protection: false,
        }
    }

    /// Check if this security level meets or exceeds a required minimum.
    pub fn meets(&self, required: &CharSecurity) -> bool {
        if required.requires_encryption && !self.requires_encryption {
            return false;
        }
        if required.requires_authentication && !self.requires_authentication {
            return false;
        }
        if required.requires_mitm_protection && !self.requires_mitm_protection {
            return false;
        }
        true
    }
}

/// A GATT characteristic with typed UUID, properties, and security.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GattCharacteristic {
    pub uuid: BleUuid,
    pub properties: CharProperties,
    pub security: CharSecurity,
    /// Human-readable name (optional, for AI agents).
    pub name: Option<String>,
}

/// A GATT service with typed UUID, kind, and characteristics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GattService {
    pub uuid: BleUuid,
    pub kind: BleServiceKind,
    pub characteristics: Vec<GattCharacteristic>,
    /// Human-readable name (optional, for AI agents).
    pub name: Option<String>,
}

impl GattService {
    /// Validate that all characteristics meet minimum security for cybercore ops.
    pub fn validate_cybercore_security(&self) -> Result<(), String> {
        let cybercore_default = CharSecurity::cybercore_default();
        for char in &self.characteristics {
            // For write/notify characteristics, enforce cybercore security
            if char.properties.contains(CharProperties::WRITE)
                || char.properties.contains(CharProperties::NOTIFY)
                || char.properties.contains(CharProperties::INDICATE)
            {
                if !char.security.meets(&cybercore_default) {
                    return Err(format!(
                        "Characteristic {:?} requires cybercore-level security",
                        char.name.as_deref().unwrap_or("unnamed")
                    ));
                }
            }
        }
        Ok(())
    }
}

/// A complete BLE profile composed of multiple services.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BleProfile {
    pub profile_name: String,
    /// Host DID binding for sovereignty.
    pub host_did: String,
    /// Bostrom address binding for sovereignty.
    pub bostrom_addr: String,
    pub services: Vec<GattService>,
}

impl BleProfile {
    /// Validate all services in the profile for cybercore security.
    pub fn validate_all_services(&self) -> Result<(), String> {
        for service in &self.services {
            service.validate_cybercore_security()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ble_uuid_conversion() {
        let uuid = Uuid::parse_str("0000180A-0000-1000-8000-00805F9B34FB").unwrap();
        let ble_uuid = BleUuid::from_uuid(uuid);
        assert_eq!(ble_uuid.to_uuid(), uuid);
    }

    #[test]
    fn test_char_security_meets() {
        let high = CharSecurity::cybercore_default();
        let low = CharSecurity::none();

        assert!(high.meets(&low));
        assert!(!low.meets(&high));
        assert!(high.meets(&high));
    }

    #[test]
    fn test_gatt_service_validation() {
        let mut char = GattCharacteristic {
            uuid: BleUuid([0; 16]),
            properties: CharProperties::WRITE | CharProperties::NOTIFY,
            security: CharSecurity::cybercore_default(),
            name: Some("test_char".to_string()),
        };
        let service = GattService {
            uuid: BleUuid::DEVICE_INFORMATION,
            kind: BleServiceKind::DeviceInformation,
            characteristics: vec![char.clone()],
            name: Some("test_service".to_string()),
        };
        assert!(service.validate_cybercore_security().is_ok());

        // Now test with insufficient security
        char.security = CharSecurity::none();
        let insecure_service = GattService {
            uuid: BleUuid::DEVICE_INFORMATION,
            kind: BleServiceKind::DeviceInformation,
            characteristics: vec![char],
            name: Some("insecure_service".to_string()),
        };
        assert!(insecure_service.validate_cybercore_security().is_err());
    }
}
