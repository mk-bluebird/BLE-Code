#![forbid(unsafe_code)]
//! BLE governance layer for neurorights-aware BLE profiles.
//!
//! This crate defines subject-level profiles, device class policies,
//! service policies, and Risk-of-Harm (`RoH`) invariants.
#![allow(missing_docs)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Neurorights classification for BLE service data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NeurorightsTag {
    BasicTelemetry,
    BciIntent,
    BiometricId,
    Unknown,
}

/// Role of a BLE service in the neurorights model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRole {
    Sensor,
    Control,
    Admin,
}

/// Subject-level profile for BLE permissions and `RoH` envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectProfile {
    pub subject_id: String,
    
    /// Upper bound on `RoH` across all BCI services, 0.0–1.0 but enforced as <= 0.3 in invariants.
    pub roh_ceiling: f32,
}

/// Device class policy (e.g., "openbci-cyton-nus").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceClassPolicy {
    pub class_id: String,
    pub require_encryption: bool,
    pub require_mic: bool,
    pub require_bonding: bool,
    pub allowed_phys: Vec<String>,
    pub max_conn_interval_ms: u32,
    pub max_pdu_bytes: u16,
}

/// BLE service policy within a device class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePolicy {
    pub service_uuid: String,
    
    /// Reference to `DeviceClassPolicy.class_id`.
    pub device_class_id: String,
    
    pub role: ServiceRole,
    pub neurorights_tag: NeurorightsTag,
    
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_notify: bool,
    
    /// `RoH` weight contribution for this service when active.
    pub roh_weight: f32,
    
    /// Whether this service requires higher-layer framing validation
    /// at a higher layer (e.g., `OpenBCI` framed protocol).
    pub requires_framing: bool,
}

/// Top-level BLE profile shard (subject + policies).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleProfileShard {
    pub subject: SubjectProfile,
    pub device_class_policies: Vec<DeviceClassPolicy>,
    pub service_policies: Vec<ServicePolicy>,
}

/// Optional environment snapshot metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    pub location: Option<String>,
    pub max_rssi_dbm: i16,
}

/// Governance invariant errors.
#[derive(Debug, Error)]
pub enum GovernanceInvariantError {
    #[error("subject_id cannot be empty")]
    EmptySubjectId,
    
    #[error("roh_ceiling {0} is invalid (must be 0.0..=0.3)")]
    InvalidRohCeiling(f32),
    
    #[error("service {service_uuid} has negative roh_weight {roh_weight}")]
    NegativeRohWeight {
        service_uuid: String,
        roh_weight: f32,
    },
    
    #[error("total RoH {sum} exceeds ceiling {ceiling}")]
    RohCeilingExceeded { sum: f32, ceiling: f32 },
}

impl BleProfileShard {
    /// Validate `RoH` and basic structural invariants.
    pub fn validate_invariants(&self) -> Result<(), GovernanceInvariantError> {
        // 1. Subject ID must not be empty
        if self.subject.subject_id.trim().is_empty() {
            return Err(GovernanceInvariantError::EmptySubjectId);
        }

        // 2. RoH ceiling must be in [0.0, 0.3]
        let ceiling = self.subject.roh_ceiling;
        if !(0.0..=0.3).contains(&ceiling) {
            return Err(GovernanceInvariantError::InvalidRohCeiling(ceiling));
        }

        // 3. No service may have negative RoH weight
        for svc in &self.service_policies {
            if svc.roh_weight < 0.0 {
                return Err(GovernanceInvariantError::NegativeRohWeight {
                    service_uuid: svc.service_uuid.clone(),
                    roh_weight: svc.roh_weight,
                });
            }
        }

        // 4. Sum of all service RoH weights must not exceed subject ceiling
        let total: f32 = self.service_policies.iter().map(|s| s.roh_weight).sum();
        if total > ceiling {
            return Err(GovernanceInvariantError::RohCeilingExceeded {
                sum: total,
                ceiling,
            });
        }

        Ok(())
    }

    /// Find device class policy by ID.
    pub fn find_device_class(&self, class_id: &str) -> Option<&DeviceClassPolicy> {
        self.device_class_policies
            .iter()
            .find(|dc| dc.class_id == class_id)
    }

    /// Find service policy by UUID.
    pub fn find_service(&self, service_uuid: &str) -> Option<&ServicePolicy> {
        self.service_policies
            .iter()
            .find(|sp| sp.service_uuid == service_uuid)
    }
}
