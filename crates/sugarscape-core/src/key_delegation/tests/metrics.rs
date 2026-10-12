use super::*;
#[test]
fn control_selection_and_changed_initial_ownership_are_explicit() {
    let table = compile(config().stratum().clone(), &Limits::registered().unwrap()).unwrap();
    super::super::game::test_invalid_table(&table);
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/game-reference.json")).unwrap();
    assert_eq!(
        *table.stats(),
        serde_json::from_value::<CompileStats>(fixture["panels"][20]["stats"].clone()).unwrap()
    );
    assert_eq!(
        *table.analysis(),
        serde_json::from_value::<Analysis>(fixture["panels"][20]["analysis"].clone()).unwrap()
    );
    let solo = resolve(&table, Selector::SoloAttempt).unwrap().unwrap();
    assert_eq!(solo.profile, Profile::new(0, 63).unwrap());
    let supplied = resolve(&table, Selector::SuppliedHandoff).unwrap().unwrap();
    assert_eq!(supplied.profile, Profile::new(486, 60).unwrap());
    let equipped = resolve(&table, Selector::InitiallyEquipped)
        .unwrap()
        .unwrap();
    assert_eq!(equipped.initial_owner, InitialOwner::Receiver);
    assert!(equipped.classification.is_none());
    let controls = controls(&table).unwrap();
    assert_eq!(controls.len(), 6);
    let e = controls[5].episode.as_ref().unwrap();
    assert_eq!(e.outcome.spent, [0, 4]);
    assert_eq!(e.outcome.transfer_count, 0);
    assert_eq!(
        controls[5].classification_scope,
        ClassificationScope::ChangedInitialControl
    );
}
#[test]
fn contrasts_have_registered_order_and_exact_plural_absence_reasons() {
    let pairs = contrast_pairs();
    assert_eq!(pairs.len(), 84);
    assert_eq!(
        pairs
            .iter()
            .filter(|(a, _, _)| *a == ContrastAxis::TransferPermission)
            .count(),
        18
    );
    assert_eq!(
        pairs
            .iter()
            .filter(|(a, _, _)| *a == ContrastAxis::TransferCost)
            .count(),
        24
    );
    assert_eq!(
        pairs
            .iter()
            .filter(|(a, _, _)| *a == ContrastAxis::GiverShare)
            .count(),
        24
    );
    assert_eq!(
        pairs
            .iter()
            .filter(|(a, _, _)| *a == ContrastAxis::Distance)
            .count(),
        18
    );
    assert_eq!(
        super::super::metrics::missing(false, false, "pure_set_empty"),
        Some("both_pure_sets_empty".into())
    );
    assert_eq!(
        super::super::metrics::missing(false, false, "control_unavailable"),
        Some("both_controls_unavailable".into())
    );
}
