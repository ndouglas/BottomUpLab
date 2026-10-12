//! Six source-labeled presets; each description ends with the survey's measured result (milestone 39).
use super::config::*;
use crate::{model::ModelConfig, presets::ModelPreset};

const CIRL: &str = "Hadfield-Menell, Dragan, Abbeel & Russell 2016, NeurIPS, arXiv 1606.03137 §3.3 and appendix §6";
const OFF_SWITCH: &str = "Hadfield-Menell, Dragan, Abbeel & Russell 2017, IJCAI, arXiv 1611.08219";

pub fn presets() -> Vec<ModelPreset> {
    let p = |id, name, source, description, config| ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Assistance(config),
    };
    vec![
        p("assistance-paperclips-expert", "The human demonstrates and the robot copies, and at 0.49 that goes wrong", CIRL,
          "Static: the expert rule (two staples below θ = 1/2, two paperclips above) and R's best reply (0,90), (50,50), (90,0); the posterior after two staples is uniform on [0, 1/2), so R acts as θ = 1/4. At θ = 0.49 the human would rather R made 50 of each (Theorem 3). Native: all four targets hold (thresholds 1/2 and the map exactly, the posterior mean 0.25 within 1e-9, and H's best reply to that map makes (1,1) on [41/92, 51/92], so at 0.49).",
          AssistanceConfig { mode: Mode::Static, ..Default::default() }),
        p("assistance-paperclips-teaching", "The human makes one of each to teach the robot", CIRL,
          "Static: the paper's equilibrium rule, one of each for θ in [41/92, 51/92], with R's best reply. Expected total reward 6373/92 ≈ 69.27 against 69 for the expert pair, computed exactly. Native: the exhaustive search over R's 27 maps returns this pair as the optimum; it is the best of six pure mutual best responses, the other five being relabelings of H's signals worth 69.0250, 68.5312 or 68.2841, and the pooling pair is not one.",
          AssistanceConfig { mode: Mode::Static, human_rule: HumanRule::Equilibrium, ..Default::default() }),
        p("assistance-paperclips-iterated", "Best replies, turn by turn, settle on a teaching rule", CIRL,
          "Iterated best response from the expert pair, one round per tick in the paper's order (H then R, ties to the smaller action, off-path actions resolved by the prior); our dynamic. Series distance counts bins and map entries away from the paper's equilibrium. Native (20 seeds, 200 ticks): from the expert pair all 20 reach the paper's pair (upper median tick 1), as do all 20 from the teaching and pooling starts; from a uniform random rule 12 of 20 do, and the other 8 end at lower-valued relabelings (maps [1,2,0] or [2,0,1] in six, [2,1,0] and [0,2,1] in one each).",
          AssistanceConfig::default()),
        p("assistance-paperclips-learning", "Two learners find an equilibrium, not always the paper's", CIRL,
          "Fictitious play (our rule): both players best-respond to smoothed frequency tables of each other's play; θ is revealed to R after deployment. H starts from a uniform random rule over bins, R from the prior's map (50,50). Common-payoff games are potential games, so play converges; which pair it reaches is the question. Native (20 seeds, 200 ticks): 19 of 20 reach the paper's pair (upper median tick 3) and one ends at the lower-valued relabeling [0,2,1]; started from pooling, all 20 end at the pooling pair (rule all (1,1), (50,50) after every action), because the actions H never makes keep uniform tables.",
          AssistanceConfig { mode: Mode::Learning, human_rule: HumanRule::UniformRandom, ..Default::default() }),
        p("assistance-offswitch-rational", "An uncertain robot lets itself be switched off", OFF_SWITCH,
          "The paper's game with a rational human (β = 0) and belief N(1/4, 1): Δ = 0.2863 > 0 (Theorem 1), so R waits; the frame is Figure 3's surface at μ = 1/4 with the configured point marked. Native: Δ = 0.286345 = σ²E[π̇] 0.386668 − |μ|Pr(C) 0.100323, and R waits; Δ ≥ 0 on all 42 Gaussian priors tested and equals eq. 3 within 3e-16; Corollary 1's point mass and two-point priors hold; with this human Figure 4's value is flat in σ (0.3992–0.3993).",
          AssistanceConfig { game: Game::OffSwitch, mu: 0.25, sigma: 1.0, beta: 0.0, ..Default::default() }),
        p("assistance-offswitch-toddler", "A confident robot facing an erratic human bypasses the switch", OFF_SWITCH,
          "The paper's toddler corner: belief N(1/4, 0.1²) and a Boltzmann human at β = 4. Theorem 2 gives Δ ≈ −0.12 < 0 and E[U_a] > 0, so R executes without asking (Figure 3, right, top left). Native: Δ = −0.120471 = σ²E[π̇] 0.000624 − |μ|Pr(C) 0.121096, and R acts; Theorem 2's decomposition matches the direct Δ within 5e-15 over 80 points, and at μ = 1/4 R acts in every one of the 335 of 420 Figure 3 cells with Δ ≤ 0.",
          AssistanceConfig { game: Game::OffSwitch, mu: 0.25, sigma: 0.1, beta: 4.0, ..Default::default() }),
    ]
}
