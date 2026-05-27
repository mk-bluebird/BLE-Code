-- Filename: scripts/spectrescripts/ghostlike.lua
-- Purpose: XR / haptic orchestration inside the ghostlike corridor.
-- This script never accesses identity or soul fields; it only reads
-- pre-classified haunt_band and safety tokens from the host.

local GHOSTLIKE_BAND = "GhostlikeMedium"

local function safe_intensity_scale(tokens)
  -- Tokens are normalized floats 0.0..1.0 from the host.
  if tokens.sanity < 0.1 or tokens.hp < 0.2 then
    return 0.0
  end
  if tokens.fear > 0.7 then
    return 0.2
  end
  return 0.5
end

-- entrypoint: called by host with a decoded RegionWindowEvent-like table.
function run_ghostlike_effect(window)
  if window.haunt_band ~= GHOSTLIKE_BAND or not window.is_ghostlike_band then
    return { intensity = 0.0, mode = "off" }
  end

  local tokens = window.tokens or {}
  local intensity = safe_intensity_scale(tokens)

  return {
    intensity = intensity,
    mode = "boo-soft",  -- host interprets this symbolically
    hex_stamp = window.ghostlike_hex_stamp,
  }
end
