use super::super::*;
#[test]
fn frozen_budget_and_atomic_low_cap() {
    let limits = Limits {
        episode_rows: 0,
        ..Limits::default()
    };
    assert_eq!(
        evaluate(&limits),
        Err(Error::ResourceLimit {
            resource: "episode_rows".into(),
            limit: 0
        })
    );
}

#[test]
fn every_counter_checks_before_increment_and_preserves_failed_charge() -> Result<(), Error> {
    use super::super::budget::Budget;
    let mut b = Budget::new(&Limits::default())?;
    for (name, limit) in [
        ("episode_rows", 4608),
        ("mean_rows", 2304),
        ("request_contrast_rows", 4608),
        ("contrast_rows", 2304),
        ("envelope_rows", 576),
        ("action_events", 46080),
        ("events", 55296),
        ("checkpoints", 73728),
        ("history_records", 46080),
        ("view_history_visits", 737280),
        ("map_cell_visits", 442368),
        ("metric_atom_operations", 555840),
        ("logical_storage_bytes", 140685568),
        ("serialization_buffer_bytes", 65536),
        ("canonical_data_bytes", 467519616),
    ] {
        let before = b.counters.clone();
        assert_eq!(
            b.charge(name, limit + 1),
            Err(Error::ResourceLimit {
                resource: name.into(),
                limit
            })
        );
        assert_eq!(b.counters, before);
    }
    Ok(())
}
#[test]
fn low_transition_caps_fail_before_candidate_and_capture() -> Result<(), Error> {
    use super::super::{budget::Budget, controller::run_budgeted};
    for resource in [
        "action_events",
        "history_records",
        "events",
        "checkpoints",
        "view_history_visits",
        "map_cell_visits",
        "checkpoints_per_episode",
    ] {
        let mut limits = Limits::default();
        match resource {
            "action_events" => limits.action_events = 0,
            "history_records" => limits.history_records = 0,
            "events" => limits.events = 0,
            "checkpoints" => limits.checkpoints = 0,
            "view_history_visits" => limits.view_history_visits = 0,
            "map_cell_visits" => limits.map_cell_visits = 0,
            _ => limits.checkpoints_per_episode = 0,
        }
        let mut budget = Budget::new(&limits)?;
        let mut seen = Vec::new();
        let result = run_budgeted(
            &Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Left),
            &mut |c| {
                seen.push(c.clock.boundary);
                Ok(())
            },
            &mut budget,
        );
        assert_eq!(
            result,
            Err(Error::ResourceLimit {
                resource: resource.into(),
                limit: 0
            })
        );
        assert!(seen.len() <= 1);
    }
    Ok(())
}
#[test]
fn bounded_encoded_length_rejects_cap_minus_one_without_allocation() -> Result<(), Error> {
    use super::super::budget::encoded_len;
    let value = Ratio::HALF;
    let n = encoded_len(&value, u64::MAX)?;
    assert_eq!(encoded_len(&value, n)?, n);
    assert_eq!(
        encoded_len(&value, n - 1),
        Err(Error::ResourceLimit {
            resource: "canonical_data_bytes".into(),
            limit: n - 1
        })
    );
    Ok(())
}
