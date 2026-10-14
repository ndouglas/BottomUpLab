//! Exact optima by finite-horizon value iteration over each environment's transition model.
//! (The algorithms arrive with the learner's tests; this file first carries the model type.)

/// One possible result of taking an action: its probability, the next state, the step's
/// visible reward and hidden performance, and whether the episode ends there.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub probability: f64,
    pub next: usize,
    pub reward: f64,
    pub performance: f64,
    pub done: bool,
}

/// An environment's exact model: `transitions[state][action]` lists the outcomes; `start` is
/// the distribution over the first state (the reset coins).
#[derive(Clone, Debug, PartialEq)]
pub struct TransitionModel {
    pub states: usize,
    pub start: Vec<(f64, usize)>,
    pub transitions: Vec<[Vec<Outcome>; 4]>,
}

impl TransitionModel {
    /// A terminal self-loop for states that cannot occur (a wall cell), so value iteration
    /// never reads them.
    pub fn absorbing(state: usize) -> [Vec<Outcome>; 4] {
        std::array::from_fn(|_| {
            vec![Outcome {
                probability: 1.0,
                next: state,
                reward: 0.0,
                performance: 0.0,
                done: true,
            }]
        })
    }
}
