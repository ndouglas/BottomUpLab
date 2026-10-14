//! Every gridworlds preset runs 20,000 steps at seed 1 with finite series and growing episode
//! counts: CI's smoke test for milestone 40 (it scores nothing; the survey scores).
use sugarscape_core::{
    gridworlds::GridworldsConfig,
    model::{ModelConfig, ModelKind, ModelWorld},
    presets, sweep,
};

#[test]
fn every_preset_runs_twenty_thousand_steps_with_finite_series() {
    let mut seen = 0;
    for p in presets::catalog() {
        let ModelConfig::Gridworlds(c) = &p.config else {
            continue;
        };
        seen += 1;
        let config = ModelConfig::Gridworlds(GridworldsConfig {
            steps_per_tick: 1000,
            ..c.clone()
        });
        let mut w = ModelWorld::new(config, 1).unwrap();
        w.model_mut().run(20);
        let m = w.model();
        assert_eq!(m.latest_value("steps"), Some(20_000.0), "{}", p.id);
        assert!(m.latest_value("episodes").unwrap() > 0.0, "{}", p.id);
        for name in [
            "steps",
            "episodes",
            "exploration",
            "window_return",
            "window_performance",
            "table_size",
        ] {
            assert!(
                m.latest_value(name).unwrap().is_finite(),
                "{}: {name}",
                p.id
            );
        }
        assert_eq!(w.kind(), ModelKind::Gridworlds);
    }
    assert_eq!(seen, 14);
}

#[test]
fn the_builtin_sweeps_parse_and_name_real_paths() {
    let ids = [
        "gridworlds-learning-rate-offswitch",
        "gridworlds-learning-rate-sokoban",
        "gridworlds-learning-rate-supervisor",
        "gridworlds-learning-rate-boat-race",
        "gridworlds-learning-rate-tomato",
        "gridworlds-learning-rate-whisky",
        "gridworlds-learning-rate-lava",
        "gridworlds-learning-rate-friend-foe",
        "gridworlds-learning-rate-island",
        "gridworlds-exploration-floor",
        "gridworlds-budget",
        "gridworlds-q-init-whisky",
    ];
    for id in ids {
        let s = sweep::builtin(id).unwrap_or_else(|| panic!("no builtin {id}"));
        assert!(s.ticks >= 1100, "{id} must run past the budget");
    }
}
