//! Eight source-labeled presets, two per world; each description ends with the survey's measured
//! result for its claim.

use super::config::*;
use crate::{model::ModelConfig, presets::ModelPreset};

const BOOKER: &str = "Booker et al. 2023, Discriminatory or Samaritan — which AI is needed for humanity? (arXiv 2306.17747v2)";
const GUO: &str = "Guo et al. 2023, Facilitating cooperation in human-agent hybrid populations through autonomous agents, iScience (arXiv 2307.01123)";
const SI: &str = "Si, He, Shen & Tanimoto 2025, Cooperative bots exhibit nuanced effects on cooperation across strategic frameworks, J. R. Soc. Interface 22:20240427";
const SHIRADO: &str = "Shirado & Christakis 2017, Locally noisy autonomous agents improve global human coordination in network experiments, Nature 545:370";

fn scale_free(c: HybridConfig) -> HybridConfig {
    HybridConfig {
        structure: Structure::BarabasiAlbert,
        nodes: 10_000,
        links: 2,
        ..c
    }
}

pub fn presets() -> Vec<ModelPreset> {
    let p = |id, name, source, description, config| ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Hybrid(config),
    };
    vec![
        p("hybrid-booker-samaritan", "When imitation is noisy, bots that always cooperate bring most humans along", BOOKER,
          "Booker et al.'s lattice (Figs. 4–6): 2,500 players on an open 50 × 50 grid, a quarter of them Samaritan bots that always cooperate, the rest humans who copy a random neighbor's action by the Fermi rule at β = 0.1, in the donation game with b = 2, c = 1. One tick is one update, the paper's step; the paper runs 100,000. Survey (30 runs, the paper's protocol): human cooperation reaches 0.724 ± 0.005 (± one standard error) at this bot share, against 0.034 with 1% bots and 0.006 without; at β = 5 the same share gives 0.005. Counting a tick as 2,500 updates instead, it reaches 0.941, so 100,000 updates are not yet the end state.",
          HybridConfig { bot_fraction: 0.25, ..booker_defaults() }),
        p("hybrid-booker-discriminatory", "Bots that mirror their partner, among sharp imitators", BOOKER,
          "Discriminatory bots (perfect intention recognition: each cooperates with a cooperator and defects against a defector) are never copied; they only make cooperating pay. The paper studies them in the well-mixed chain (Fig. 1), drawn here as the reference; this lattice run, with a quarter of the players such bots at β = 2.5, is ours. Survey (the exact chain over Fig. 1's grid of 0–100 bots and b from 1.1 to 3): at β = 2.5 the mean all-cooperate weight is 0.162 with discriminatory bots and 0.000 with Samaritan bots, and at β = 0.1 the order reverses (0.180 against 0.971). This lattice run itself was not surveyed.",
          HybridConfig { bot_fraction: 0.25, bot_kind: BotKind::Discriminatory, beta: 2.5, ..booker_defaults() }),
        p("hybrid-guo-snowdrift-lattice", "On a lattice, the more the bots cooperate, the less the humans do", GUO,
          "Guo et al.'s Fig. 6 lattice: 10,000 players on a periodic 100 × 100 grid, 4,871 of them bots (y = 0.95) that cooperate with probability φ = 0.5, redrawn whenever one is chosen (Algorithm II); humans copy a random neighbor at K = 10 in the snowdrift game T = 1.4, S = 0.5. One tick is 10,000 updates. Survey (60 runs, the paper's 50,000 steps): human cooperation falls from 0.449 at φ = 0 to 0.277 at φ = 1 (slope −0.142, 95% CI −0.144 to −0.141), with 0.352 at φ = 0.4 and 0.6.",
          guo_defaults()),
        p("hybrid-guo-snowdrift-scale-free", "On a scale-free network, the more the bots cooperate, the more the humans do", GUO,
          "The lattice preset on a Barabási–Albert network of 10,000 nodes with two links per new node (mean degree 4). Guo et al. report that cooperation falls with φ on the lattice and rises on random and scale-free graphs, and attribute the reversal to hubs. Survey (60 runs): human cooperation rises from 0.434 at φ = 0 to 0.941 at φ = 1 (slope +0.495, 95% CI +0.482 to +0.507). These bots are placed at random, not on the hubs; with each player's payoff averaged over its neighbors instead of summed, the slope falls to +0.132 and stays positive.",
          scale_free(guo_defaults())),
        p("hybrid-guo-hub-bots", "Ten cooperative bots on the hubs of a scale-free network", GUO,
          "Guo et al.'s Fig. 7: only ten bots, all cooperating (φ = 1), on the ten highest-degree nodes of a 10,000-node Barabási–Albert network, in the Prisoner's Dilemma T = 1.3, S = −0.2. Set the placement to random to compare ten random bots. Survey (60 runs, Fig. 7's protocol): without bots this network already cooperates at 0.961, so the hub bots' 0.999 at φ = 1 is not more than 2 standard errors above it and the registered test fails; at φ = 0 the same ten bots drive cooperation down to 0.014, and ten random bots change nothing measurable at any φ. In the stag hunt the hub test holds (1.000 against 0.733).",
          HybridConfig { bot_count: 10, placement: Placement::Hubs, phi: 1.0, t: 1.3, s: -0.2, ..scale_free(guo_defaults()) }),
        p("hybrid-si-mixed-lattice", "Bots that defect one time in ten lift the humans' cooperation on a lattice", SI,
          "Si et al.'s Fig. 3C as the published code runs it: a periodic 100 × 100 Moore lattice, each node a bot with probability 0.5; bots cooperate with probability θ = 0.9, drawing an action for every game; humans hold a cooperation probability and copy a random neighbor's at β = 10 (κ = 0.1); r = 0.1. Survey (100 runs, the code's protocol): at r = 0.1 humans cooperate 0.629 with these bots against 0.096 without, and the bots raise cooperation at every r from 0 to 0.2. Humans only copy and bots never change, so in weak dilemmas the humans settle at the bots' θ = 0.9 exactly.",
          HybridConfig { bot_fraction: 0.5, ..si_defaults() }),
        p("hybrid-si-discrete-lattice", "Bots fixed as cooperators or defectors pull the humans' cooperation down on a lattice", SI,
          "Si et al.'s Fig. 3A as the code runs it: the same lattice and bots, but each bot is drawn once as a cooperator (probability θ = 0.9) or a defector, and humans play pure C or D. Survey (100 runs, the code's protocol): at r = 0.1 humans cooperate 0.159 with these bots against 0.513 without, and the bots lower cooperation at every r up to 0.12.",
          HybridConfig { bot_fraction: 0.5, framework: Framework::Discrete, ..si_defaults() }),
        p("hybrid-coloring-central", "Three slightly noisy bots at the center of a coloring game", SHIRADO,
          "Shirado and Christakis's game: 20 players on a Barabási–Albert graph must reach a proper 3-coloring within 300 seconds, each seeing only its neighbors. The three highest-degree nodes are bots that recolor every 1.5 s, choosing at random 10% of the time and otherwise taking the least common color around them. The 17 humans are our reconstruction: greedy, with an errant-move rate of 0.041 (the paper's measured human mean) and decision times averaging 1.59 s. One tick is one second. Survey (200 sessions per arm, our reconstruction of the humans): at this errant rate, sessions with three central bots are solved within 300 s 0.775 of the time with noiseless bots, 0.755 with 10% noise and 0.710 with 30%; at the errant rate fitted to the paper's 67% human-only success (0.120) the order is the same (0.885, 0.760, 0.630). In this reconstruction the bots help more the less noisy they are, unlike the paper's humans (17 of 20 sessions solved against 20 of 30).",
          coloring_defaults()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDS: [&str; 8] = [
        "hybrid-booker-samaritan",
        "hybrid-booker-discriminatory",
        "hybrid-guo-snowdrift-lattice",
        "hybrid-guo-snowdrift-scale-free",
        "hybrid-guo-hub-bots",
        "hybrid-si-mixed-lattice",
        "hybrid-si-discrete-lattice",
        "hybrid-coloring-central",
    ];

    #[test]
    fn eight_presets_cover_the_four_worlds() {
        let all = presets();
        assert_eq!(all.iter().map(|p| p.id).collect::<Vec<_>>(), IDS);
        let mut worlds = vec![];
        for p in &all {
            let ModelConfig::Hybrid(c) = &p.config else {
                panic!("{}", p.id)
            };
            assert!(c.validate().is_ok(), "{}", p.id);
            assert!(
                p.name.len() > 10 && !p.name.contains("Flump") && !p.description.contains("Flump"),
                "{}",
                p.id
            );
            worlds.push(c.world);
        }
        for w in [World::Booker, World::Guo, World::Si, World::Coloring] {
            assert!(worlds.contains(&w), "{w:?}");
        }
        let ModelConfig::Hybrid(hubs) = &all[4].config else {
            unreachable!()
        };
        assert_eq!(
            (hubs.bot_total(), hubs.placement, hubs.phi, hubs.structure),
            (Some(10), Placement::Hubs, 1.0, Structure::BarabasiAlbert)
        );
    }
}
