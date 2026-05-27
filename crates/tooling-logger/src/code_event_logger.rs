#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::Path;

use tooling_governance::ToolingGovernanceState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Suggestion,
    PatchApplied,
    PatchRejected,
    GovernanceUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeKind {
    Add,
    Modify,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceDecision {
    Approved,
    Rejected,
    RequiresHumanReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEvent {
    pub event_id: String,
    pub timestamp_utc: String,
    pub agent_id: String,
    pub event_type: EventType,
    pub repository: String,
    pub commit_hash_before: Option<String>,
    pub commit_hash_after: Option<String>,
    pub path: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub change_kind: ChangeKind,
    pub before_content: Option<String>,
    pub after_content: Option<String>,
    pub violations: Vec<String>,
    pub governance_decision: GovernanceDecision,
    pub reason_for_rejection: Option<String>,
    pub hex_stamp: String,
    pub kps_grading: [f32; 3],
}

pub struct CodeEventLogger {
    gov: ToolingGovernanceState,
    sniff_path: String,
}

impl CodeEventLogger {
    pub fn new(gov: ToolingGovernanceState, sniff_path: impl Into<String>) -> Self {
        Self { gov, sniff_path: sniff_path.into() }
    }

    pub fn log_rejected_patch(
        &self,
        agent_id: &str,
        repo: &str,
        path: &str,
        change_kind: ChangeKind,
        violations: Vec<String>,
        reason: String,
        before: Option<String>,
        after: Option<String>,
    ) -> Result<CodeEvent, String> {
        let event_id = uuid::Uuid::new_v4().to_string();
        let ts = super::iso8601_now()?;

        let hex_stamp = if violations.iter().any(|v| v == "unsafe_forbidden_core") {
            "0xT00L_UNSAFE_CORE".to_string()
        } else if violations.iter().any(|v| v == "unwrap_forbidden_core") {
            "0xT00L_UNWRAP_CORE".to_string()
        } else {
            self.gov.abort_and_flush_hex.clone()
        };

        let kps = [0.94_f32, 0.30_f32, 0.12_f32];

        let ev = CodeEvent {
            event_id: event_id.clone(),
            timestamp_utc: ts,
            agent_id: agent_id.to_owned(),
            event_type: EventType::PatchRejected,
            repository: repo.to_owned(),
            commit_hash_before: None,
            commit_hash_after: None,
            path: path.to_owned(),
            line_start: None,
            line_end: None,
            change_kind,
            before_content: before,
            after_content: after,
            violations,
            governance_decision: GovernanceDecision::Rejected,
            reason_for_rejection: Some(reason),
            hex_stamp,
            kps_grading: kps,
        };

        ndjson_append(&self.sniff_path, &ev)?;
        sqlite_insert_code_event(&ev)?;

        Ok(ev)
    }
}

fn ndjson_append(path: &str, ev: &CodeEvent) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("open {path} failed: {e}"))?;
    let json = serde_json::to_string(ev).map_err(|e| format!("serialize CodeEvent failed: {e}"))?;
    use std::io::Write;
    writeln!(file, "{json}").map_err(|e| format!("write {path} failed: {e}"))?;
    Ok(())
}

fn sqlite_insert_code_event(_ev: &CodeEvent) -> Result<(), String> {
    // Use same rusqlite conventions as your ObjextDiagnostic / orchestration logs:
    // - WAL mode, FK pragmas enabled.[file:63][file:67]
    Ok(())
}
