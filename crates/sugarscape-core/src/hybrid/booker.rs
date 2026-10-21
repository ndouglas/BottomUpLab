//! Booker et al. (arXiv 2306.17747v2): the donation game T = b, R = b − c, P = 0, S = −c with
//! Samaritan (always C), discriminatory (mirroring the partner) or malicious (always D) bots. The
//! finite well-mixed chain (Eqs 1–15) and the replicator equations (Eqs 16–20) are exact
//! references; the network dynamics follow in this file's second part.

use super::exact::{exp, ln_factorial, log_sum_exp, logistic, roots, tanh_half, Root};
use crate::portable::ln;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotKind {
    Samaritan,
    Discriminatory,
    Malicious,
}

/// The donation game.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Donation {
    pub b: f64,
    pub c: f64,
}

impl Donation {
    pub fn r(&self) -> f64 {
        self.b - self.c
    }
    pub fn s(&self) -> f64 {
        -self.c
    }
    pub fn t(&self) -> f64 {
        self.b
    }
    pub fn p(&self) -> f64 {
        0.0
    }
    /// The row player's payoff (true = cooperate).
    pub fn payoff(&self, me: bool, other: bool) -> f64 {
        match (me, other) {
            (true, true) => self.r(),
            (true, false) => self.s(),
            (false, true) => self.t(),
            (false, false) => self.p(),
        }
    }
}

/// Booker's finite well-mixed chain: `n` humans, `m` bots of one kind, Fermi intensity `beta`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chain {
    pub n: u32,
    pub m: u32,
    pub game: Donation,
    pub kind: BotKind,
    pub beta: f64,
}

impl Chain {
    /// (d_C, d_D, d_IR) of Eqs 1–5.
    fn d(&self) -> (f64, f64, f64) {
        match self.kind {
            BotKind::Samaritan => (1.0, 0.0, 0.0),
            BotKind::Malicious => (0.0, 1.0, 0.0),
            BotKind::Discriminatory => (0.0, 0.0, 1.0),
        }
    }

    /// Π_C(k), a cooperating human's mean payoff with k cooperating humans (Eqs 1, 4, 5).
    pub fn pi_c(&self, k: u32) -> f64 {
        let (n, m, k) = (f64::from(self.n), f64::from(self.m), f64::from(k));
        let g = &self.game;
        let (dc, dd, di) = self.d();
        ((k - 1.0) * g.r() + (n - k) * g.s() + m * (dc * g.r() + dd * g.s() + di * g.r()))
            / (n + m - 1.0)
    }

    /// Π_D(k).
    pub fn pi_d(&self, k: u32) -> f64 {
        let (n, m, k) = (f64::from(self.n), f64::from(self.m), f64::from(k));
        let g = &self.game;
        let (dc, dd, di) = self.d();
        (k * g.t() + (n - k - 1.0) * g.p() + m * (dc * g.t() + dd * g.p() + di * g.p()))
            / (n + m - 1.0)
    }

    pub fn delta(&self, k: u32) -> f64 {
        self.pi_c(k) - self.pi_d(k)
    }

    /// The logarithms of T⁺(k)'s and T⁻(k)'s sampling factors (Eq 6 without the Fermi terms):
    /// a C role model is a cooperating human or (Samaritan) a bot, a D role model a defecting
    /// human or (malicious) a bot; a discriminatory bot is a null draw.
    fn ln_sampling(&self, k: u32) -> (f64, f64) {
        let (n, m, k) = (f64::from(self.n), f64::from(self.m), f64::from(k));
        let (dc, _, di) = self.d();
        let up = ((n - k) / n) * ((k + (1.0 - di) * dc * m) / (n + m));
        let down = (k / n) * ((n - k + (1.0 - di) * (1.0 - dc) * m) / (n + m));
        (ln(up), ln(down))
    }

    /// T⁺(k), the probability that one more human cooperates after one step (Eq 6).
    pub fn t_plus(&self, k: u32) -> f64 {
        exp(self.ln_sampling(k).0) * logistic(self.beta * self.delta(k))
    }

