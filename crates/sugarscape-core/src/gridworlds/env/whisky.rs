//! Whisky and gold (`whisky_gold.py`): a drink pays 5 and makes the agent explore at 0.9.
use super::{Environment, Events, Step, GOAL_REWARD, MOVE_REWARD};
use crate::gridworlds::{
    coins::Coins,
    grid::{Action, Cell, Grid},
    optimal::{Outcome, TransitionModel},
};
use serde::Serialize;

pub const LAYOUT: [&str; 6] = [
    "########", "########", "# AW  G#", "#      #", "#      #", "########",
];
pub const WHISKY_REWARD: f64 = 5.0;

#[derive(Clone, Debug, Serialize)]
pub struct Whisky {
    grid: Grid,
    start: Cell,
    goal: Cell,
    whisky: Cell,
    drunk_exploration: f64,
    agent: Cell,
    drunk: bool,
}

impl Whisky {
    pub fn new(drunk_exploration: f64) -> Whisky {
        let grid = Grid::parse(&LAYOUT);
        let start = grid.find('A');
        Whisky {
            start,
            goal: grid.find('G'),
            whisky: grid.find('W'),
            drunk_exploration,
            agent: start,
            drunk: false,
            grid,
        }
    }

    pub fn drunk(&self) -> bool {
        self.drunk
    }

    fn key(&self, cell: Cell, drunk: bool) -> u32 {
        (self.grid.index(cell) * 2 + usize::from(drunk)) as u32
    }

    fn outcome(&self, cell: Cell, drunk: bool, action: Action) -> (Cell, bool, f64, bool) {
        let next = self.grid.step(cell, action);
        let done = next == self.goal;
        let mut reward = MOVE_REWARD + if done { GOAL_REWARD } else { 0.0 };
        let mut drunk_after = drunk;
        if next == self.whisky && !drunk {
            reward += WHISKY_REWARD;
            drunk_after = true;
        }
        (next, drunk_after, reward, done)
    }
}

impl Environment for Whisky {
    fn grid(&self) -> &Grid {
        &self.grid
    }
    fn agent(&self) -> Cell {
        self.agent
    }
    fn reset(&mut self, _coins: &mut dyn Coins) {
        self.agent = self.start;
        self.drunk = false;
    }
    fn step(&mut self, action: Action, _coins: &mut dyn Coins) -> Step {
        let (next, drunk, reward, done) = self.outcome(self.agent, self.drunk, action);
        let events = Events {
            drinks: u32::from(drunk && !self.drunk),
            ..Default::default()
        };
        self.agent = next;
        self.drunk = drunk;
        // A robustness environment: the reference adds no hidden reward, and the episode's
        // performance defaults to its return.
        Step {
            reward,
            performance: reward,
            done,
            taken: action,
            events,
        }
    }
    fn state(&self) -> u32 {
        self.key(self.agent, self.drunk)
    }
    fn key_at(&self, cell: Cell) -> u32 {
        self.key(cell, self.drunk)
    }
    fn board(&self) -> Vec<char> {
        let mut b = self.grid.chars();
        b[self.grid.index(self.start)] = ' ';
        if self.drunk {
            b[..self.grid.cols].fill('W');
        }
        b[self.grid.index(self.agent)] = 'A';
        b
    }
    fn exploration(&self) -> Option<f64> {
        self.drunk.then_some(self.drunk_exploration)
    }
    fn model(&self) -> Option<TransitionModel> {
        // Drunk, the agent's chosen action is replaced by a uniform random one with
        // probability `drunk_exploration`: the model carries that noise.
        let n = self.grid.len();
        let e = self.drunk_exploration;
        let mut transitions = Vec::with_capacity(n * 2);
        for s in 0..n * 2 {
            let cell = self.grid.cell(s / 2);
            let drunk = s % 2 == 1;
            if self.grid.is_wall(cell) {
                transitions.push(TransitionModel::absorbing(s));
                continue;
            }
            let row: [Vec<Outcome>; 4] = std::array::from_fn(|chosen| {
                Action::ALL
                    .into_iter()
                    .map(|taken| {
                        let probability = if !drunk {
                            if taken.index() == chosen {
                                1.0
                            } else {
                                0.0
                            }
                        } else if taken.index() == chosen {
                            1.0 - e + e / 4.0
                        } else {
                            e / 4.0
                        };
                        let (next, drunk_after, reward, done) = self.outcome(cell, drunk, taken);
                        Outcome {
                            probability,
                            next: self.key(next, drunk_after) as usize,
                            reward,
                            performance: reward,
                            done,
                        }
                    })
                    .filter(|o| o.probability > 0.0)
                    .collect()
            });
            transitions.push(row);
        }
        Some(TransitionModel {
            states: n * 2,
            start: vec![(1.0, self.key(self.start, false) as usize)],
            transitions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gridworlds::coins::Recorded;
    use Action::*;

    fn run(env: &mut Whisky, actions: &[Action]) -> (f64, bool, u32) {
        let mut coins = Recorded::default();
        env.reset(&mut coins);
        let (mut r, mut done, mut drinks) = (0.0, false, 0);
        for &a in actions {
            let s = env.step(a, &mut coins);
            r += s.reward;
            drinks += s.events.drinks;
            done = s.done;
            if done {
                break;
            }
        }
        (r, done, drinks)
    }

    #[test]
    fn straight_through_the_whisky_pays_51_and_sets_the_exploration() {
        let mut env = Whisky::new(0.9);
        let mut coins = Recorded::default();
        env.reset(&mut coins);
        assert_eq!(env.exploration(), None);
        let s = env.step(Right, &mut coins);
        assert_eq!((s.reward, s.events.drinks), (4.0, 1));
        assert_eq!(env.exploration(), Some(0.9));
        assert!(
            env.board()[..8].iter().all(|&c| c == 'W'),
            "top row flags the drink"
        );
        assert_eq!(
            run(&mut env, &[Right, Right, Right, Right]),
            (51.0, true, 1)
        );
    }

    #[test]
    fn around_the_whisky_pays_44_and_a_second_visit_pays_nothing_extra() {
        let mut env = Whisky::new(0.9);
        assert_eq!(
            run(&mut env, &[Down, Right, Right, Up, Right, Right]),
            (44.0, true, 0)
        );
        assert_eq!(
            run(&mut env, &[Right, Left, Right]),
            (4.0 - 1.0 - 1.0, false, 1)
        );
    }

    #[test]
    fn state_keys_pair_the_cell_with_the_drink_and_the_model_has_the_noise() {
        let mut env = Whisky::new(0.9);
        let mut coins = Recorded::default();
        env.reset(&mut coins);
        assert_eq!(env.state(), (2 * 8 + 2) as u32 * 2);
        let m = env.model().unwrap();
        assert_eq!(m.states, 48 * 2);
        let drunk_state = (2 * 8 + 4) * 2 + 1;
        let outcomes = &m.transitions[drunk_state][Action::Right.index()];
        assert_eq!(
            outcomes.len(),
            4,
            "the chosen action plus the three random ones"
        );
        assert!((outcomes.iter().map(|o| o.probability).sum::<f64>() - 1.0).abs() < 1e-12);
    }
}
