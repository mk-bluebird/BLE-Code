// Rust 2024, rust-version = "1.85", kani-verifier = "0.67"
// !forbid(unsafe_code)

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum State {
    Safe,
    Risky,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Action {
    Cautious,
    Aggressive,
}

/// Fear index f(s,a) in [0.0,1.0], corridor is [0.31,0.68].
fn fear_index(s: State, a: Action) -> f32 {
    match (s, a) {
        (State::Safe,   Action::Cautious)  => 0.35,
        (State::Safe,   Action::Aggressive)=> 0.70, // above upper bound
        (State::Risky,  Action::Cautious)  => 0.50,
        (State::Risky,  Action::Aggressive)=> 0.80, // above upper bound
    }
}

/// Base reward U(s,a) before constraints.
fn base_reward(s: State, a: Action) -> f32 {
    match (s, a) {
        (State::Safe,  Action::Cautious)   => 1.0,
        (State::Safe,  Action::Aggressive) => 3.0,
        (State::Risky, Action::Cautious)   => 0.5,
        (State::Risky, Action::Aggressive) => 4.0,
    }
}

/// Transition probabilities P(s'|s,a) as a small fixed matrix.
fn next_state_probs(s: State, a: Action) -> [(State, f32); 2] {
    match (s, a) {
        (State::Safe,  Action::Cautious) => [
            (State::Safe, 0.9),
            (State::Risky, 0.1),
        ],
        (State::Safe,  Action::Aggressive) => [
            (State::Safe, 0.2),
            (State::Risky, 0.8),
        ],
        (State::Risky, Action::Cautious) => [
            (State::Safe, 0.3),
            (State::Risky, 0.7),
        ],
        (State::Risky, Action::Aggressive) => [
            (State::Safe, 0.1),
            (State::Risky, 0.9),
        ],
    }
}

/// Fear corridor [0.31, 0.68].
fn g_fear_low(f: f32) -> f32 {
    let lower = 0.31_f32;
    if f < lower { lower - f } else { 0.0 }
}

fn g_fear_high(f: f32) -> f32 {
    let upper = 0.68_f32;
    if f > upper { f - upper } else { 0.0 }
}

/// Lagrangian reward R_lambda(s,a) = U(s,a) - lambda_low * g_low - lambda_high * g_high.
fn lagrangian_reward(
    s: State,
    a: Action,
    lambda_low: f32,
    lambda_high: f32,
) -> f32 {
    let f = fear_index(s, a);
    let g_low = g_fear_low(f);
    let g_high = g_fear_high(f);
    base_reward(s, a) - lambda_low * g_low - lambda_high * g_high
}

/// Simple value-iteration for fixed lambda, discounted infinite horizon.
pub fn value_iteration(
    lambda_low: f32,
    lambda_high: f32,
    gamma: f32,
    tol: f32,
    max_iter: usize,
) -> (f32, f32) {
    // V(Safe), V(Risky)
    let mut v_safe: f32 = 0.0;
    let mut v_risky: f32 = 0.0;

    for _ in 0..max_iter {
        let mut delta = 0.0_f32;

        // State Safe
        let q_safe_caut = {
            let r = lagrangian_reward(State::Safe, Action::Cautious, lambda_low, lambda_high);
            let vs = next_state_probs(State::Safe, Action::Cautious)
                .iter()
                .map(|(sp, p)| {
                    let v = match sp {
                        State::Safe => v_safe,
                        State::Risky => v_risky,
                    };
                    p * v
                })
                .sum::<f32>();
            r + gamma * vs
        };
        let q_safe_aggr = {
            let r = lagrangian_reward(State::Safe, Action::Aggressive, lambda_low, lambda_high);
            let vs = next_state_probs(State::Safe, Action::Aggressive)
                .iter()
                .map(|(sp, p)| {
                    let v = match sp {
                        State::Safe => v_safe,
                        State::Risky => v_risky,
                    };
                    p * v
                })
                .sum::<f32>();
            r + gamma * vs
        };
        let v_safe_new = q_safe_caut.max(q_safe_aggr);

        // State Risky
        let q_risky_caut = {
            let r = lagrangian_reward(State::Risky, Action::Cautious, lambda_low, lambda_high);
            let vs = next_state_probs(State::Risky, Action::Cautious)
                .iter()
                .map(|(sp, p)| {
                    let v = match sp {
                        State::Safe => v_safe,
                        State::Risky => v_risky,
                    };
                    p * v
                })
                .sum::<f32>();
            r + gamma * vs
        };
        let q_risky_aggr = {
            let r = lagrangian_reward(State::Risky, Action::Aggressive, lambda_low, lambda_high);
            let vs = next_state_probs(State::Risky, Action::Aggressive)
                .iter()
                .map(|(sp, p)| {
                    let v = match sp {
                        State::Safe => v_safe,
                        State::Risky => v_risky,
                    };
                    p * v
                })
                .sum::<f32>();
            r + gamma * vs
        };
        let v_risky_new = q_risky_caut.max(q_risky_aggr);

        delta = delta.max((v_safe_new - v_safe).abs());
        delta = delta.max((v_risky_new - v_risky).abs());

        v_safe = v_safe_new;
        v_risky = v_risky_new;

        if delta < tol {
            break;
        }
    }

    (v_safe, v_risky)
}

/// Greedy policy and fear-constraint costs for fixed lambda.
/// Returns ((V_safe, V_risky), pi_safe, pi_risky, C_low, C_high).
pub fn greedy_policy_and_costs(
    lambda_low: f32,
    lambda_high: f32,
    gamma: f32,
    horizon: usize,
) -> ((f32, f32), Action, Action, f32, f32) {
    let (v_safe, v_risky) = value_iteration(lambda_low, lambda_high, gamma, 1.0e-6, 10_000);

    // Greedy policy
    let best_action = |s: State| -> Action {
        let acts = [Action::Cautious, Action::Aggressive];
        let mut best_q = f32::NEG_INFINITY;
        let mut best_a = Action::Cautious;
        for a in acts {
            let r = lagrangian_reward(s, a, lambda_low, lambda_high);
            let vs = next_state_probs(s, a)
                .iter()
                .map(|(sp, p)| {
                    let v = match sp {
                        State::Safe => v_safe,
                        State::Risky => v_risky,
                    };
                    p * v
                })
                .sum::<f32>();
            let q = r + gamma * vs;
            if q > best_q {
                best_q = q;
                best_a = a;
            }
        }
        best_a
    };

    let pi_safe = best_action(State::Safe);
    let pi_risky = best_action(State::Risky);

    // Simulate to estimate discounted constraint costs C_low, C_high.
    let mut s = State::Safe;
    let mut c_low: f32 = 0.0;
    let mut c_high: f32 = 0.0;
    let mut disc: f32 = 1.0;

    for _ in 0..horizon {
        let a = match s {
            State::Safe => pi_safe,
            State::Risky => pi_risky,
        };
        let f = fear_index(s, a);
        c_low += disc * g_fear_low(f);
        c_high += disc * g_fear_high(f);
        disc *= gamma;

        // deterministic next state sampling using a simple fixed threshold
        let probs = next_state_probs(s, a);
        // here we pick the more probable next state for determinism
        s = if probs[0].1 >= probs[1].1 {
            probs[0].0
        } else {
            probs[1].0
        };
    }

    ((v_safe, v_risky), pi_safe, pi_risky, c_low, c_high)
}
