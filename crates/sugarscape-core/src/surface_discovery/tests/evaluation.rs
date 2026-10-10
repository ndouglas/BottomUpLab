use super::super::*;

#[test]
fn complete_episodes_match_independent_reference() {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/evaluation-reference.json")).unwrap();
    for row in reference["episodes"].as_array().unwrap() {
        let protocol: Protocol = serde_json::from_value(row["protocol"].clone()).unwrap();
        let prior = serde_json::from_value(row["prior"].clone()).unwrap();
        let environment = serde_json::from_value(row["environment"].clone()).unwrap();
        let sequence = row["sequence"].as_u64().unwrap() as u16;
        let limits = Limits::registered().unwrap();
        let policy = CompiledPolicy::build(&protocol, prior, &limits).unwrap();
        let replay = Replay::build(&protocol, &policy, &limits).unwrap();
        let actual = run_episode(&protocol, environment, sequence, &policy, &replay).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            row["episode"],
            "{:?} {:?} {:?} {}",
            protocol.investigator,
            protocol.policy,
            environment,
            sequence
        );
    }
}

#[test]
fn all_population_aggregates_projections_and_contrasts_match_reference() {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/evaluation-reference.json")).unwrap();
    let planning: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/planning-reference.json")).unwrap();
    let limits = Limits::registered().unwrap();
    let mut panels = Vec::new();
    for initial in planning["initial_values"].as_array().unwrap() {
        let protocol: Protocol = serde_json::from_value(initial["protocol"].clone()).unwrap();
        let prior = serde_json::from_value(initial["prior"].clone()).unwrap();
        let start = std::time::Instant::now();
        let policy = CompiledPolicy::build(&protocol, prior, &limits).unwrap();
        let replay = Replay::build(&protocol, &policy, &limits).unwrap();
        for row in reference["panels"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["protocol"] == initial["protocol"] && r["prior"] == initial["prior"])
        {
            let environment = serde_json::from_value(row["environment"].clone()).unwrap();
            let panel = evaluate_panel(&protocol, environment, &policy, &replay).unwrap();
            assert_eq!(
                serde_json::to_value(&panel.aggregate).unwrap(),
                row["aggregate"],
                "setting {}",
                row["setting"]
            );
            let mut expected = row["projection"].clone();
            expected
                .as_object_mut()
                .unwrap()
                .remove("episode_projection_receipt");
            for failure in expected["failures"].as_array_mut().unwrap() {
                failure.as_object_mut().unwrap().remove("prefixes");
            }
            assert_eq!(
                serde_json::to_value(project_panel(&panel).unwrap()).unwrap(),
                expected,
                "setting {}",
                row["setting"]
            );
            if let Ok(directory) = std::env::var("SUGARSCAPE_DISCOVERY_PARITY_DIR") {
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(std::path::Path::new(&directory).join(format!(
                        "panel-{:02}.json",
                        row["setting"].as_u64().unwrap()
                    )))
                    .unwrap();
                serde_json::to_writer(std::io::BufWriter::new(file), &panel).unwrap();
            }
            panels.push(panel);
        }
        eprintln!(
            "complete compilation engineering {:?} {:?} {:?}: {:?}, elapsed {:?}",
            protocol.investigator,
            protocol.policy,
            prior,
            replay.stats(),
            start.elapsed()
        );
    }
    for row in reference["comparisons"].as_array().unwrap() {
        let find = |name: &str| {
            panels
                .iter()
                .find(|p| {
                    serde_json::to_value(p.protocol.investigator).unwrap() == row["investigator"]
                        && serde_json::to_value(p.environment).unwrap() == row["environment"]
                        && serde_json::to_value(p.protocol.policy).unwrap() == row[name]
                })
                .unwrap()
        };
        assert_eq!(
            serde_json::to_value(
                compare_panels(find("left_policy"), find("right_policy")).unwrap()
            )
            .unwrap(),
            *row
        );
    }
    for row in reference["role_expectations"].as_array().unwrap() {
        let group: Vec<_> = panels
            .iter()
            .filter(|p| {
                serde_json::to_value(&p.protocol).unwrap() == row["protocol"]
                    && matches!(p.environment, Environment::InFamily(_))
            })
            .cloned()
            .collect();
        assert_eq!(
            serde_json::to_value(uniform_actual_mechanism_summary(&group).unwrap()).unwrap(),
            *row
        );
        let projection = project_uniform_actual_mechanisms(&group).unwrap();
        for bins in projection.posterior_calibration {
            for bin in bins {
                assert_eq!(
                    bin.ones_mass,
                    bin.probability.checked_mul(bin.issued_mass).unwrap()
                );
            }
        }
    }
    for row in reference["role_contrasts"].as_array().unwrap() {
        let group = |name: &str| {
            panels
                .iter()
                .filter(|p| {
                    serde_json::to_value(p.protocol.investigator).unwrap() == row["investigator"]
                        && serde_json::to_value(p.protocol.policy).unwrap() == row[name]
                        && matches!(p.environment, Environment::InFamily(_))
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            serde_json::to_value(
                compare_uniform_actual_mechanisms(&group("left_policy"), &group("right_policy"))
                    .unwrap()
            )
            .unwrap(),
            *row
        );
    }
    assert_eq!(panels.len(), 48);
}

#[test]
fn first_helper_contradiction_halts_before_another_paid_or_free_transition() {
    let protocol = Protocol::new(Role::B, PolicyKind::InspectOnly);
    let limits = Limits::registered().unwrap();
    let policy = CompiledPolicy::build(&protocol, OwnPrior::Uniform, &limits).unwrap();
    let replay = Replay::build(&protocol, &policy, &limits).unwrap();
    let episode = run_episode(&protocol, Environment::DataFlip, 0, &policy, &replay).unwrap();
    let failure = episode.failure.as_ref().unwrap();
    let position = Position {
        phase: Phase::Live { trial: 0 },
        round: 3,
        slot: 1,
    };
    assert_eq!(failure.role, Role::A);
    assert_eq!(failure.prefix.checkpoint, Checkpoint::AfterSlot(position));
    assert_eq!(failure.spent, [5, 7]);
    assert_eq!(episode.privileged.last().unwrap().event.position, position);
    assert!(episode
        .histories
        .iter()
        .all(|p| p.checkpoint == Checkpoint::AfterSlot(position)));
    assert!(episode.predictions.iter().all(Vec::is_empty));
    assert!(episode
        .metrics
        .iter()
        .all(|m| m.reward.is_none() && m.net.is_none() && m.correct.is_none()));
    assert_eq!(
        episode.catalog_discoveries[0].as_ref().unwrap().source,
        CatalogCertaintySource::Supplied
    );
    assert!(failure.last_supported.is_some());
}

#[test]
fn prediction_ties_are_zero_and_missing_target_is_an_error() {
    let mut belief = Belief {
        models: [Probability::new(1, 4).unwrap(); 4],
        target: Some(Probability::new(1, 2).unwrap()),
    };
    assert!(!predict_target(&belief).unwrap());
    belief.target = None;
    assert!(matches!(
        predict_target(&belief),
        Err(Error::InvalidHistory(_))
    ));
}

// Synthetic metric records exercise partial predictions, which do not occur
// before the first contradiction in the registered actual census.
#[test]
fn partial_prediction_mass_is_retained_and_terminal_comparisons_use_no_survivor_mean() {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/evaluation-reference.json")).unwrap();
    let row = &reference["episodes"][0];
    let episode: Episode = serde_json::from_value(row["episode"].clone()).unwrap();
    let mut panel = Panel {
        protocol: serde_json::from_value(row["protocol"].clone()).unwrap(),
        environment: serde_json::from_value(row["environment"].clone()).unwrap(),
        episodes: (0..256)
            .map(|sequence| {
                let mut e = episode.clone();
                e.sequence = sequence;
                e
            })
            .collect(),
        aggregate: serde_json::from_value(reference["panels"][0]["aggregate"].clone()).unwrap(),
    };
    let right = panel.clone();
    let partial = &mut panel.episodes[0];
    partial.failure = Some(Failure {
        role: Role::A,
        prefix: partial.histories[0].clone(),
        last_supported: Some(partial.prediction_beliefs[0][1].clone()),
        spent: partial.metrics.clone().map(|m| m.spent),
    });
    for role in 0..2 {
        partial.predictions[role].truncate(2);
        partial.prediction_beliefs[role].truncate(2);
        partial.metrics[role].reward = None;
        partial.metrics[role].net = None;
        partial.metrics[role].correct = None;
        partial.model_projections[role] = None;
    }
    let projection = project_panel(&panel).unwrap();
    assert_eq!(
        projection.unavailable_mass,
        Probability::new(1, 256).unwrap()
    );
    assert_eq!(projection.accuracy, [None, None]);
    assert!(projection
        .identification
        .iter()
        .all(|i| i.true_model_mass.is_none()));
    for bins in &projection.posterior_calibration {
        assert_eq!(
            bins.iter().find(|b| b.trial == 0).unwrap().issued_mass,
            Probability::new(1, 1).unwrap()
        );
        assert_eq!(
            bins.iter().find(|b| b.trial == 2).unwrap().issued_mass,
            Probability::new(255, 256).unwrap()
        );
    }
    let contrast = compare_panels(&panel, &right).unwrap();
    assert_eq!(
        contrast.difference.failed_mass,
        Probability::new(1, 256).unwrap()
    );
    assert_eq!(contrast.difference.net, [None, None]);
    assert_eq!(contrast.difference.reward, [None, None]);
    assert_eq!(contrast.difference.group_net, None);
    assert_eq!(
        contrast.difference.spent,
        [ScoreFraction::new(0, 1).unwrap(); 2]
    );
    panel.episodes.reverse();
    assert_eq!(compare_panels(&panel, &right).unwrap(), contrast);
    panel.episodes[0].sequence = panel.episodes[1].sequence;
    assert!(matches!(
        compare_panels(&panel, &right),
        Err(Error::InvalidReport(_))
    ));
    panel.episodes.pop();
    assert!(matches!(
        project_panel(&panel),
        Err(Error::InvalidReport(_))
    ));
}