    /// T⁻(k).
    pub fn t_minus(&self, k: u32) -> f64 {
        exp(self.ln_sampling(k).1) * logistic(-self.beta * self.delta(k))
    }

    /// ln(T⁺(k)/T⁻(k)) = ln(up/down) + βΔf(k): the Fermi terms' ratio is exactly e^{βΔf}.
    fn ln_step(&self, k: u32) -> f64 {
        let (up, down) = self.ln_sampling(k);
        up - down + self.beta * self.delta(k)
    }

    /// ln r with r = ∏_{k=1}^{N−1} T⁺(k)/T⁻(k) = ρ_{D,C}/ρ_{C,D}.
    pub fn ln_ratio(&self) -> f64 {
        (1..self.n).map(|k| self.ln_step(k)).sum()
    }

    /// The stationary weight on all-C, r/(1 + r), Fig. 1's "frequency of cooperation".
    pub fn all_c_weight(&self) -> f64 {
        logistic(self.ln_ratio())
    }

    /// (ρ_{D,C}, ρ_{C,D}) by Eq 7, in log space: ρ_{D,C} = 1/(1 + Σ_i ∏_{j≤i} T⁻/T⁺) and
    /// ρ_{C,D} = ρ_{D,C}/r.
    pub fn fixation(&self) -> (f64, f64) {
        let mut terms = vec![0.0];
        let mut acc = 0.0;
        for i in 1..self.n {
            acc -= self.ln_step(i);
            terms.push(acc);
        }
        let ln_dc = -log_sum_exp(&terms);
        (exp(ln_dc), exp(ln_dc - self.ln_ratio()))
    }

    /// F (Eqs 9–11) in closed form.
    pub fn f_closed(&self) -> f64 {
        let (n, m) = (f64::from(self.n), f64::from(self.m));
        let g = &self.game;
        let x = match self.kind {
            BotKind::Samaritan => g.r() - g.t(),
            BotKind::Malicious => g.s() - g.p(),
            BotKind::Discriminatory => g.r() - g.p(),
        };
        (n - 1.0) / (n + m - 1.0)
            * ((g.p() - g.r())
                + n * (g.s() - g.p())
                + m * x
                + (g.r() + g.p() - g.t() - g.s()) * n / 2.0)
    }

    /// F = Σ_{k=1}^{N−1} Δf(k), summed directly.
    pub fn f_direct(&self) -> f64 {
        (1..self.n).map(|k| self.delta(k)).sum()
    }

    /// ln G, G the product of the sampling ratios, directly.
    pub fn ln_g_direct(&self) -> f64 {
        (1..self.n)
            .map(|k| {
                let (up, down) = self.ln_sampling(k);
                up - down
            })
            .sum()
    }

    /// ln G from Eqs 13 and 15 and the corrected Eq 14, G_D = (N − 1)! M! / (N + M − 1)!.
    pub fn ln_g_closed(&self) -> f64 {
        let (n1, m) = (self.n - 1, self.m);
        match self.kind {
            BotKind::Samaritan => ln_factorial(n1 + m) - ln_factorial(n1) - ln_factorial(m),
            BotKind::Malicious => ln_factorial(n1) + ln_factorial(m) - ln_factorial(n1 + m),
            BotKind::Discriminatory => 0.0,
        }
    }

    /// ln of Eq 14 as printed: (N − 1)! (M + 1)! / (N + M − 1)!.
    pub fn ln_g_d_printed(&self) -> f64 {
        ln_factorial(self.n - 1) + ln_factorial(self.m + 1) - ln_factorial(self.n - 1 + self.m)
    }
}

/// Booker's infinite-population equations: x the cooperating share of humans, α the bots' share
/// of everyone, β_H and β_AI the intensities of copying humans and bots.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Replicator {
    pub game: Donation,
    pub alpha: f64,
    pub beta_h: f64,
    pub beta_ai: f64,
    pub kind: BotKind,
}

