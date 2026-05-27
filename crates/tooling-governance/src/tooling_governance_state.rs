#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyBand {
    SniffOnly,
    GovernedDig,
    DeepAutonomy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolingAgentOverride {
    pub agent_id: String,
    pub autonomy_band: AutonomyBand,
    pub allowed_paths: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolingGovernanceState {
    pub id: String,
    pub projectid: String,
    pub version: String,
    pub generated_at_utc: String,
    pub generator: String,

    pub unsafe_forbidden_core: bool,
    pub unwrap_forbidden_core: bool,
    pub self_modification_forbidden: bool,
    pub soulmodelingforbidden: bool,

    pub workspace_root: String,
    pub workspace_edit_allowed: Vec<String>,

    pub default_autonomy_band: AutonomyBand,
    pub agent_overrides: Vec<ToolingAgentOverride>,

    pub codeevent_schema_id: String,
    pub abort_and_flush_hex: String,
    pub kerew_profile_id: String,

    pub status: String,
    pub attested_by: Vec<String>,
    pub signing_manifest_ref: String,
    pub chain_of_custody_ref: String,
}

impl ToolingGovernanceState {
    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Empty ToolingGovernanceState.id".into());
        }
        if self.workspace_root.trim().is_empty() {
            return Err("workspace_root must be non-empty".into());
        }
        if self.workspace_edit_allowed.is_empty() {
            return Err("workspace_edit_allowed must not be empty".into());
        }
        if self.self_modification_forbidden {
            // no override may include the config path
            let cfg_path = "schemas/tooling-governance-state.v1.aln";
            if self
                .agent_overrides
                .iter()
                .any(|o| o.allowed_paths.iter().any(|p| p.contains(cfg_path)))
            {
                return Err("self_modification_forbidden but some overrides can edit governance state".into());
            }
        }
        Ok(())
    }

    pub fn autonomy_for_agent(&self, agent_id: &str) -> AutonomyBand {
        self.agent_overrides
            .iter()
            .find(|o| o.agent_id == agent_id)
            .map(|o| o.autonomy_band)
            .unwrap_or(self.default_autonomy_band)
    }

    pub fn is_path_edit_allowed(&self, path: &str, agent_id: &str) -> bool {
        let band = self.autonomy_for_agent(agent_id);
        if matches!(band, AutonomyBand::SniffOnly) {
            return false;
        }

        // First, global workspace allowed prefixes
        let in_workspace = self
            .workspace_edit_allowed
            .iter()
            .any(|prefix| path.starts_with(prefix));
        if !in_workspace {
            return false;
        }

        // Then, per-agent overrides may further restrict
        if let Some(ov) = self.agent_overrides.iter().find(|o| o.agent_id == agent_id) {
            return ov.allowed_paths.iter().any(|prefix| path.starts_with(prefix));
        }

        true
    }
}
