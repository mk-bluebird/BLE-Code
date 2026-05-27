// Filename: ui/lib/ghostlikeClassifier.js

/**
 * Classify a normalized Haunt-Density value into a discrete band.
 * H is expected to be in the range [0.0, 1.0].
 */
export function classifyHauntBand(H) {
  if (H < 0.20) return "ControlLow";
  if (H < 0.30) return "ControlUpper";
  if (H < 0.60) return "GhostlikeMedium";
  if (H < 0.90) return "Restricted";
  return "Containment";
}

/**
 * Default ghostlike envelope config, aligned with GhostNet core.
 */
export const defaultGhostlikeCfg = {
  h_min: 0.20,
  h_max: 0.45,
  x_max: 0.75,
  fear_min: 0.10,
  sanity_min: 0.05,
  hp_min: 0.20,
};

/**
 * Classify a region-time window as ghostlike or not.
 * Expects an event with:
 *   - haunt_density: number (0.0..1.0)
 *   - tokens: { fear, sanity, hp, risk_index }
 *   - governance: { soul_modeling_forbidden: boolean }
 * cfg is the envelope; defaultGhostlikeCfg is a safe default.
 */
export function isGhostlikeWindow(event, cfg = defaultGhostlikeCfg) {
  if (!event || typeof event.haunt_density !== "number") {
    return {
      band: "ControlLow",
      isGhostlikeBand: false,
      hexStamp: "0x4841554e545f42414e445f4e4f4e5f47484f53544c494b45",
    };
  }

  const H = event.haunt_density;
  const band = classifyHauntBand(H);

  const tokens = event.tokens || {};
  const gov = event.governance || {};

  const fear = typeof tokens.fear === "number" ? tokens.fear : 0.0;
  const sanity = typeof tokens.sanity === "number" ? tokens.sanity : 0.0;
  const hp = typeof tokens.hp === "number" ? tokens.hp : 0.0;
  const riskIndex =
    typeof tokens.risk_index === "number" ? tokens.risk_index : Number.POSITIVE_INFINITY;

  const guardSoulOk = gov.soul_modeling_forbidden === true;

  const guardTokensOk =
    fear >= cfg.fear_min &&
    sanity > cfg.sanity_min &&
    hp >= cfg.hp_min &&
    riskIndex < cfg.x_max;

  const inBand =
    H >= cfg.h_min &&
    H <= cfg.h_max &&
    band === "GhostlikeMedium";

  const isGhostlikeBand = inBand && guardSoulOk && guardTokensOk;

  const hexStamp = isGhostlikeBand
    ? "0x47484f53544c494b455f53454d414e5449435f5631"
    : "0x4841554e545f42414e445f4e4f4e5f47484f53544c494b45";

  return {
    band,
    isGhostlikeBand,
    hexStamp,
  };
}