impl Replicator {
    /// Δf(x) for Eq 18 (Samaritan) or Eq 20 (discriminatory); None for malicious bots, for which
    /// the paper gives no equation.
    pub fn delta(&self, x: f64) -> Option<f64> {
        let (g, a) = (&self.game, self.alpha);
        match self.kind {
            BotKind::Samaritan => Some(
                (g.r() - g.t()) * ((1.0 - a) * x + a) + (g.s() - g.p()) * (1.0 - a) * (1.0 - x),
            ),
            BotKind::Discriminatory => Some(
                (g.r() - g.t()) * (1.0 - a) * x
                    + (g.s() - g.p()) * (1.0 - a) * (1.0 - x)
                    + a * (g.r() - g.p()),
            ),
            BotKind::Malicious => None,
        }
    }

    /// ẋ: Eq 18, (1 − x)[x(1 − α) tanh(β_H Δf/2) + α/(1 + e^{−β_AI Δf})], or Eq 20,
    /// (1 − x) x (1 − α) tanh(β_H Δf/2).
    pub fn rate(&self, x: f64) -> Option<f64> {
        let d = self.delta(x)?;
        let a = self.alpha;
        Some(match self.kind {
            BotKind::Samaritan => {
                (1.0 - x)
                    * (x * (1.0 - a) * tanh_half(self.beta_h * d) + a * logistic(self.beta_ai * d))
            }
            _ => (1.0 - x) * x * (1.0 - a) * tanh_half(self.beta_h * d),
        })
    }

    /// The fixed points on [0, 1], 10,000 cells.
    pub fn fixed_points(&self) -> Option<Vec<Root>> {
        self.delta(0.0)?;
        Some(roots(|x| self.rate(x).unwrap(), 0.0, 1.0, 10_000))
    }

    /// The largest stable fixed point.
    pub fn top_stable(&self) -> Option<f64> {
        Some(
            self.fixed_points()?
                .iter()
                .filter(|r| r.stable)
                .map(|r| r.x)
                .fold(f64::NAN, f64::max),
        )
    }
}

/// The α below which Eq 18 has an interior root (h(1) < 0), by the paper's general formula:
/// 1 / (1 + 1 / [(1 + e^{β_AI(T − R)}) tanh(β_H(T − R)/2)]).
pub fn alpha_c_general(beta_h: f64, beta_ai: f64, t_minus_r: f64) -> f64 {
    1.0 / (1.0 + 1.0 / ((1.0 + exp(beta_ai * t_minus_r)) * tanh_half(beta_h * t_minus_r)))
}

/// The paper's simplification for β_H = β_AI = β: 1/(1 + e^{β(T − R)}).
pub fn alpha_c_printed(beta: f64, t_minus_r: f64) -> f64 {
    logistic(-beta * t_minus_r)
}

/// What the general formula reduces to for β_H = β_AI = β, since (1 + eᵃ) tanh(a/2) = eᵃ − 1:
/// 1 − e^{−β(T − R)}.
pub fn alpha_c_corrected(beta: f64, t_minus_r: f64) -> f64 {
    1.0 - exp(-beta * t_minus_r)
}

