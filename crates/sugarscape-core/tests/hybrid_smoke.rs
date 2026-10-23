//! Every hybrid preset runs 200 ticks at seed 1 with sane series: CI's smoke test for
//! milestone 42 (it scores nothing; the survey scores).
use sugarscape_core::{
    model::{ModelConfig, ModelKind, ModelWorld},
    presets, sweep,
};

#[test]
fn every_preset_runs_two_hundred_ticks_with_sane_series() {
    let mut seen = 0;
    for p in presets::catalog() {
        let ModelConfig::Hybrid(c) = &p.config else {
            continue;
        };
        seen += 1;
        let mut w = ModelWorld::new(p.config.clone(), 1).unwrap();
        w.model_mut().run(200);
        let m = w.model();
        assert_eq!(w.kind(), ModelKind::Hybrid);
        let share = m.latest_value("bot_share").unwrap();
        assert!((0.0..1.0).contains(&share), "{}: bot share {share}", p.id);
        if c.world == sugarscape_core::hybrid::World::Coloring {
            let t = m.latest_value("game_time").unwrap();
            assert!(
                (0.0..=300.0).contains(&t) && m.latest_value("conflicts").unwrap().is_finite(),
                "{}",
                p.id
            );
        } else {
            let coop = m.latest_value("cooperation").unwrap();
            assert!((0.0..=1.0).contains(&coop), "{}: cooperation {coop}", p.id);
            assert_eq!(m.tick(), 200);
        }
    }
    assert_eq!(seen, 8);
}

#[test]
fn the_builtin_sweeps_parse_and_name_real_paths() {
    for id in [
        "hybrid-booker-lattice-alpha",
        "hybrid-guo-phi-networks",
        "hybrid-si-lattice-r",
    ] {
        let s = sweep::builtin(id).unwrap_or_else(|| panic!("no builtin {id}"));
        assert!(!s.points().unwrap().is_empty(), "{id}");
    }
}
