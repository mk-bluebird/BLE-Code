# Ghostlike Band (Spectral Only)

- “Ghostlike” is a human-level label for a specific Haunt-Density band and token envelope.
- In code and schemas we use only spectral terms: `HauntBand::GhostlikeMedium`, `haunt_density`, `spectral_object`, `risk_index`, tokens, and governance flags.

A region–time window is considered in the ghostlike band iff:

- HauntDensity \(H\) is in a medium corridor (default 0.20–0.45).
- Zone is in Control upper edge or Monitored, never Restricted/Containment.
- Risk index stays below the SPOOK shutdown threshold and OperationMode is Normal or Guarded.
- FEAR, SANITY, and HP remain above configured floors.
- `soul_modeling_forbidden = true` and `spectral_quantification_active = true`.
- No person IDs, soul fields, or moral labels are attached; only `RegionSessionKey = (location_bucket, time_bucket, session_id)`.

Implementation notes:

- ALN shard: `schemas/spectral.ghostlike.band.v1.aln`.
- Rust core classifier: `ghostnet-core/src/spectral/ghostlike.rs`.
- JS helper: `ui/lib/ghostlikeClassifier.js`.
- Optional Lua orchestration: `scripts/spectrescripts/ghostlike.lua`.

Hex-stamp for this semantic band:

- `0x47484f53544c494b455f53454d414e5449435f5631` (`GHOSTLIKE_SEMANTIC_V1`).
