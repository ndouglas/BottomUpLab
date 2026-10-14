//! The environments' random draws behind one interface, so tests can replay recorded ones.
use crate::rng::SimRng;
use rand::Rng;
use std::collections::VecDeque;

/// The reference environments draw `np.random.rand()` / `random()` for coins and
/// `np.random.choice` for picks. Each environment compares its draw the way the reference does.
pub trait Coins {
    /// A uniform draw on [0, 1).
    fn uniform(&mut self) -> f64;
    /// An index in `0..n`.
    fn choice(&mut self, n: u32) -> u32;
}

impl Coins for SimRng {
    fn uniform(&mut self) -> f64 {
        self.gen::<f64>()
    }
    fn choice(&mut self, n: u32) -> u32 {
        self.gen_range(0..n)
    }
}

/// Replays draws recorded from the reference (the docking fixtures).
#[derive(Clone, Debug, Default)]
pub struct Recorded {
    pub uniforms: VecDeque<f64>,
    pub choices: VecDeque<u32>,
}

impl Recorded {
    pub fn new(uniforms: Vec<f64>, choices: Vec<u32>) -> Recorded {
        Recorded {
            uniforms: uniforms.into(),
            choices: choices.into(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.uniforms.is_empty() && self.choices.is_empty()
    }
}

impl Coins for Recorded {
    fn uniform(&mut self) -> f64 {
        self.uniforms.pop_front().expect("recorded coins exhausted")
    }
    fn choice(&mut self, n: u32) -> u32 {
        let c = self.choices.pop_front().expect("recorded coins exhausted");
        assert!(c < n, "recorded choice {c} out of {n}");
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_coins_replay_in_order_and_the_rng_implements_the_trait() {
        let mut r = Recorded::new(vec![0.25, 0.75], vec![1]);
        assert_eq!(r.uniform(), 0.25);
        assert_eq!(r.choice(2), 1);
        assert_eq!(r.uniform(), 0.75);
        assert!(r.is_empty());
        let mut rng = crate::rng::seeded(1);
        let u = Coins::uniform(&mut rng);
        assert!((0.0..1.0).contains(&u));
        assert!(Coins::choice(&mut rng, 3) < 3);
    }

    #[test]
    #[should_panic(expected = "recorded coins exhausted")]
    fn exhausted_recorded_coins_panic() {
        let mut r = Recorded::new(vec![], vec![]);
        r.uniform();
    }
}
