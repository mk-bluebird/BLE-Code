#![forbid(unsafe_code)]

//! Non-actuating BLE guard.
//!
//! This crate never calls OS BLE APIs. It only evaluates intents and link
//! parameters against BleProfileShard policy and internal RoH state.

mod link_policy;

use ble_governance::{BleProfileShard, ServicePolicy};
use ble_model::{BleIntent, BleLinkParams};
use link_policy::check_link_against_class;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Decision returned by the guard for any BLE intent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum BleGuardDecision {
    Allowed,
    Rejected { reason: String },
}

/// Runtime state tracked by the guard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleSessionState {
    /// Active connections keyed by (class_id, device_id).
    pub active_connections: HashMap<(String, String), ActiveConnection>,
    /// Accumulated RoH score for BLE BCI services under this guard.
    pub accumulated_roh_ble: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveConnection {
    pub class_id: String,
    pub device_id: String,
    /// Currently subscribed service UUIDs.
    pub subscribed_services: Vec<String>,
}

/// Main guard object holding the profile and mutable session state.
#[derive(Debug)]
pub struct BleGuard {
    profile: BleProfileShard,
    state: BleSessionState,
}

impl BleGuard {
    /// Create a new guard from a validated profile.
    pub fn new(profile: BleProfileShard) -> Self {
        BleGuard {
            profile,
            state: BleSessionState {
                active_connections: HashMap::new(),
                accumulated_roh_ble: 0.0,
            },
        }
    }

    pub fn state(&self) -> &BleSessionState {
        &self.state
    }

    /// Evaluate a scan request.
    /// For now this is a structural check + allow/deny on subject.allow_ble.
    pub fn guard_scan(&self) -> BleGuardDecision {
        if !self.profile.subject.allow_ble {
            return BleGuardDecision::Rejected {
                reason: "BLE disabled for subject".into(),
            };
        }
        BleGuardDecision::Allowed
    }

    /// Evaluate a connect request.
    pub fn guard_connect(&mut self, intent: &BleIntent, link: &BleLinkParams) -> BleGuardDecision {
        // Structural: only Connect intents are accepted here.
        let (class_id, device_id) = match intent {
            BleIntent::Connect {
                class_id,
                device_id,
            } => (class_id, device_id),
            _ => {
                return BleGuardDecision::Rejected {
                    reason: "guard_connect called with non-Connect intent".into(),
                };
            }
        };

        if !self.profile.subject.allow_ble {
            return BleGuardDecision::Rejected {
                reason: "BLE disabled for subject".into(),
            };
        }

        // Enforce max_parallel_devices.
        let current_devices = self.state.active_connections.len() as u32;
        if current_devices >= self.profile.subject.max_parallel_devices {
            return BleGuardDecision::Rejected {
                reason: "max_parallel_devices exceeded".into(),
            };
        }

        // Look up DeviceClassPolicy and evaluate link parameters.
        let class_policy = match self
            .profile
            .device_class_policies
            .iter()
            .find(|p| p.class_id == *class_id)
        {
            Some(p) => p,
            None => {
                return BleGuardDecision::Rejected {
                    reason: format!("No DeviceClassPolicy for class_id={class_id}"),
                };
            }
        };

        if let Err(reason) = check_link_against_class(class_policy, link) {
            return BleGuardDecision::Rejected { reason };
        }

        // If we reach here, the connection is allowed.
        self.state.active_connections.insert(
            (class_id.clone(), device_id.clone()),
            ActiveConnection {
                class_id: class_id.clone(),
                device_id: device_id.clone(),
                subscribed_services: Vec::new(),
            },
        );

        BleGuardDecision::Allowed
    }

