// Inside your TokenEvent or RegionWindow event:

use crate::spectral::ghostlike::{classify_ghostlike_window, GhostlikeEnvelopeConfig, HauntBand};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionWindowEvent {
    pub key: RegionSessionKey,
    pub haunt_density: f32,
    pub zone: HauntZone,
    pub tokens: TokenWindow,
    pub governance: GovernanceFlags,

    // Derived:
    pub haunt_band: HauntBand,
    pub is_ghostlike_band: bool,
    pub ghostlike_hex_stamp: String,
}

impl RegionWindowEvent {
    pub fn derive_ghostlike(mut self, cfg: &GhostlikeEnvelopeConfig) -> Self {
        let snapshot = HauntDensitySnapshot {
            key: self.key.clone(),
            haunt_density: self.haunt_density,
            zone: self.zone,
        };

        let cls = classify_ghostlike_window(&snapshot, &self.tokens, &self.governance, cfg);

        self.haunt_band = cls.band;
        self.is_ghostlike_band = cls.is_ghostlike_band;
        self.ghostlike_hex_stamp = cls.hex_stamp.to_string();
        self
    }
}