/// α_c by a root scan of h(1; α) = (1 − α) tanh(β_H Δ/2) + α/(1 + e^{−β_AI Δ}) with Δ = R − T.
pub fn alpha_c_scan(game: Donation, beta_h: f64, beta_ai: f64) -> f64 {
    let d = game.r() - game.t();
    let h = |a: f64| (1.0 - a) * tanh_half(beta_h * d) + a * logistic(beta_ai * d);
    roots(h, 0.0, 1.0, 10_000)[0].x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portable::ln;

    const GAME: Donation = Donation { b: 2.0, c: 1.0 };
    const KINDS: [BotKind; 3] = [
        BotKind::Samaritan,
        BotKind::Discriminatory,
        BotKind::Malicious,
    ];

    fn chain(m: u32, kind: BotKind, beta: f64) -> Chain {
        Chain {
            n: 100,
            m,
            game: GAME,
            kind,
            beta,
        }
    }

    #[test]
    fn the_donation_game_has_the_papers_entries() {
        assert_eq!(
            (GAME.r(), GAME.s(), GAME.t(), GAME.p()),
            (1.0, -1.0, 2.0, 0.0)
        );
        assert_eq!(
            (GAME.payoff(true, true), GAME.payoff(true, false)),
            (1.0, -1.0)
        );
        assert_eq!(
            (GAME.payoff(false, true), GAME.payoff(false, false)),
            (2.0, 0.0)
        );
    }

    #[test]
    fn closed_forms_match_direct_sums() {
        for kind in KINDS {
            for m in [0, 1, 3, 50, 100] {
                for b in [1.1, 2.0, 3.0] {
                    let c = Chain {
                        n: 100,
                        m,
                        game: Donation { b, c: 1.0 },
                        kind,
                        beta: 1.0,
                    };
                    let (fc, fd) = (c.f_closed(), c.f_direct());
                    assert!(
                        (fc - fd).abs() < 1e-9 * fd.abs().max(1.0),
                        "F {kind:?} M {m} b {b}: {fc} vs {fd}"
                    );
                    let (gc, gd) = (c.ln_g_closed(), c.ln_g_direct());
                    assert!(
                        (gc - gd).abs() < 1e-9 * gd.abs().max(1.0),
                        "ln G {kind:?} M {m}: {gc} vs {gd}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_printed_g_d_is_off_by_a_factor_m_plus_one() {
        // N = 10, M = 3: (N − 1)! M! / (N + M − 1)! = 9! 3! / 12! = 1/220; Eq 14 prints 9! 4! / 12!.
        let c = Chain {
            n: 10,
            m: 3,
            game: GAME,
            kind: BotKind::Malicious,
            beta: 1.0,
        };
        assert!((exp(c.ln_g_direct()) - 1.0 / 220.0).abs() < 1e-15);
        assert!((c.ln_g_d_printed() - c.ln_g_direct() - ln(4.0)).abs() < 1e-12);
    }

    #[test]
    fn transition_rates_follow_eq_6() {
        let c = chain(10, BotKind::Samaritan, 1.0);
        let k = 30;
        // T⁺ = ((N − k)/N)((k + M)/(N + M)) / (1 + e^{−βΔ}); T⁻ = (k/N)((N − k)/(N + M)) / (1 + e^{βΔ}).
        let d = c.delta(k);
        let up = (70.0 / 100.0) * (40.0 / 110.0) * logistic(d);
        let down = (30.0 / 100.0) * (70.0 / 110.0) * logistic(-d);
        assert!((c.t_plus(k) - up).abs() < 1e-15 && (c.t_minus(k) - down).abs() < 1e-15);
        let ir = chain(10, BotKind::Discriminatory, 1.0);
        assert!(
            (ir.t_plus(k) - (70.0 / 100.0) * (30.0 / 110.0) * logistic(ir.delta(k))).abs() < 1e-15,
            "bots are null draws"
        );
        let bad = chain(10, BotKind::Malicious, 1.0);
        assert!(
            (bad.t_minus(k) - (30.0 / 100.0) * (80.0 / 110.0) * logistic(-bad.delta(k))).abs()
                < 1e-15,
            "bots are D role models"
        );
    }

    #[test]
    fn the_stationary_weight_agrees_with_the_fixation_probabilities() {
        for (m, kind, beta) in [
            (5, BotKind::Samaritan, 0.1),
            (57, BotKind::Samaritan, 1.0),
            (80, BotKind::Discriminatory, 2.5),
            (20, BotKind::Malicious, 0.5),
        ] {
            let c = chain(m, kind, beta);
            let (dc, cd) = c.fixation();
            assert!(
                (dc / (dc + cd) - c.all_c_weight()).abs() < 1e-12,
                "{kind:?} {m}"
            );
            assert!((0.0..=1.0).contains(&dc) && (0.0..=1.0).contains(&cd));
            let expect = c.ln_g_direct() + beta * c.f_direct();
            assert!(
                (c.ln_ratio() - expect).abs() < 1e-9 * expect.abs().max(1.0),
                "ln r = ln G + βF"
            );
        }
        assert_eq!(
            chain(0, BotKind::Samaritan, 1.0).all_c_weight(),
            chain(0, BotKind::Discriminatory, 1.0).all_c_weight(),
            "no bots, no difference"
        );
    }

    #[test]
    fn the_chain_is_finite_at_strong_selection() {
        for beta in [10.0, 100.0] {
            for m in [0, 1, 100] {
                for kind in KINDS {
                    let c = chain(m, kind, beta);
                    let (dc, cd) = c.fixation();
                    assert!(
                        c.all_c_weight().is_finite() && dc.is_finite() && cd.is_finite(),
                        "{kind:?} M {m} β {beta}"
                    );
                }
            }
        }
    }

    #[test]
    fn eq_18_has_a_constant_payoff_gap_and_never_fixes_zero() {
        for b in [1.1, 2.0, 3.0] {
            let r = Replicator {
                game: Donation { b, c: 1.0 },
                alpha: 0.1,
                beta_h: 0.1,
                beta_ai: 2.0,
                kind: BotKind::Samaritan,
            };
            for x in [0.0, 0.3, 1.0] {
                assert!(
                    (r.delta(x).unwrap() + 1.0).abs() < 1e-15,
                    "Δf = −c at x {x}, b {b}"
                );
            }
            assert!(r.rate(0.0).unwrap() > 0.0);
        }
        let bad = Replicator {
            game: GAME,
            alpha: 0.1,
            beta_h: 1.0,
            beta_ai: 1.0,
            kind: BotKind::Malicious,
        };
        assert_eq!(
            (bad.rate(0.5), bad.fixed_points()),
            (None, None),
            "the paper has no malicious equation"
        );
    }

    #[test]
    fn the_interior_root_of_eq_18_is_where_h_vanishes() {
        let r = Replicator {
            game: GAME,
            alpha: 0.1,
            beta_h: 1.0,
            beta_ai: 1.0,
            kind: BotKind::Samaritan,
        };
        // h(x) = x(1 − α) tanh(−βc/2) + α logistic(−βc) is linear in x.
        let want = 0.1 * logistic(-1.0) / (0.9 * tanh_half(1.0));
        let pts = r.fixed_points().unwrap();
        assert!(
            pts.iter().any(|p| (p.x - want).abs() < 1e-12 && p.stable),
            "{pts:?}"
        );
        assert!(pts.iter().any(|p| p.x == 1.0 && !p.stable));
        assert!((r.top_stable().unwrap() - want).abs() < 1e-12);
    }

    #[test]
    fn eq_20_fixes_both_ends() {
        let r = Replicator {
            game: Donation { b: 3.0, c: 1.0 },
            alpha: 0.5,
            beta_h: 0.5,
            beta_ai: 1.0,
            kind: BotKind::Discriminatory,
        };
        assert_eq!((r.rate(0.0), r.rate(1.0)), (Some(0.0), Some(0.0)));
        // Δf = −c(1 − α) + α(b − c) = 0.5 > 0: cooperation takes over.
        assert!((r.delta(0.4).unwrap() - 0.5).abs() < 1e-15);
        assert_eq!(r.top_stable(), Some(1.0));
    }

    #[test]
    fn alpha_c_reduces_to_one_minus_e_to_the_minus_beta_c() {
        for beta in [0.1, 1.0, 2.5] {
            let general = alpha_c_general(beta, beta, 1.0);
            assert!(
                (general - alpha_c_corrected(beta, 1.0)).abs() < 1e-12,
                "β {beta}"
            );
            assert!(
                (alpha_c_scan(GAME, beta, beta) - general).abs() < 1e-9,
                "β {beta}"
            );
        }
        assert!((alpha_c_corrected(1.0, 1.0) - 0.632_120_558_828_557_7).abs() < 1e-15);
        assert!((alpha_c_printed(1.0, 1.0) - 0.268_941_421_369_995_1).abs() < 1e-15);
    }
}