    /// Evaluate a subscribe request.
    pub fn guard_subscribe(&mut self, intent: &BleIntent) -> BleGuardDecision {
        let (class_id, device_id, service_uuid) = match intent {
            BleIntent::SubscribeCharacteristic {
                class_id,
                device_id,
                service_uuid,
                ..
            } => (class_id, device_id, service_uuid),
            _ => {
                return BleGuardDecision::Rejected {
                    reason: "guard_subscribe called with non-SubscribeCharacteristic intent".into(),
                };
            }
        };

        // Ensure connection exists.
        let key = (class_id.clone(), device_id.clone());
        let conn = match self.state.active_connections.get_mut(&key) {
            Some(c) => c,
            None => {
                return BleGuardDecision::Rejected {
                    reason: "No active connection for subscribe".into(),
                };
            }
        };

        // Look up service policy.
        let svc_policy = match self.find_service_policy(class_id, service_uuid) {
            Some(p) => p,
            None => {
                return BleGuardDecision::Rejected {
                    reason: format!(
                        "No ServicePolicy for class_id={class_id}, service_uuid={service_uuid}"
                    ),
                };
            }
        };

        if !svc_policy.allow_notify {
            return BleGuardDecision::Rejected {
                reason: "Notifications not allowed for this service".into(),
            };
        }

        // Neurorights / RoH: adding this subscription must not overflow roh_ceiling.
        if let Some(reason) = self.check_roh_budget(svc_policy) {
            return BleGuardDecision::Rejected { reason };
        }

        if !conn.subscribed_services.iter().any(|u| u == service_uuid) {
            conn.subscribed_services.push(service_uuid.clone());
        }

        BleGuardDecision::Allowed
    }

    /// Evaluate a characteristic write.
    pub fn guard_write(&mut self, intent: &BleIntent, link: &BleLinkParams) -> BleGuardDecision {
        let (class_id, device_id, service_uuid) = match intent {
            BleIntent::WriteCharacteristic {
                class_id,
                device_id,
                service_uuid,
                ..
            } => (class_id, device_id, service_uuid),
            _ => {
                return BleGuardDecision::Rejected {
                    reason: "guard_write called with non-WriteCharacteristic intent".into(),
                };
            }
        };

        // Ensure connection exists.
        let key = (class_id.clone(), device_id.clone());
        if !self.state.active_connections.contains_key(&key) {
            return BleGuardDecision::Rejected {
                reason: "No active connection for write".into(),
            };
        }

        // Link parameters must still respect class policy.
        let class_policy = match self
            .profile
            .device_class_policies
            .iter()
            .find(|p| p.class_id == *class_id)
        {
            Some(p) => p,
            None => {
                return BleGuardDecision::Rejected {
                    reason: format!("No DeviceClassPolicy for class_id={class_id}"),
                };
            }
        };
        if let Err(reason) = check_link_against_class(class_policy, link) {
            return BleGuardDecision::Rejected { reason };
        }

        // Service policy must permit writes.
        let svc_policy = match self.find_service_policy(class_id, service_uuid) {
            Some(p) => p,
            None => {
                return BleGuardDecision::Rejected {
                    reason: format!(
                        "No ServicePolicy for class_id={class_id}, service_uuid={service_uuid}"
                    ),
                };
            }
        };

        if !svc_policy.allow_write {
            return BleGuardDecision::Rejected {
                reason: "Writes not allowed for this service".into(),
            };
        }

        // BCI write may still need RoH budget checks depending on your policy.
        if let Some(reason) = self.check_roh_budget(svc_policy) {
            return BleGuardDecision::Rejected { reason };
        }

        BleGuardDecision::Allowed
    }

    fn find_service_policy(&self, class_id: &str, service_uuid: &str) -> Option<&ServicePolicy> {
        self.profile
            .service_policies
            .iter()
            .find(|p| p.class_id == class_id && p.service_uuid.eq_ignore_ascii_case(service_uuid))
    }

    /// Enforce `roh_ceiling` using accumulated_roh_ble + this service's weight.
    fn check_roh_budget(&mut self, svc: &ServicePolicy) -> Option<String> {
        use ble_governance::NeurorightsTag;

        if !matches!(svc.neurorights_tag, NeurorightsTag::BciIntent) {
            return None; // Only BCI services count toward the BCI RoH pool.
        }

        let ceiling = self.profile.subject.roh_ceiling;
        let projected = self.state.accumulated_roh_ble + svc.roh_weight;

        if projected > ceiling + f32::EPSILON {
            Some(format!(
                "RoH ceiling exceeded: current={:.3}, adding={:.3}, ceiling={:.3}",
                self.state.accumulated_roh_ble, svc.roh_weight, ceiling
            ))
        } else {
            // Safe to accumulate.
            self.state.accumulated_roh_ble = projected;
            None
        }
    }
}
