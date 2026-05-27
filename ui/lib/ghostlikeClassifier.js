// Filename: ui/lib/ghostlikeClassifier.js

export function classifyHauntBand(H) {
  if (H < 0.20) return "ControlLow";
  if (H < 0.30) return "ControlUpper";
  if (H < 0.60) return "GhostlikeMedium";
  if (H < 0.90) return "Restricted";
  return "Containment";
}

export function isGhostlikeWindow(event, cfg) {
  const H = event.haunt_density;
  const band = classifyHauntBand(H);

  const tokens = event.tokens || {};
  const gov = event.governance || {};

  const guardSoulOk = gov.soul_modeling_forbidden === true;
  const guardTokensOk =
    tokens.fear >= cfg.fear_min &&
    tokens.sanity > cfg.sanity_min &&
    tokens.hp >= cfg.hp_min &&
    tokens.risk_index < cfg.x_max;

  const inBand = H >= cfg.h_min && H <= cfg.h_max &&
                 band === "GhostlikeMedium";

  return {
    band,
    isGhostlikeBand: inBand && guardSoulOk && guardTokensOk
  };
}

// Example default config matching Rust:
export const defaultGhostlikeCfg = {
  h_min: 0.20,
  h_max: 0.45,
  x_max: 0.75,
  fear_min: 0.10,
  sanity_min: 0.05,
  hp_min: 0.20
};
