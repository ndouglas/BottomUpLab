use super::{cake::expected_weights, pareto::verify_classification, r};
use crate::negotiable::{
    cake_payoff, cake_rows, classify, CakeRow, Classification, ParetoKind, Policy, Ratio, WireRatio,
};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture<T> {
    schema: String,
    settings: Settings,
    rows: Vec<T>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    reference_kind: String,
    source_hashes: SourceHashes,
    actions: Vec<Action>,
    cake_model_id: usize,
    policies: Vec<PolicySetting>,
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
    id: u8,
    name: String,
    utilities: [WireRatio; 2],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicySetting {
    id: u8,
    red_action: u8,
    green_action: u8,
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
struct ParetoRow {
    policy_id: u8,
    payoff: [WireRatio; 2],
    paper_weak: Classification,
    strong: Classification,
}

fn pair(wire: [WireRatio; 2]) -> [Ratio; 2] {
    wire.map(|x| Ratio::from_wire(x).unwrap())
}

fn validate_settings(s: Settings, kind: &str) {
    assert_eq!(s.reference_kind, kind);
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
    for (action, (id, name, utilities)) in s.actions.into_iter().zip([
        (0, "alice_only", [r(30, 1), r(0, 1)]),
        (1, "equal_split", [r(20, 1); 2]),
        (2, "bob_only", [r(0, 1), r(30, 1)]),
    ]) {
        assert_eq!(
            (action.id, action.name.as_str(), pair(action.utilities)),
            (id, name, utilities)
        );
    }
    assert_eq!(s.policies.len(), 9);
    for (id, p) in s.policies.into_iter().enumerate() {
        assert_eq!(
            (p.id, p.red_action, p.green_action),
            (id as u8, id as u8 / 3, id as u8 % 3)
        );
    }
    assert_eq!(
        s.weights
            .into_iter()
            .map(|x| Ratio::from_wire(x).unwrap())
            .collect::<Vec<_>>(),
        expected_weights()
    );
    assert_eq!(s.likelihood_models.len(), 3);
    for (id, model) in s.likelihood_models.into_iter().enumerate() {
        assert_eq!(model.id, id);
        assert_eq!(
            pair(model.red_likelihoods),
            [[r(9, 10), r(1, 10)], [r(1, 2); 2], [r(1, 1), r(4, 5)]][id]
        );
    }
    assert_eq!(
        s.priors.into_iter().map(pair).collect::<Vec<_>>(),
        [[r(0, 1), r(1, 1)], [r(1, 2); 2], [r(1, 1), r(0, 1)]]
    );
}

#[test]
fn all_225_objectives_and_complete_maximizers_match_sealed_reference() {
    let bytes = include_bytes!("fixtures/cake-reference.json");
    assert!(bytes.len() <= 8 * 1024 * 1024);
    let fixture: Fixture<CakeRow> = serde_json::from_slice(bytes).unwrap();
    assert_eq!(fixture.schema, "negotiable-foundation-v1");
    validate_settings(fixture.settings, "cake");
    assert_eq!(fixture.rows.len(), 225);
    let mut keys = BTreeSet::new();
    for (row, (weight, id)) in fixture.rows.iter().zip(
        expected_weights()
            .into_iter()
            .flat_map(|w| (0..9).map(move |id| (w, id))),
    ) {
        assert_eq!(Ratio::from_wire(row.weight.clone()).unwrap(), weight);
        assert_eq!(row.policy_id, id);
        assert!(keys.insert((row.weight.num.clone(), row.weight.den.clone(), id)));
        pair(row.expected.clone());
        Ratio::from_wire(row.objective.clone()).unwrap();
        for set in [&row.global_maximizers, &row.local_maximizers] {
            assert!(!set.is_empty());
            assert!(set.iter().all(|&id| id < 9));
            assert!(set.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }
    let actual = cake_rows().unwrap();
    assert_eq!(actual.len(), 225);
    for (actual, expected) in actual.iter().zip(&fixture.rows) {
        assert_eq!(actual, expected);
    }
}

#[test]
fn all_nine_stochastic_labels_match_and_every_witness_is_verified() {
    let bytes = include_bytes!("fixtures/pareto-reference.json");
    assert!(bytes.len() <= 8 * 1024 * 1024);
    let fixture: Fixture<ParetoRow> = serde_json::from_slice(bytes).unwrap();
    assert_eq!(fixture.schema, "negotiable-foundation-v1");
    validate_settings(fixture.settings, "pareto");
    assert_eq!(fixture.rows.len(), 9);
    let points = (0..9)
        .map(|id| cake_payoff(Policy::from_id(id).unwrap()).unwrap())
        .collect::<Vec<_>>();
    for (id, row) in fixture.rows.into_iter().enumerate() {
        assert_eq!(row.policy_id as usize, id);
        assert_eq!(pair(row.payoff), points[id]);
        for (kind, expected) in [
            (ParetoKind::PaperWeak, row.paper_weak),
            (ParetoKind::Strong, row.strong),
        ] {
            verify_classification(&points, id, kind, &expected, 3);
            let actual = classify(&points, id, kind).unwrap();
            assert_eq!(
                actual.efficient, expected.efficient,
                "policy {id}, {kind:?}"
            );
            verify_classification(&points, id, kind, &actual, 2);
            assert_eq!(
                classify(&points, id, kind).unwrap(),
                actual,
                "stable witness enumeration"
            );
        }
    }
}
