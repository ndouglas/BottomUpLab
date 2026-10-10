use super::super::diagnostics::{self, Library};
use super::super::*;
fn reference() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/evaluation-reference.json")).unwrap()
}
fn incomplete_reference_report() -> DiagnosticReport {
    let sealed = diagnostics::references::Reference::read().unwrap();
    let mut report = diagnostics::new_report(&sealed).unwrap();
    let reference = reference();
    let witness = &reference["episodes"][0];
    let episode: Episode = serde_json::from_value(witness["episode"].clone()).unwrap();
    let mut library = Library::default();
    let compact = library.compact(&episode).unwrap();
    report.histories = library.histories;
    let mut projection = reference["panels"][0]["projection"].clone();
    projection
        .as_object_mut()
        .unwrap()
        .remove("episode_projection_receipt");
    report.panels.push(CompactPanel {
        protocol: serde_json::from_value(witness["protocol"].clone()).unwrap(),
        environment: serde_json::from_value(witness["environment"].clone()).unwrap(),
        aggregate: serde_json::from_value(reference["panels"][0]["aggregate"].clone()).unwrap(),
        projection: serde_json::from_value(projection).unwrap(),
        episodes: vec![compact],
    });
    report.passed = true;
    for check in &mut report.checks {
        check.passed = true;
    }
    report
}
#[test]
fn census_is_fixed_ordered_and_excludes_known_transfer() {
    let settings = registered_settings();
    assert_eq!(settings.len(), 48);
    assert_eq!(
        settings
            .iter()
            .filter(|(_, e)| matches!(e, Environment::InFamily(_)))
            .count(),
        40
    );
    assert_eq!(
        settings
            .iter()
            .filter(|(_, e)| *e == Environment::DataFlip)
            .count(),
        8
    );
    for model in Mechanism::ALL {
        for role in [Role::A, Role::B] {
            let policies: Vec<_> = settings
                .iter()
                .filter(|(p, e)| p.investigator == role && *e == Environment::InFamily(model))
                .map(|(p, _)| p.policy)
                .collect();
            assert_eq!(
                policies,
                vec![
                    PolicyKind::RawAdaptive,
                    PolicyKind::SuppliedAdaptive,
                    PolicyKind::NoProbe,
                    PolicyKind::KnownRaw,
                    PolicyKind::InspectOnly
                ]
            );
        }
    }
    assert!(!settings
        .iter()
        .any(|(p, e)| p.policy == PolicyKind::KnownRaw && *e == Environment::DataFlip));
    let reference = reference();
    for (index, (protocol, environment)) in settings.iter().enumerate() {
        assert_eq!(
            serde_json::to_value(protocol).unwrap(),
            reference["panels"][index]["protocol"]
        );
        assert_eq!(
            serde_json::to_value(environment).unwrap(),
            reference["panels"][index]["environment"]
        );
    }
}
#[test]
fn cached_passed_flags_cannot_validate_a_partial_report() {
    let mut report = incomplete_reference_report();
    report.passed = true;
    assert!(!report_integrity(&report, &Limits::registered().unwrap()).unwrap());
    assert!(summary_json(&report).is_err());
}
#[test]
fn compact_reconstruction_equals_every_bounded_complete_reference_witness() {
    let reference = reference();
    let sealed = diagnostics::references::Reference::read().unwrap();
    for witness in reference["episodes"].as_array().unwrap() {
        let mut report = diagnostics::new_report(&sealed).unwrap();
        let episode: Episode = serde_json::from_value(witness["episode"].clone()).unwrap();
        let mut library = Library::default();
        let row = library.compact(&episode).unwrap();
        report.histories = library.histories;
        let panel = reference["panels"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| {
                p["protocol"] == witness["protocol"] && p["environment"] == witness["environment"]
            })
            .unwrap();
        let mut projection = panel["projection"].clone();
        projection
            .as_object_mut()
            .unwrap()
            .remove("episode_projection_receipt");
        for f in projection["failures"].as_array_mut().unwrap() {
            f.as_object_mut().unwrap().remove("prefixes");
        }
        // Witness sequences are zero; a missing earlier row is never fabricated.
        assert_eq!(episode.sequence, 0);
        report.panels.push(CompactPanel {
            protocol: serde_json::from_value(witness["protocol"].clone()).unwrap(),
            environment: serde_json::from_value(witness["environment"].clone()).unwrap(),
            aggregate: serde_json::from_value(panel["aggregate"].clone()).unwrap(),
            projection: serde_json::from_value(projection).unwrap(),
            episodes: vec![row],
        });
        assert_eq!(
            serde_json::to_value(reconstruct_episode(&report, 0, 0).unwrap()).unwrap(),
            witness["episode"]
        );
    }
}
#[test]
fn current_episode_validation_rejects_independent_scientific_mutations() {
    let report = incomplete_reference_report();
    let protocol = report.panels[0].protocol.clone();
    let limits = Limits::registered().unwrap();
    let policy = CompiledPolicy::build(&protocol, OwnPrior::Uniform, &limits).unwrap();
    let replay = Replay::build(&protocol, &policy, &limits).unwrap();
    assert!(diagnostics::episode_matches_current(&report, 0, 0, &policy, &replay).unwrap());
    let mut mutations = Vec::new();
    let mut r = report.clone();
    r.panels[0].episodes[0].metrics[0].spent += 1;
    mutations.push(("cost", r));
    let mut r = report.clone();
    r.panels[0].episodes[0].decisions[0].decision.choice = Choice::Stop;
    mutations.push(("choice", r));
    let mut r = report.clone();
    r.panels[0].episodes[0].decisions[0].decision.alternatives[0]
        .value
        .numerator += 1;
    mutations.push(("alternative numerator", r));
    let mut r = report.clone();
    r.histories[0].role = Role::B;
    mutations.push(("role", r));
    let mut r = report.clone();
    r.histories[0].ids.surface = "forged".into();
    mutations.push(("IDs", r));
    let mut r = report.clone();
    r.panels[0].environment = Environment::InFamily(Mechanism::Inert);
    mutations.push(("environment", r));
    let mut r = report.clone();
    r.panels[0].episodes[0].sequence = 1;
    mutations.push(("sequence", r));
    let mut r = report.clone();
    r.panels[0].episodes[0].histories[0] = u64::MAX;
    mutations.push(("history index", r));
    let mut r = report.clone();
    r.panels[0].episodes[0].decisions[0].prefix.entry_count = u64::MAX;
    mutations.push(("prefix index", r));
    let mut r = report.clone();
    r.panels[0].episodes[0].decisions[0].prefix.checkpoint = Checkpoint::Finished;
    mutations.push(("prefix checkpoint", r));
    let mut r = report.clone();
    let event = r.histories[0]
        .entries
        .iter_mut()
        .find_map(|e| {
            if let Entry::Physical(LocalEntry::Action(event)) = e {
                if matches!(event.outcome, crate::shared_surface::Outcome::Read(_)) {
                    Some(event)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    event.outcome = crate::shared_surface::Outcome::Read(Symbol::Blank);
    mutations.push(("observation", r));
    let mut r = report.clone();
    r.panels[0].episodes[0].prediction_beliefs[0][0].models[0].denominator = 0;
    mutations.push(("rational", r));
    for (label, r) in mutations {
        // Each logical revalidation gets a fresh compilation and replay owner.
        let policy = CompiledPolicy::build(&protocol, OwnPrior::Uniform, &limits).unwrap();
        let replay = Replay::build(&protocol, &policy, &limits).unwrap();
        assert!(
            !matches!(
                diagnostics::episode_matches_current(&r, 0, 0, &policy, &replay),
                Ok(true)
            ),
            "{label}"
        );
    }
}
#[test]
fn full_shape_missing_duplicate_and_reordered_settings_or_sequences_fail_before_execution() {
    let report = incomplete_reference_report();
    let limits = Limits::registered().unwrap();
    for mode in 0..5 {
        let mut r = report.clone();
        r.panels = vec![report.panels[0].clone(); 48];
        for (p, (protocol, environment)) in r.panels.iter_mut().zip(registered_settings()) {
            p.protocol = protocol;
            p.environment = environment;
            p.episodes = (0..256)
                .map(|sequence| {
                    let mut e = report.panels[0].episodes[0].clone();
                    e.sequence = sequence;
                    e
                })
                .collect();
        }
        match mode {
            0 => {
                r.panels.pop();
            }
            1 => r.panels[1] = r.panels[0].clone(),
            2 => r.panels.swap(0, 1),
            3 => r.panels[0].episodes[1].sequence = 0,
            4 => r.panels[0].episodes.swap(0, 1),
            _ => unreachable!(),
        }
        assert!(!report_integrity(&r, &limits).unwrap());
    }
}
#[test]
fn sealed_receipts_are_bound_separately_and_source_mismatch_is_contextual() {
    let report = incomplete_reference_report();
    let limits = Limits::registered().unwrap();
    assert!(report
        .reference_artifacts
        .iter()
        .any(|r| r.path.ends_with("setting-47-episodes.jsonl.gz")));
    assert!(report
        .reference_artifacts
        .iter()
        .any(|r| r.path.ends_with("full-episodes.jsonl.gz")));
    let mut forged = report.clone();
    forged.reference_artifacts[0].sha256 = "0".repeat(64);
    assert!(!report_integrity(&forged, &limits).unwrap());
    let mut forged = report.clone();
    forged.reference_artifacts[0].bytes += 1;
    assert!(!report_integrity(&forged, &limits).unwrap());
    let mut forged = report.clone();
    forged.source_hash = "0".repeat(64);
    assert!(matches!(
        report_integrity(&forged, &limits),
        Err(Error::InvalidProtocol(_))
    ));
}
#[test]
fn hashing_matches_independent_standard_vectors() {
    for (bytes, want) in [
        (
            "",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            "abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
    ] {
        assert_eq!(diagnostics::hash::sha256(bytes.as_bytes()), want);
    }
}
#[test]
fn tiny_engine_limits_remain_computational_errors_without_episode_evidence() {
    let limits = Limits {
        states: 1,
        branches: 1,
        allocation_bytes: 1,
    };
    assert!(matches!(
        diagnose(&limits),
        Err(Error::ResourceLimit { .. })
    ));
}
#[test]
fn streaming_report_hash_matches_exact_json_and_summary_retains_unavailable_mass() {
    let report = incomplete_reference_report();
    let bytes = serde_json::to_vec(&report).unwrap();
    assert_eq!(
        diagnostics::content_hash(&report).unwrap(),
        diagnostics::hash::sha256(&bytes)
    );
    let summary = diagnostics::build_summary(&report).unwrap();
    assert_eq!(summary.report_hash, diagnostics::hash::sha256(&bytes));
    assert_eq!(summary.source_hash, report.source_hash);
    assert_eq!(summary.settings[0].projection, report.panels[0].projection);
    assert!(serde_json::to_value(summary)
        .unwrap()
        .get("histories")
        .is_none());
    use std::io::Write;
    let mut digest = diagnostics::hash::Digest::default();
    for chunk in bytes.chunks(3) {
        digest.write_all(chunk).unwrap();
    }
    assert_eq!(digest.finish(), diagnostics::hash::sha256(&bytes));
}
#[test]
fn malformed_compact_index_preserves_context_instead_of_becoming_an_episode_outcome() {
    let mut report = incomplete_reference_report();
    let row = report.panels[0].clone();
    report.panels = vec![row.clone(); 48];
    for (panel, (protocol, environment)) in report.panels.iter_mut().zip(registered_settings()) {
        panel.protocol = protocol;
        panel.environment = environment;
        panel.episodes = (0..256)
            .map(|sequence| {
                let mut episode = row.episodes[0].clone();
                episode.sequence = sequence;
                episode
            })
            .collect();
    }
    report.panels[0].episodes[0].histories[0] = u64::MAX;
    let error = report_integrity(&report, &Limits::registered().unwrap()).unwrap_err();
    assert!(matches!(error, Error::InvalidReport(_)));
    assert!(error.to_string().contains("history index out of range"));
}
