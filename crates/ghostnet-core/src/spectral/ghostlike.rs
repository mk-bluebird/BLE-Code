// Filename: ghostnet-core/src/spectral/ghostlike.rs

use crate::spectral::hauntdensity::HauntDensitySnapshot;
use crate::tokens::TokenWindow;
use crate::governance::GovernanceFlags;

/// Haunt band classification for a region-time window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HauntBand {
    ControlLow,
    ControlUpper,
    GhostlikeMedium,  // "ghostlike" corridor H ∈ [0.20, 0.45]
    Restricted,
    Containment,
}

/// Derived ghostlike classification result.
#[derive(Debug, Clone)]
pub struct GhostlikeClassification {
    pub band: HauntBand,
    pub is_ghostlike_band: bool,
}

impl GhostlikeClassification {
    pub fn kps_grading(&self) -> (f32, f32, f32) {
        // Example: modest psych + spectral load for ghostlike band.
        match self.band {
            HauntBand::GhostlikeMedium => (9.0, 3.5, 3.8),
            _ => (8.5, 3.0, 3.5),
        }
    }

    /// Optional: return a hex-stamp label for auditing.
    pub fn hex_stamp(&self) -> &'static str {
        match self.band {
            HauntBand::GhostlikeMedium => "0x47484f53544c494b455f53454d414e5449435f5631",
            _ => "0x4841554e545f42414e445f4e4f4e5f47484f53544c494b45",
        }
    }
}

/// Safety envelope configuration for "ghostlike" corridors.
#[derive(Debug, Clone)]
pub struct GhostlikeEnvelopeConfig {
    pub h_min: f32,       // e.g. 0.20
    pub h_max: f32,       // e.g. 0.45
    pub x_max: f32,       // risk cap before SPOOK
    pub fear_min: f32,
    pub sanity_min: f32,
    pub hp_min: f32,
}

/// Classify a window into a HauntBand based on H.
fn classify_band(h: f32) -> HauntBand {
    if h < 0.20 {
        HauntBand::ControlLow
    } else if h < 0.30 {
        HauntBand::ControlUpper
    } else if h < 0.60 {
        // This [0.20, 0.45] subset within Control/Monitored is what we call "ghostlike".
        HauntBand::GhostlikeMedium
    } else if h < 0.90 {
        HauntBand::Restricted
    } else {
        HauntBand::Containment
    }
}

/// Core classifier: decides whether a window is "ghostlike" under safety guards.
pub fn classify_ghostlike_window(
    snapshot: &HauntDensitySnapshot,
    tokens: &TokenWindow,
    gov: &GovernanceFlags,
    cfg: &GhostlikeEnvelopeConfig,
) -> GhostlikeClassification {
    let h = snapshot.h; // normalized Haunt-Density 0..1
    let band = classify_band(h);

    let guard_soul_ok = gov.soul_modeling_forbidden; // must be true
    let guard_tokens_ok =
        tokens.fear >= cfg.fear_min &&
        tokens.sanity > cfg.sanity_min &&
        tokens.hp >= cfg.hp_min &&
        tokens.risk_index < cfg.x_max;

    let is_ghostlike =
        matches!(band, HauntBand::GhostlikeMedium) &&
        h >= cfg.h_min &&
        h <= cfg.h_max &&
        guard_soul_ok &&
        guard_tokens_ok;

    GhostlikeClassification {
        band,
        is_ghostlike_band: is_ghostlike,
    }
}
