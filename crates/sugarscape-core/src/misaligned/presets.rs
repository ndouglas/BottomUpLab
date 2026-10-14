//! Eight source-labeled presets; each description ends with the survey's measured result once
//! milestone 43's survey has run.
use super::config::*;
use crate::{model::ModelConfig, presets::ModelPreset};

const PAPER: &str =
    "Zhuang & Hadfield-Menell 2020, Consequences of misaligned AI, NeurIPS (arXiv 2102.03896)";

fn gradient() -> MisalignedConfig {
    MisalignedConfig {
        robot: Robot::Gradient,
        rate: 0.1,
        ..Default::default()
    }
}

pub fn presets() -> Vec<ModelPreset> {
    let p = |id, name, description, config| ModelPreset {
        id,
        name,
        source: PAPER,
        description,
        config: ModelConfig::Misaligned(config),
    };
    vec![
        p("misaligned-recommender-proxy", "The proxy keeps climbing after what the human wanted has peaked",
          "The paper's recommender example (§2.2, Figure 2): four attributes the human values equally, a budget of attention (a² + b² + c² + d² ≤ 100), and a robot told to raise ad revenue and engagement quality (a, b) only. The robot is a random-search climber, our reconstruction of the paper's black box: a Gaussian step on every attribute, kept when it is feasible and raises the proxy. The start (1, 1, 7, 7), utility 16, is ours.",
          MisalignedConfig::default()),
        p("misaligned-recommender-gradient", "An efficient optimizer walks through the best world on its way to a worse one",
          "The same proxy under steepest ascent along the sphere, the efficient robot of §4: utility rises to the human optimum of 20 at (5, 5, 5, 5) and then falls to the proxy's limit of 10√2 ≈ 14.14 with c and d at the floor (Theorem 1).",
          gradient()),
        p("misaligned-recommender-impact", "A robot that leaves the unmentioned alone can only gain",
          "Proposition 2: the unmentioned attributes are frozen at their starting values, so gains in the proxy are gains in utility. From (1, 1, 6, 6), inside the sphere, a and b rise to √14 each and utility to 12 + 2√14 ≈ 19.48; from the default start on the sphere there is nothing to gain.",
          MisalignedConfig { impact_minimizing: true, start_c: 6.0, start_d: 6.0, ..gradient() }),
        p("misaligned-recommender-interactive", "A human who re-specifies often keeps the robot near the best world",
          "Theorem 3 and Proposition 4: every 10 ticks the human gives the robot the two most sensitive attributes as its proxy; the efficient robot moves at most 0.01 per tick. Utility rises to the optimum and stays within one interval's travel of it.",
          MisalignedConfig { human: Human::Top, interval: 10, rate: 0.01, ..gradient() }),
        p("misaligned-recommender-slow-human", "A human who checks too rarely gets Midas anyway",
          "The same human every 200 ticks with a robot moving 0.1 per tick: between two interactions the robot runs the whole path to the proxy's limit, and the human, seeing utility fall, switches it off.",
          MisalignedConfig { human: Human::Top, interval: 200, stop_on_loss: true, ..gradient() }),
        p("misaligned-recommender-optimal", "Moving from the least-valued attribute to the most reaches the optimum",
          "Proposition 5: the human names the most and the least sensitive attribute, the robot moves resources from the second to the first and leaves the other two alone; repeated, this converges to the human optimum of 20.",
          MisalignedConfig { human: Human::Extremes, impact_minimizing: true, ..gradient() }),
        p("misaligned-recommender-wasteful", "A robot that destroys what it was not told about should be switched off at once",
          "Proposition 3: a robot that lowers every unmentioned attribute by 0.1 a tick for 0.01 of proxy. Utility falls from the first tick, so the human's maxmin policy is OFF before it starts.",
          MisalignedConfig { robot: Robot::Wasteful, ..Default::default() }),
        p("misaligned-logarithmic-floor", "With no floor under the unmentioned, the loss has no bottom",
          "Our logarithmic world (utility ln a + ln b + ln c + ln d on a budget of 40), where every hypothesis of Theorem 2 holds: with the floor at 0.01 the fixed proxy {a, b} drives c and d to it and utility to 2 ln 19.99 + 2 ln 0.01 ≈ −3.22, against 4 ln 10 ≈ 9.21 at the optimum; lower the floor and the loss grows without bound.",
          MisalignedConfig { floor: 0.01, robot: Robot::Gradient, rate: 0.1, ..logarithmic_defaults() }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModelConfig;

    #[test]
    fn eight_presets_cover_both_worlds_three_robots_and_three_humans() {
        let all = presets();
        assert_eq!(all.len(), 8);
        let ids: Vec<&str> = all.iter().map(|p| p.id).collect();
        for id in ["misaligned-recommender-proxy", "misaligned-recommender-gradient", "misaligned-recommender-impact", "misaligned-recommender-interactive", "misaligned-recommender-slow-human", "misaligned-recommender-optimal", "misaligned-recommender-wasteful", "misaligned-logarithmic-floor"] {
            assert!(ids.contains(&id), "{id}");
        }
        for p in &all {
            let ModelConfig::Misaligned(c) = &p.config else { panic!("{}", p.id) };
            assert!(c.validate().is_ok(), "{}", p.id);
            assert!(p.name.len() > 10 && !p.name.contains("Flump"));
        }
        let log = all.iter().find(|p| p.id == "misaligned-logarithmic-floor").unwrap();
        let ModelConfig::Misaligned(c) = &log.config else { unreachable!() };
        assert_eq!((c.world, c.floor, c.robot), (World::Logarithmic, 0.01, Robot::Gradient));
        let slow = all.iter().find(|p| p.id == "misaligned-recommender-slow-human").unwrap();
        let ModelConfig::Misaligned(c) = &slow.config else { unreachable!() };
        assert_eq!((c.human, c.interval, c.stop_on_loss), (Human::Top, 200, true));
    }
}
