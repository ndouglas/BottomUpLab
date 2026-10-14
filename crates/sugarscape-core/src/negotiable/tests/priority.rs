use super::{
    history_weights, r, update_weights, Error, HistoryResult, Observation, Ratio, WireRatio,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn zero_mass_is_not_reset_to_an_equal_prior() {
    assert_eq!(
        update_weights([r(1, 1), r(0, 1)], [r(0, 1), r(1, 5)]),
        Err(Error::ImpossibleObservation)
    );
}

#[test]
fn red_updates_equal_prior_to_nine_tenths() {
    assert_eq!(
        update_weights([r(1, 2); 2], [r(9, 10), r(1, 10)]),
        Ok([r(9, 10), r(1, 10)])
    );
}

#[test]
fn green_updates_equal_prior_to_one_tenth() {
    assert_eq!(
        history_weights([r(1, 2); 2], [r(9, 10), r(1, 10)], &[Observation::Green]),
        Ok(HistoryResult::Supported([r(1, 10), r(9, 10)]))
    );
}

#[test]
fn common_beliefs_preserve_a_nonuniform_prior() {
    assert_eq!(
        history_weights(
            [r(1, 3), r(2, 3)],
            [r(1, 2); 2],
            &[Observation::Red, Observation::Green, Observation::Red]
        ),
        Ok(HistoryResult::Supported([r(1, 3), r(2, 3)]))
    );
}

#[test]
fn zero_likelihood_eliminates_a_model() {
    assert_eq!(
        history_weights([r(1, 2); 2], [r(1, 1), r(4, 5)], &[Observation::Green]),
        Ok(HistoryResult::Supported([r(0, 1), r(1, 1)]))
    );
}

#[test]
fn a_zero_prior_does_not_revive() {
    assert_eq!(
        history_weights(
            [r(0, 1), r(1, 1)],
            [r(9, 10), r(1, 10)],
            &[Observation::Red; 8]
        ),
        Ok(HistoryResult::Supported([r(0, 1), r(1, 1)]))
    );
}

#[test]
fn impossible_prefix_is_retained_through_later_supported_suffixes() {
    assert_eq!(
        history_weights(
            [r(1, 1), r(0, 1)],
            [r(1, 1), r(4, 5)],
            &[Observation::Red, Observation::Green, Observation::Red]
        ),
        Ok(HistoryResult::Impossible {
            first_impossible_prefix: 2
        })
    );
}

#[test]
fn eliminated_support_can_make_a_later_observation_impossible() {
    assert_eq!(
        history_weights(
            [r(1, 2); 2],
            [r(1, 1), r(0, 1)],
            &[Observation::Green, Observation::Red, Observation::Green]
        ),
        Ok(HistoryResult::Impossible {
            first_impossible_prefix: 2
        })
    );
}

#[test]
fn empty_history_preserves_validated_prior() {
    assert_eq!(
        history_weights([r(1, 3), r(2, 3)], [r(9, 10), r(1, 10)], &[]),
        Ok(HistoryResult::Supported([r(1, 3), r(2, 3)]))
    );
}

#[test]
fn invalid_weights_are_rejected_even_for_empty_histories() {
    for prior in [
        [r(-1, 2), r(3, 2)],
        [r(0, 1); 2],
        [r(1, 1); 2],
        [r(1, 3); 2],
    ] {
        assert_eq!(
            history_weights(prior, [r(1, 2); 2], &[]),
            Err(Error::InvalidWeights)
        );
        assert_eq!(
            update_weights(prior, [r(1, 2); 2]),
            Err(Error::InvalidWeights)
        );
    }
}

#[test]
fn invalid_probabilities_are_rejected_even_for_empty_histories() {
    for red in [[r(-1, 10), r(1, 2)], [r(1, 2), r(11, 10)]] {
        assert_eq!(
            history_weights([r(1, 2); 2], red, &[]),
            Err(Error::InvalidProbability)
        );
        assert_eq!(
            update_weights([r(1, 2); 2], red),
            Err(Error::InvalidProbability)
        );
    }
}

#[test]
fn history_ninth_step_is_rejected_before_iteration() {
    assert_eq!(
        history_weights(
            [r(1, 1), r(0, 1)],
            [r(1, 1), r(0, 1)],
            &[Observation::Green; 9]
        ),
        Err(Error::HistoryTooLong)
    );
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema: String,
    settings: Settings,
    rows: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    reference_kind: String,
    source_hashes: SourceHashes,
    actions: Vec<Action>,
    cake_model_id: usize,
    policies: Vec<Policy>,
    weights: Vec<WireRatio>,
    likelihood_models: Vec<Model>,
    priors: Vec<[WireRatio; 2]>,
    observations: Vec<String>,
    history_horizon: usize,
    pareto_definitions: ParetoDefinitions,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceHashes {
    main_pdf: String,
    supplement_pdf: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Action {
    id: usize,
    name: String,
    utilities: [WireRatio; 2],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    id: usize,
    red_action: usize,
    green_action: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Model {
    id: usize,
    red_likelihoods: [WireRatio; 2],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParetoDefinitions {
    paper_weak: String,
    strong: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    model_id: usize,
    prior_index: usize,
    history: Vec<u8>,
    result: ResultRow,
}
#[derive(Deserialize)]
#[serde(tag = "status", deny_unknown_fields)]
enum ResultRow {
    #[serde(rename = "supported")]
    Supported { weights: [WireRatio; 2] },
    #[serde(rename = "impossible")]
    Impossible { first_impossible_prefix: usize },
}

fn pair(wire: [WireRatio; 2]) -> [Ratio; 2] {
    wire.map(|x| Ratio::from_wire(x).unwrap())
}

fn validate_settings(s: Settings) -> (BTreeMap<usize, [Ratio; 2]>, Vec<[Ratio; 2]>) {
    assert_eq!(s.reference_kind, "priority");
    assert_eq!(
        s.source_hashes.main_pdf,
        "8eaba6509a3fc39a08905ccadac6ad2cd69c308d9b8cb404248fb6a944a60492"
    );
    assert_eq!(
        s.source_hashes.supplement_pdf,
        "643655853ddf456513aa430094449db81bb939447ac1ad63bf5b864826b5c140"
    );
    assert_eq!(s.cake_model_id, 0);
    assert_eq!(s.history_horizon, 8);
    assert_eq!(s.observations, ["red", "green"]);
    assert_eq!(
        s.pareto_definitions.paper_weak,
        "dominated iff a hull point strictly improves both coordinates"
    );
    assert_eq!(
        s.pareto_definitions.strong,
        "dominated iff a hull point weakly improves both and strictly improves at least one"
    );
    assert_eq!(s.actions.len(), 3);
    for (a, (id, name, utilities)) in s.actions.into_iter().zip([
        (0, "alice_only", [r(30, 1), r(0, 1)]),
        (1, "equal_split", [r(20, 1); 2]),
        (2, "bob_only", [r(0, 1), r(30, 1)]),
    ]) {
        assert_eq!(
            (a.id, a.name.as_str(), pair(a.utilities)),
            (id, name, utilities)
        );
    }
    assert_eq!(s.policies.len(), 9);
    for (id, p) in s.policies.into_iter().enumerate() {
        assert_eq!((p.id, p.red_action, p.green_action), (id, id / 3, id % 3));
    }
    let expected_weights = [
        (0, 1),
        (1, 20),
        (1, 19),
        (1, 10),
        (3, 20),
        (2, 11),
        (1, 5),
        (1, 4),
        (3, 10),
        (7, 20),
        (2, 5),
        (9, 20),
        (1, 2),
        (11, 20),
        (3, 5),
        (13, 20),
        (7, 10),
        (3, 4),
        (4, 5),
        (9, 11),
        (17, 20),
        (9, 10),
        (18, 19),
        (19, 20),
        (1, 1),
    ]
    .map(|(n, d)| r(n, d));
    assert_eq!(
        s.weights
            .into_iter()
            .map(|w| Ratio::from_wire(w).unwrap())
            .collect::<Vec<_>>(),
        expected_weights
    );
    assert_eq!(s.likelihood_models.len(), 3);
    let expected_models = [[r(9, 10), r(1, 10)], [r(1, 2); 2], [r(1, 1), r(4, 5)]];
    let mut models = BTreeMap::new();
    for (id, model) in s.likelihood_models.into_iter().enumerate() {
        assert_eq!(model.id, id);
        let likelihoods = pair(model.red_likelihoods);
        assert_eq!(likelihoods, expected_models[id]);
        assert!(models.insert(model.id, likelihoods).is_none());
    }
    let priors = s.priors.into_iter().map(pair).collect::<Vec<_>>();
    assert_eq!(
        priors,
        [[r(0, 1), r(1, 1)], [r(1, 2); 2], [r(1, 1), r(0, 1)]]
    );
    (models, priors)
}

#[test]
fn all_registered_history_cells_match_independent_reference_by_key() {
    let bytes = include_bytes!("fixtures/priority-reference.json");
    assert!(bytes.len() <= 8 * 1024 * 1024);
    let fixture: Fixture = serde_json::from_slice(bytes).unwrap();
    assert_eq!(fixture.schema, "negotiable-foundation-v1");
    let (models, priors) = validate_settings(fixture.settings);
    assert_eq!(fixture.rows.len(), 4599);
    let mut keys = BTreeSet::new();
    let mut previous = None;
    let mut expected = BTreeMap::new();
    for row in fixture.rows {
        assert!(models.contains_key(&row.model_id));
        assert!(row.prior_index < priors.len());
        assert!(row.history.len() <= 8);
        assert!(row.history.iter().all(|&x| x <= 1));
        let key = (
            row.model_id,
            row.prior_index,
            row.history.len(),
            row.history.clone(),
        );
        if let Some(prev) = previous {
            assert!(prev < key);
        }
        previous = Some(key.clone());
        assert!(keys.insert(key.clone()), "duplicate fixture key: {key:?}");
        let result = match row.result {
            ResultRow::Supported { weights } => {
                let weights = pair(weights);
                assert!(weights.iter().all(|x| x.num() >= 0));
                assert_eq!(weights[0].checked_add(weights[1]), Ok(r(1, 1)));
                HistoryResult::Supported(weights)
            }
            ResultRow::Impossible {
                first_impossible_prefix,
            } => {
                assert!((1..=row.history.len()).contains(&first_impossible_prefix));
                HistoryResult::Impossible {
                    first_impossible_prefix,
                }
            }
        };
        expected.insert(key, result);
    }
    // Enumerate the declared domain independently of the fixture's array position.
    let mut compared = 0;
    for (&model_id, &red) in &models {
        for (prior_index, &prior) in priors.iter().enumerate() {
            for length in 0..=8 {
                for code in 0..(1usize << length) {
                    let history = (0..length)
                        .rev()
                        .map(|shift| ((code >> shift) & 1) as u8)
                        .collect::<Vec<_>>();
                    let key = (model_id, prior_index, length, history.clone());
                    let observations = history
                        .iter()
                        .map(|&x| {
                            if x == 0 {
                                Observation::Red
                            } else {
                                Observation::Green
                            }
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(
                        &history_weights(prior, red, &observations).unwrap(),
                        expected.get(&key).expect("missing registered key"),
                        "key: {key:?}"
                    );
                    compared += 1;
                }
            }
        }
    }
    assert_eq!(compared, 4599);
}

#[test]
fn fixture_dtos_reject_unknown_and_partial_results() {
    for json in [
        r#"{"status":"supported","weights":[{"num":"1","den":"2"},{"num":"1","den":"2"}],"first_impossible_prefix":1}"#,
        r#"{"status":"impossible"}"#,
        r#"{"status":"impossible","first_impossible_prefix":1,"weights":[]}"#,
    ] {
        assert!(serde_json::from_str::<ResultRow>(json).is_err());
    }
}
