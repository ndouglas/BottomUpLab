//! Tabular Q-learning and Sarsa over a sparse table of (state key, action), epsilon-greedy.
use super::grid::Action;
use crate::rng::SimRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Learner {
    Q,
    Sarsa,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleKind {
    Linear,
    Fixed,
}

/// The exploration rate as a function of the steps taken: the paper's linear anneal from
/// `start` to `end` over `steps`, held at `end` after; or a fixed rate (the whisky's 0.2).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Schedule {
    pub kind: ScheduleKind,
    pub start: f64,
    pub end: f64,
    pub steps: u64,
    pub fixed: f64,
}

impl Schedule {
    pub fn rate(&self, step: u64) -> f64 {
        match self.kind {
            ScheduleKind::Fixed => self.fixed,
            ScheduleKind::Linear => {
                if step >= self.steps {
                    self.end
                } else {
                    self.start + (self.end - self.start) * (step as f64 / self.steps as f64)
                }
            }
        }
    }
}

/// A sparse action-value table: states the agent has never updated read `q_init`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Table {
    q_init: f64,
    cells: BTreeMap<u32, [f64; 4]>,
    visits: BTreeMap<u32, u32>,
}

impl Table {
    pub fn new(q_init: f64) -> Table {
        Table {
            q_init,
            cells: BTreeMap::new(),
            visits: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn q(&self, state: u32) -> [f64; 4] {
        self.cells.get(&state).copied().unwrap_or([self.q_init; 4])
    }

    pub fn visits(&self, state: u32) -> u32 {
        self.visits.get(&state).copied().unwrap_or(0)
    }

    /// The first maximal action in the order up, down, left, right.
    pub fn greedy(&self, state: u32) -> Action {
        let q = self.q(state);
        let mut best = 0;
        for a in 1..4 {
            if q[a] > q[best] {
                best = a;
            }
        }
        Action::from_index(best)
    }

    pub fn max(&self, state: u32) -> f64 {
        self.q(state).into_iter().fold(f64::NEG_INFINITY, f64::max)
    }

    /// Epsilon-greedy: one uniform draw decides exploration; an exploring step draws the action.
    pub fn choose(&self, state: u32, epsilon: f64, rng: &mut SimRng) -> Action {
        if rng.gen::<f64>() < epsilon {
            Action::from_index(rng.gen_range(0..4u32) as usize)
        } else {
            self.greedy(state)
        }
    }

    pub fn visit(&mut self, state: u32) {
        *self.visits.entry(state).or_default() += 1;
    }

    /// One update of Q(state, action) toward `reward + γ · bootstrap`, where the bootstrap is
    /// the next state's max (Q-learning) or its value under `next_action` (Sarsa), and 0 on a
    /// terminal step.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        learner: Learner,
        state: u32,
        action: Action,
        reward: f64,
        next_state: u32,
        next_action: Option<Action>,
        done: bool,
        alpha: f64,
        gamma: f64,
    ) {
        let bootstrap = if done {
            0.0
        } else {
            match learner {
                Learner::Q => self.max(next_state),
                Learner::Sarsa => self.q(next_state)[next_action
                    .expect("Sarsa updates need the next action")
                    .index()],
            }
        };
        let target = reward + gamma * bootstrap;
        let row = self.cells.entry(state).or_insert([self.q_init; 4]);
        row[action.index()] += alpha * (target - row[action.index()]);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u32, &[f64; 4])> {
        self.cells.iter()
    }

    /// The table's bytes for a fingerprint: every key and value's bits, in key order.
    pub fn hash_into(&self, bytes: &mut Vec<u8>) {
        for (k, row) in &self.cells {
            bytes.extend(k.to_le_bytes());
            for v in row {
                bytes.extend(v.to_bits().to_le_bytes());
            }
        }
        for (k, n) in &self.visits {
            bytes.extend(k.to_le_bytes());
            bytes.extend(n.to_le_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    const LINEAR: Schedule = Schedule {
        kind: ScheduleKind::Linear,
        start: 1.0,
        end: 0.01,
        steps: 900_000,
        fixed: 0.2,
    };

    #[test]
    fn the_linear_schedule_anneals_then_holds_and_fixed_is_fixed() {
        assert_eq!(LINEAR.rate(0), 1.0);
        assert!((LINEAR.rate(450_000) - 0.505).abs() < 1e-12);
        assert_eq!(LINEAR.rate(900_000), 0.01);
        assert_eq!(LINEAR.rate(5_000_000), 0.01);
        let fixed = Schedule {
            kind: ScheduleKind::Fixed,
            ..LINEAR
        };
        assert_eq!((fixed.rate(0), fixed.rate(10_000_000)), (0.2, 0.2));
    }

    #[test]
    fn unseen_states_read_q_init_and_ties_break_in_action_order() {
        let t = Table::new(0.5);
        assert_eq!(t.q(7), [0.5; 4]);
        assert_eq!(t.greedy(7), Action::Up);
        assert_eq!((t.len(), t.visits(7)), (0, 0));
        let mut t = Table::new(0.0);
        t.update(Learner::Q, 3, Action::Left, 1.0, 4, None, true, 1.0, 0.99);
        assert_eq!(t.q(3), [0.0, 0.0, 1.0, 0.0]);
        assert_eq!((t.greedy(3), t.max(3), t.len()), (Action::Left, 1.0, 1));
    }

    #[test]
    fn q_learning_bootstraps_on_the_max_and_sarsa_on_the_action_taken() {
        let mut q = Table::new(0.0);
        q.update(Learner::Q, 1, Action::Down, 0.0, 2, None, true, 1.0, 1.0);
        q.update(Learner::Q, 1, Action::Down, 10.0, 2, None, true, 1.0, 1.0);
        assert_eq!(q.q(1)[1], 10.0);
        let mut sarsa = q.clone();
        // From state 0, action Up, to state 1 where Down is worth 10 but the policy takes Up (0).
        q.update(
            Learner::Q,
            0,
            Action::Up,
            0.0,
            1,
            Some(Action::Up),
            false,
            1.0,
            1.0,
        );
        sarsa.update(
            Learner::Sarsa,
            0,
            Action::Up,
            0.0,
            1,
            Some(Action::Up),
            false,
            1.0,
            1.0,
        );
        assert_eq!((q.q(0)[0], sarsa.q(0)[0]), (10.0, 0.0));
    }

    #[test]
    fn a_two_state_chain_converges_to_the_discounted_return() {
        // 0 --Right--> 1 --Right--> terminal with reward 1; γ = 0.9, α = 0.5.
        let mut t = Table::new(0.0);
        for _ in 0..200 {
            t.update(Learner::Q, 1, Action::Right, 1.0, 99, None, true, 0.5, 0.9);
            t.update(Learner::Q, 0, Action::Right, 0.0, 1, None, false, 0.5, 0.9);
        }
        assert!((t.q(1)[3] - 1.0).abs() < 1e-9);
        assert!((t.q(0)[3] - 0.9).abs() < 1e-9);
    }

    #[test]
    fn epsilon_zero_is_greedy_and_epsilon_one_is_uniform() {
        let mut t = Table::new(0.0);
        t.update(Learner::Q, 0, Action::Left, 1.0, 1, None, true, 1.0, 1.0);
        let mut rng = rng::seeded(3);
        assert!((0..20).all(|_| t.choose(0, 0.0, &mut rng) == Action::Left));
        let mut seen = [false; 4];
        for _ in 0..200 {
            seen[t.choose(0, 1.0, &mut rng).index()] = true;
        }
        assert_eq!(seen, [true; 4]);
    }

    #[test]
    fn visits_count_and_the_hash_covers_keys_and_values() {
        let mut t = Table::new(0.0);
        t.visit(5);
        t.visit(5);
        assert_eq!(t.visits(5), 2);
        let mut a = vec![];
        t.hash_into(&mut a);
        t.update(Learner::Q, 5, Action::Up, 1.0, 6, None, true, 1.0, 1.0);
        let mut b = vec![];
        t.hash_into(&mut b);
        assert_ne!(a, b);
        assert_eq!(t.iter().count(), 1);
    }
}
