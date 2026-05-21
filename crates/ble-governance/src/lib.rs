#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod aln_loader;

/// Neurorights semantics of a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NeurorightsTag {
    BasicTelemetry,
    BciIntent,
    BiometricId,
    Unknown,
}

/// Role of a service in the BLE profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRole {
    Sensor,
    Control,
    Admin,
}

/// Subject-level profile for BLE permissions and RoH envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubjectProfile {
    /// Stable subject identifier (DID, user ID, etc.).
    pub subject_id: String,
    /// Upper bound on RoH across all BCI services, 0.0–1.0 but enforced as <= 0.3 in invariants.
    pub roh_ceiling: f32,
    /// Whether any BLE activity is allowed at all for this subject.
    pub allow_ble: bool,
    /// Maximum number of simultaneous device connections.
    pub max_parallel_devices: u32,
}

/// Policy for a particular device class (e.g., "openbci-cyton-nus").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceClassPolicy {
    pub class_id: String,
    /// Min PHY allowed for this class (e.g., LE1M).
    pub min_phy: String,
    /// Max PHY allowed for this class.
    pub max_phy: String,
    /// Whether the link MUST be encrypted.
    pub require_encryption: bool,
    /// Whether MIC protection is required.
    pub require_mic: bool,
    /// Whether bonding is required.
    pub require_bonding: bool,
    /// Max allowed connection interval in ms.
    pub max_conn_interval_ms: u32,
    /// Max allowed PDU size in bytes.
    pub max_pdu_bytes: u16,
    /// Whether CTE usage is allowed for this class.
    pub allow_cte: bool,
    /// Whether BCI payloads are allowed through this class at all.
    pub allow_bci_payload: bool,
}

/// Policy for a specific GATT service within a device class.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServicePolicy {
    /// Reference to DeviceClassPolicy.class_id.
    pub class_id: String,
    /// BLE service UUID (string).
    pub service_uuid: String,
    /// Role (sensor/control/admin).
    pub role: ServiceRole,
    /// Access flags.
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_notify: bool,
    /// RoH weight contribution for this service when active.
    /// This is per-service; subject-level ceiling is enforced at shard level.
    pub roh_weight: f32,
    /// Neurorights classification.
    pub neurorights_tag: NeurorightsTag,
    /// True when BLE is treated purely as a transport and content validation occurs
    /// at a higher layer (e.g., OpenBCI framed protocol).
    pub passthrough: bool,
}

/// Aggregate BLE profile shard for a subject or configuration namespace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BleProfileShard {
    pub subject: SubjectProfile,
    pub device_class_policies: Vec<DeviceClassPolicy>,
    pub service_policies: Vec<ServicePolicy>,
}

/// Environment-level security policy shard (RF crowding, enforcement actions).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BleSecurityPolicyShard {
    /// Max number of devices and RSSI thresholds for RF crowding metrics.
    pub max_device_count: u32,
    pub max_rssi_dbm: i16,
    /// If true, forbid new BCI sessions when crowding metrics are exceeded.
    pub forbid_new_bci_when_crowded: bool,
}

/// Errors from invariant validation.
#[derive(Debug, Error)]
pub enum GovernanceInvariantError {
    #[error("subject_id must not be empty")]
    EmptySubjectId,
    #[error("roh_ceiling must be between 0.0 and 0.3 inclusive, got {0}")]
    InvalidRohCeiling(f32),
    #[error("service {service_uuid} has negative roh_weight {roh_weight}")]
    NegativeRohWeight {
        service_uuid: String,
        roh_weight: f32,
    },
    #[error("sum of roh_weight for BCI services ({sum}) exceeds roh_ceiling ({ceiling})")]
    RohCeilingExceeded { sum: f32, ceiling: f32 },
}

impl BleProfileShard {
    /// Validate RoH and basic structural invariants.
    ///
    /// Guarantees:
    /// - subject.subject_id is non-empty.
    /// - 0.0 <= roh_ceiling <= 0.3
    /// - no service has negative roh_weight
    /// - sum of roh_weight for NeurorightsTag::BciIntent services <= roh_ceiling
    pub fn validate_invariants(&self) -> Result<(), GovernanceInvariantError> {
        let subj = &self.subject;

        if subj.subject_id.trim().is_empty() {
            return Err(GovernanceInvariantError::EmptySubjectId);
        }

        if !(0.0..=0.3).contains(&subj.roh_ceiling) {
            return Err(GovernanceInvariantError::InvalidRohCeiling(
                subj.roh_ceiling,
            ));
        }

        let mut sum_bci = 0.0_f32;

        for svc in &self.service_policies {
            if svc.roh_weight < 0.0 {
                return Err(GovernanceInvariantError::NegativeRohWeight {
                    service_uuid: svc.service_uuid.clone(),
                    roh_weight: svc.roh_weight,
                });
            }

            if matches!(svc.neurorights_tag, NeurorightsTag::BciIntent) {
                sum_bci += svc.roh_weight;
            }
        }

        if sum_bci > subj.roh_ceiling + f32::EPSILON {
            return Err(GovernanceInvariantError::RohCeilingExceeded {
                sum: sum_bci,
                ceiling: subj.roh_ceiling,
            });
        }

        Ok(())
    }
}

impl BleProfileShard {
    pub fn validate_invariants(&self) -> Result<(), GovernanceInvariantError> {
        let subj = &self.subject;

        // 1. subject_id must be non-empty
        if subj.subject_id.trim().is_empty() {
            return Err(GovernanceInvariantError::EmptySubjectId);
        }

        // 2. roh_ceiling must be between 0.0 and 0.3 (inclusive)
        if !(0.0..=0.3).contains(&subj.roh_ceiling) {
            return Err(GovernanceInvariantError::InvalidRohCeiling(
                subj.roh_ceiling,
            ));
        }

        // 3. No negative roh_weight and 4. Sum of BCI weights <= roh_ceiling
        let mut sum_bci = 0.0_f32;

        for svc in &self.service_policies {
            if svc.roh_weight < 0.0 {
                return Err(GovernanceInvariantError::NegativeRohWeight {
                    service_uuid: svc.service_uuid.clone(),
                    roh_weight: svc.roh_weight,
                });
            }

            if matches!(svc.neurorights_tag, NeurorightsTag::BciIntent) {
                sum_bci += svc.roh_weight;
            }
        }

        if sum_bci > subj.roh_ceiling + f32::EPSILON {
            return Err(GovernanceInvariantError::RohCeilingExceeded {
                sum: sum_bci,
                ceiling: subj.roh_ceiling,
            });
        }

        Ok(())
    }
}
