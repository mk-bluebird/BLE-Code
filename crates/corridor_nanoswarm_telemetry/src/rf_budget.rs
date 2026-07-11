// File: crates/corridor_nanoswarm_telemetry/src/rf_budget.rs
// License: MIT OR Apache-2.0

/// RF budget per corridor slice, to keep nanoswarm RF emissions
/// within bee RF corridor bands and host budgets.
///
/// This is intended to be bound to BeeRF CorridorBand entries and
/// HostBudgetBand / EcoBand structures from `corridor_core`.
#[derive(Clone, Copy, Debug)]
pub struct RFBudgetSlice {
    pub slice_id: u16,
    /// Maximum duty-cycle for nanoswarm transmissions (0.0 - 1.0).
    pub max_duty_cycle: f32,
    /// Maximum TX power in dBm (e.g. -20.0).
    pub max_tx_power_dbm: f32,
    /// Aggregated RF risk coordinate threshold for nanoswarm (0.0 - 1.0).
    pub max_r_rf_nano: f32,
}

impl RFBudgetSlice {
    pub fn is_within_budget(
        &self,
        duty_cycle: f32,
        tx_power_dbm: f32,
        r_rf_nano: f32,
    ) -> bool {
        duty_cycle <= self.max_duty_cycle
            && tx_power_dbm <= self.max_tx_power_dbm
            && r_rf_nano <= self.max_r_rf_nano
    }
}

/// Evaluator that tracks RF usage by nanoswarm nodes per slice.
#[derive(Clone, Debug)]
pub struct RFBudgetEvaluator {
    budgets: heapless::Vec<RFBudgetSlice, 32>,
}

impl RFBudgetEvaluator {
    pub fn new() -> Self {
        RFBudgetEvaluator {
            budgets: heapless::Vec::new(),
        }
    }

    pub fn add_budget(&mut self, budget: RFBudgetSlice) -> Result<(), ()> {
        self.budgets.push(budget).map_err(|_| ())
    }

    pub fn get_budget(&self, slice_id: u16) -> Option<RFBudgetSlice> {
        self.budgets
            .iter()
            .copied()
            .find(|b| b.slice_id == slice_id)
    }

    /// Check if a proposed RF usage is allowed for a slice.
    pub fn check(
        &self,
        slice_id: u16,
        duty_cycle: f32,
        tx_power_dbm: f32,
        r_rf_nano: f32,
    ) -> bool {
        if let Some(budget) = self.get_budget(slice_id) {
            budget.is_within_budget(duty_cycle, tx_power_dbm, r_rf_nano)
        } else {
            // No budget defined => no corridor, no act.
            false
        }
    }
}
