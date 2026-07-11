// File: crates/corridor_nanoswarm_telemetry/src/mac.rs
// License: MIT OR Apache-2.0

use crate::rf_budget::RFBudgetEvaluator;

/// MAC parameters for nanoswarm gossip under RF ceilings.
#[derive(Clone, Copy, Debug)]
pub struct MacParams {
    /// Superframe duration in seconds.
    pub superframe_s: f32,
    /// Maximum TX attempts per node per superframe.
    pub max_tx_per_superframe: u8,
    /// Base transmit probability per superframe (0.0 - 1.0).
    pub base_p_tx: f32,
}

/// MAC state per node.
#[derive(Clone, Debug)]
pub struct MacState {
    pub slice_id: u16,
    pub node_id: u16,
    pub harvested_energy_j: f32,
    pub tx_attempts_current_sf: u8,
}

impl MacState {
    pub fn new(slice_id: u16, node_id: u16) -> Self {
        MacState {
            slice_id,
            node_id,
            harvested_energy_j: 0.0,
            tx_attempts_current_sf: 0,
        }
    }

    pub fn reset_superframe(&mut self) {
        self.tx_attempts_current_sf = 0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacDecision {
    /// Node will attempt a transmission in this superframe.
    Transmit,
    /// Node will remain silent (no TX).
    Silent,
}

/// Decide whether a node should transmit in the current superframe,
/// based on MAC parameters, harvested energy, RF budgets, and an
/// externally computed RF risk coordinate (r_rf_total).
pub fn decide_tx(
    params: &MacParams,
    state: &mut MacState,
    rf_eval: &RFBudgetEvaluator,
    duty_cycle_estimate: f32,
    tx_power_dbm: f32,
    r_rf_nano: f32,
    r_rf_total: f32,
    random_u: f32,
) -> MacDecision {
    // No corridor, no act if RF budget is missing or violated.
    if !rf_eval.check(
        state.slice_id,
        duty_cycle_estimate,
        tx_power_dbm,
        r_rf_nano,
    ) {
        return MacDecision::Silent;
    }

    // If total RF risk is near ceiling, reduce probability.
    let mut p_tx = params.base_p_tx;
    if r_rf_total > 0.8 {
        p_tx *= 0.25;
    } else if r_rf_total > 0.6 {
        p_tx *= 0.5;
    }

    // Enforce per-superframe TX attempt limit.
    if state.tx_attempts_current_sf >= params.max_tx_per_superframe {
        return MacDecision::Silent;
    }

    // Energy-aware gating: require some harvested energy threshold.
    if state.harvested_energy_j < 0.001 {
        return MacDecision::Silent;
    }

    if random_u < p_tx {
        state.tx_attempts_current_sf = state.tx_attempts_current_sf.saturating_add(1);
        MacDecision::Transmit
    } else {
        MacDecision::Silent
    }
}
