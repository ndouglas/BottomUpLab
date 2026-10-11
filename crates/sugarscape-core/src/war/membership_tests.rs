//! Behavioral comparison against the frozen pre-optimization HashSet engine.
use super::{config::Geometry, engine::Engagement, engine_tests::config, frozen_engine};

// Catches changed draw order, side offsets, retention, and scratch leaking across steps/restores.
#[test]
fn seeded_steps_and_restores_match_frozen_hash_engine() {
    for geometry in [Geometry::AimedFire, Geometry::DuelContact] {
        for (blue, red) in [
            (0, 0),
            (0, 3),
            (3, 0),
            (1, 1),
            (2, 5),
            (5, 2),
            (17, 17),
            (4096, 4096),
            (4096, 1),
            (1, 4096),
        ] {
            for (blue_rate, red_rate) in [
                (1.0, 1.0),
                (0.2, 1.0),
                (1.0, 0.2),
                (0.0, 1.0),
                (1.0, 0.0),
                (0.0, 0.0),
                (1e-18, 1.0),
                (f64::from_bits(1), 1.0),
            ] {
                for seed in [0, 7, 91, u64::MAX] {
                    let mut c = config(geometry);
                    c.blue = blue;
                    c.red = red;
                    c.blue_rate = blue_rate;
                    c.red_rate = red_rate;
                    c.max_steps = 6;
                    let context = format!("{c:?}; seed={seed}");
                    let mut actual = Engagement::new(c.clone(), seed).unwrap();
                    let mut expected = frozen_engine::Engagement::new(c.clone(), seed).unwrap();
                    assert_eq!(actual.ending(), expected.ending(), "{context}");
                    for step in 0..=c.max_steps {
                        let before = actual.checkpoint();
                        let a = actual.step();
                        let b = expected.step();
                        assert_eq!(a, b, "{context}; iteration={step}");
                        assert_eq!(
                            actual.checkpoint(),
                            expected.checkpoint(),
                            "{context}; iteration={step}"
                        );
                        if a.is_err() {
                            assert_eq!(actual.checkpoint(), before, "{context}");
                        }
                        // Round-trip every candidate state, including sparse/terminal/failure states.
                        let saved = serde_json::from_slice(
                            &serde_json::to_vec(&actual.checkpoint()).unwrap(),
                        )
                        .unwrap();
                        actual = Engagement::new(c.clone(), seed).unwrap();
                        expected = frozen_engine::Engagement::new(c.clone(), seed).unwrap();
                        actual.restore(&saved).unwrap();
                        expected.restore(&saved).unwrap();
                        assert_eq!(actual.checkpoint(), expected.checkpoint(), "{context}");
                        if !matches!(a, Ok(Some(_))) {
                            break;
                        }
                    }
                }
            }
        }
    }
}

// Catches unchecked supplied IDs, missing duplicate checks, and changed diagnostics/rollback.
#[test]
fn supplied_matching_failures_match_frozen_hash_engine_on_sparse_populations() {
    let mut c = config(Geometry::DuelContact);
    c.blue = 4;
    c.red = 3;
    c.max_steps = 4;
    let mut actual = Engagement::new(c.clone(), 7).unwrap();
    let mut expected = frozen_engine::Engagement::new(c, 7).unwrap();
    let pairs = [(3, 4), (0, 6), (2, 5)];
    let words = [0, u64::MAX, u64::MAX, u64::MAX, 0, u64::MAX];
    assert_eq!(
        actual.step_supplied(&pairs, &words),
        expected.step_supplied(&pairs, &words)
    );
    let saved = actual.checkpoint();
    assert_eq!(
        (&saved.blue_alive, &saved.red_alive),
        (&vec![1, 2, 3], &vec![4, 6])
    );
    for pairs in [
        vec![(1, 4)],
        vec![(1, 4), (1, 6)],
        vec![(1, 4), (2, 4)],
        vec![(0, 4), (2, 6)],
        vec![(1, 5), (2, 6)],
        vec![(4, 1), (2, 6)],
        vec![(u32::MAX, 4), (2, 6)],
        vec![(1, u32::MAX), (2, 6)],
        vec![(1, 4), (4, 6)],
        vec![(1, 4), (2, 7)],
    ] {
        let a = actual.step_supplied(&pairs, &[0; 4]);
        assert!(a.is_err(), "{pairs:?}");
        assert_eq!(a, expected.step_supplied(&pairs, &[0; 4]), "{pairs:?}");
        assert_eq!(actual.checkpoint(), saved);
        assert_eq!(expected.checkpoint(), saved);
    }
    // Matching order differs from canonical population order, with unmatched blue ID 2 retained.
    let pairs = [(3, 6), (1, 4)];
    let words = [0, u64::MAX, u64::MAX, 0];
    let a = actual.step_supplied(&pairs, &words).unwrap().unwrap();
    assert_eq!(
        Some(a.clone()),
        expected.step_supplied(&pairs, &words).unwrap()
    );
    assert_eq!(
        a.casualties.iter().map(|c| c.id).collect::<Vec<_>>(),
        [1, 6]
    );
    assert_eq!(actual.checkpoint().blue_alive, [2, 3]);
    assert_eq!(actual.checkpoint().red_alive, [4]);
}
