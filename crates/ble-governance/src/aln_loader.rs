#![forbid(unsafe_code)]

use crate::{BleProfileShard, GovernanceInvariantError};
use aln_core::AlnError;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BleProfileLoadError {
    #[error("I/O error reading profile: {0}")]
    Io(#[from] std::io::Error),

    #[error("ALN parse error: {0}")]
    Aln(#[from] AlnError),

    #[error("Governance invariants violated: {0}")]
    Invariants(#[from] GovernanceInvariantError),
}

/// Load an ALN file from disk into a typed object.
fn load_aln_file<T>(path: &std::path::Path) -> Result<T, BleProfileLoadError>
where
    T: DeserializeOwned,
{
    let text = std::fs::read_to_string(path)?;
    let value = aln_core::from_aln_str::<T>(&text)?;
    Ok(value)
}

/// Load `.ble-profile.aln` and validate its invariants.
/// This is the canonical entry point for guard and tooling.
pub fn load_ble_profile_shard(
    path: &std::path::Path,
) -> Result<BleProfileShard, BleProfileLoadError> {
    let shard: BleProfileShard = load_aln_file(path)?;
    shard.validate_invariants()?;
    Ok(shard)
}
