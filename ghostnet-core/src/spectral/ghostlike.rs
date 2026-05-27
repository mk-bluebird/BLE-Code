// Filename: ghostnet-core/src/spectral/ghostlike.rs

#![forbid(unsafe_code)]

use crate::spectral::hauntdensity::HauntDensitySnapshot;
use crate::tokens::TokenWindow;
use crate::governance::GovernanceFlags;

/// Haunt band classification for a region-time window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HauntBand {
    ControlLow,
    ControlUpper,
    GhostlikeMedium,
    Restricted,
    Containment,
}

/// Derived ghostlike classification result.
#[derive(Debug, Clone)]
pub struct GhostlikeClassification {
    pub band: HauntBand,
    pub is_ghostlike_band: bool,
    /// Optional hex label for auditing.
    pub hex_stamp: &'static str,
}

/// Safety envelope configuration for "ghostlike" corridors.
#[derive(Debug, Clone)]
pub struct GhostlikeEnvelopeConfig {
    pub h_min: f32,   // e.g., 0.20
    pub h_max: f32,   // e.g., 0.45
    pub x_max: f32,   // risk cap before SPOOK
    pub fear_min: f32,
    pub sanity_min: f32,
    pub hp_min: f32,
}

/// K/P/S grading for a band.
#[derive(Debug, Clone, Copy)]
pub struct KpsGrade {
    pub k: f32,
    pub p: f32,
    pub s: f32,
}

impl GhostlikeClassification {
    pub fn kps_grading(&self) -> KpsGrade {
        match self.band {
            HauntBand::GhostlikeMedium => KpsGrade {
                k: 9.2,
                p: 3.6,
                s: 3.8,
            },
            _ => KpsGrade {
                k: 8.5,
                p: 3.0,
                s: 3.5,
            },
        }
    }
}

fn classify_band(h: f32) -> HauntBand {
    if h < 0.20 {
        HauntBand::ControlLow
    } else if h < 0.30 {
        HauntBand::ControlUpper
    } else if h < 0.60 {
        HauntBand::GhostlikeMedium
    } else if h < 0.90 {
        HauntBand::Restricted
    } else {
        HauntBand::Containment
    }
}

const HEX_GHOSTLIKE_SEMANTIC_V1: &str = "0x47484f53544c494b455f53454d414e5449435f5631";
const HEX_NON_GHOSTLIKE: &str = "0x4841554e545f42414e445f4e4f4e5f47484f53544c494b45";

/// Core classifier: decide whether a window is "ghostlike" under safety guards.
pub fn classify_ghostlike_window(
    snapshot: &HauntDensitySnapshot,
    tokens: &TokenWindow,
    gov: &GovernanceFlags,
    cfg: &GhostlikeEnvelopeConfig,
) -> GhostlikeClassification {
    let h = snapshot.haunt_density; // normalized 0.0..1.0
    let band = classify_band(h);

    let guard_soul_ok = gov.soul_modeling_forbidden;
    let guard_tokens_ok = tokens.fear >= cfg.fear_min
        && tokens.sanity > cfg.sanity_min
        && tokens.hp >= cfg.hp_min
        && tokens.risk_index < cfg.x_max;

    let is_ghostlike = matches!(band, HauntBand::GhostlikeMedium)
        && h >= cfg.h_min
        && h <= cfg.h_max
        && guard_soul_ok
        && guard_tokens_ok;

    let hex_stamp = if is_ghostlike {
        HEX_GHOSTLIKE_SEMANTIC_V1
    } else {
        HEX_NON_GHOSTLIKE
    };

    GhostlikeClassification {
        band,
        is_ghostlike_band: is_ghostlike,
        hex_stamp,
    }
}
