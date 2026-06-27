#![forbid(unsafe_code)]
//! BLE pairing, bonding, and link-level policy abstraction.
//!
//! This crate wraps raw pairing modes in higher-level "policies" that encode
//! safe defaults. AI agents learn that any BLE code interacting with cybernetic
//! or bioscale devices must use a `BleSecurityPolicy` set to at least
//! `EncryptedWithMitm`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Security level for a BLE link.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum BleSecurityLevel {
    /// No encryption or authentication.
    NoSecurity,
    /// Encrypted link without MITM protection.
    EncryptedNoMitm,
    /// Encrypted link with MITM protection (recommended for cybercore).
    EncryptedWithMitm,
}

/// Security policy for BLE connections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BleSecurityPolicy {
    /// Minimum required security level.
    pub min_level: BleSecurityLevel,
    /// Whether to bond on successful connection.
    pub bond_on_success: bool,
    /// Whether to allow legacy (pre-Bluetooth 4.2) pairing.
    pub allow_legacy_pairing: bool,
}

impl BleSecurityPolicy {
    /// Default policy for cybercore/neuromotor/bioscale operations.
    /// Requires encrypted, MITM-protected links and bonds on success.
    pub const fn cybercore_default() -> Self {
        BleSecurityPolicy {
            min_level: BleSecurityLevel::EncryptedWithMitm,
            bond_on_success: true,
            allow_legacy_pairing: false,
        }
    }

    /// Relaxed policy for non-sensitive telemetry.
    pub const fn telemetry_default() -> Self {
        BleSecurityPolicy {
            min_level: BleSecurityLevel::EncryptedNoMitm,
            bond_on_success: false,
            allow_legacy_pairing: false,
        }
    }

    /// Check if a link context meets this policy.
    pub fn is_link_acceptable(&self, ctx: &BleLinkContext) -> bool {
        ctx.security_level >= self.min_level
    }
}

/// Context describing an established BLE link.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BleLinkContext {
    /// Peer identifier (MAC address or other stable ID).
    pub peer_id: String,
    /// Current security level of the link.
    pub security_level: BleSecurityLevel,
    /// Whether the devices are bonded.
    pub bonded: bool,
}

/// Error types for security operations.
#[derive(Debug, Error)]
pub enum BleSecurityError {
    #[error("Link security level {current:?} does not meet policy minimum {required:?}")]
    InsufficientSecurity {
        current: BleSecurityLevel,
        required: BleSecurityLevel,
    },

    #[error("Legacy pairing is not allowed by policy")]
    LegacyPairingNotAllowed,

    #[error("Bonding failed: {0}")]
    BondingFailed(String),

    #[error("Connection rejected: {0}")]
    ConnectionRejected(String),
}

/// Result type for security operations.
pub type BleSecurityResult<T> = Result<T, BleSecurityError>;

/// Enforce a security policy on a link context.
pub fn enforce_policy_link(
    policy: &BleSecurityPolicy,
    ctx: &BleLinkContext,
) -> BleSecurityResult<()> {
    if !policy.is_link_acceptable(ctx) {
        return Err(BleSecurityError::InsufficientSecurity {
            current: ctx.security_level,
            required: policy.min_level,
        });
    }

    if policy.bond_on_success && !ctx.bonded {
        // In a real implementation, this would trigger bonding.
        // Here we just note that bonding is expected.
        return Err(BleSecurityError::BondingFailed(
            "Expected bonded link but peer is not bonded".to_string(),
        ));
    }

    Ok(())
}

/// Request a secure connection with the given policy.
///
/// In a real implementation, this would interact with the BLE stack
/// to establish a connection with the required security parameters.
pub fn request_secure_connection(
    _policy: &BleSecurityPolicy,
    _peer_id: &str,
) -> BleSecurityResult<BleLinkContext> {
    // Placeholder: in real code this would initiate the connection
    // For now, return an error indicating this is a stub
    Err(BleSecurityError::ConnectionRejected(
        "Platform-specific implementation required".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_level_ordering() {
        assert!(BleSecurityLevel::EncryptedWithMitm > BleSecurityLevel::EncryptedNoMitm);
        assert!(BleSecurityLevel::EncryptedNoMitm > BleSecurityLevel::NoSecurity);
    }

    #[test]
    fn test_cybercore_policy() {
        let policy = BleSecurityPolicy::cybercore_default();
        assert_eq!(policy.min_level, BleSecurityLevel::EncryptedWithMitm);
        assert!(policy.bond_on_success);
        assert!(!policy.allow_legacy_pairing);
    }

    #[test]
    fn test_enforce_policy_acceptable() {
        let policy = BleSecurityPolicy::cybercore_default();
        let ctx = BleLinkContext {
            peer_id: "test_peer".to_string(),
            security_level: BleSecurityLevel::EncryptedWithMitm,
            bonded: true,
        };
        assert!(enforce_policy_link(&policy, &ctx).is_ok());
    }

    #[test]
    fn test_enforce_policy_insufficient() {
        let policy = BleSecurityPolicy::cybercore_default();
        let ctx = BleLinkContext {
            peer_id: "test_peer".to_string(),
            security_level: BleSecurityLevel::EncryptedNoMitm,
            bonded: false,
        };
        let result = enforce_policy_link(&policy, &ctx);
        assert!(matches!(
            result,
            Err(BleSecurityError::InsufficientSecurity { .. })
        ));
    }

    #[test]
    fn test_enforce_policy_bonding_required() {
        let policy = BleSecurityPolicy::cybercore_default();
        let ctx = BleLinkContext {
            peer_id: "test_peer".to_string(),
            security_level: BleSecurityLevel::EncryptedWithMitm,
            bonded: false,
        };
        let result = enforce_policy_link(&policy, &ctx);
        assert!(matches!(result, Err(BleSecurityError::BondingFailed(_))));
    }
}
