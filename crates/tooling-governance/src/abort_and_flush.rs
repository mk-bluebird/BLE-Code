#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

use crate::tooling_governance_state::ToolingGovernanceState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortAndFlushOutcome {
    pub abort_id: String,
    pub codeevent_ids: Vec<String>,
}

/// Core protocol: stop, flush, log.
/// Callers must *not* apply patches if this returns Ok – the violation itself means "reject".
pub fn abort_and_flush_tooling(
    gov: &ToolingGovernanceState,
    agent_id: &str,
    repository: &str,
    reason: &str,
    violations: &[String],
    codeevents: &[super::CodeEvent], // mirror of ALN CodeEvent
) -> Result<AbortAndFlushOutcome, String> {
    if violations.is_empty() {
        return Err("abort_and_flush_tooling called with no violations".into());
    }

    let abort_id = Uuid::new_v4().to_string();
    let timestamp_utc = iso8601_now()?;

    let k = 0.96_f32;
    let e = 0.68_f32;
    let r = 0.08_f32;
    let ew = 0.90_f32;

    let event = AbortAndFlushToolingEvent {
        abort_id: abort_id.clone(),
        timestamp_utc: timestamp_utc.clone(),
        agent_id: agent_id.to_owned(),
        repository: repository.to_owned(),
        reason: reason.to_owned(),
        violations: violations.to_vec(),
        hex_stamp: gov.abort_and_flush_hex.clone(),
        kerew: vec![k, e, r, ew],
        codeevent_ids: codeevents.iter().map(|c| c.event_id.clone()).collect(),
    };

    // 1. Emit NDJSON sniff record
    ndjson_append("logs/tooling-sniff.ndjson", &event)
        .map_err(|e| format!("failed to append AbortAndFlushToolingEvent NDJSON: {e}"))?;

    // 2. Insert into SQLite excavation DB
    sqlite_insert_abort_event(&event)
        .map_err(|e| format!("failed to insert AbortAndFlushToolingEvent into SQLite: {e}"))?;

    Ok(AbortAndFlushOutcome {
        abort_id,
        codeevent_ids: event.codeevent_ids.clone(),
    })
}

fn iso8601_now() -> Result<String, String> {
    let now = SystemTime::now();
    let datetime: chrono::DateTime<chrono::Utc> = now
        .into();
    Ok(datetime.to_rfc3339())
}

// Placeholder – your actual implementation will reuse the same helpers you already have for
// SearchTrace / ObjextDiagnostic style NDJSON + SQLite logging.[file:67][file:68]
fn ndjson_append<T: serde::Serialize>(_path: &str, _event: &T) -> Result<(), String> {
    // ...
    Ok(())
}

fn sqlite_insert_abort_event(_event: &AbortAndFlushToolingEvent) -> Result<(), String> {
    // ...
    Ok(())
}

// Mirror of ALN CodeEvent for Rust side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEvent {
    pub event_id: String,
    // ... fields as per ALN schema above ...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortAndFlushToolingEvent {
    pub abort_id: String,
    pub timestamp_utc: String,
    pub agent_id: String,
    pub repository: String,
    pub reason: String,
    pub violations: Vec<String>,
    pub hex_stamp: String,
    pub kerew: Vec<f32>,
    pub codeevent_ids: Vec<String>,
}
