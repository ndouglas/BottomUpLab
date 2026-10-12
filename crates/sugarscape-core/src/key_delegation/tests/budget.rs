use super::*;
#[test]
fn new_wire_schemas_reject_arrays_duplicates_and_missing_nulls() {
    assert!(serde_json::from_str::<Limits>("[1,2,3]").is_err());
    assert!(serde_json::from_str::<Limits>(
        r#"{"profiles":1,"transitions":2,"allocation_bytes":3,"profiles":1}"#
    )
    .is_err());
    assert!(serde_json::from_str::<Selection>(
        r#"{"initial_owner":"Receiver","profile":{"giver":0,"receiver":60}}"#
    )
    .is_err());
    assert!(serde_json::from_str::<CompileStats>(
        r#"{"profiles":-0,"transitions":2,"peak_bytes":3}"#
    )
    .is_err());
}
#[test]
fn malformed_registered_limits_reject() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/limits-reference.json")).unwrap();
    v["limits"]["allocation_bytes"] = 1.into();
    assert!(super::super::budget::Limits::from_reference(&v.to_string()).is_err());
}
#[test]
fn counters_reject_before_retaining_the_prospective_step() {
    use super::super::budget::{Budget, FIXED};
    for limit in [0, 1, 2] {
        let l = Limits {
            profiles: limit,
            transitions: limit,
            allocation_bytes: u64::MAX,
        };
        let mut b = Budget::new(&l).unwrap();
        for _ in 0..limit {
            b.profile().unwrap();
        }
        assert!(b.profile().is_err());
        assert_eq!(b.stats.profiles, limit);
        b.boundary(0, limit).unwrap();
        assert!(b.boundary(0, 1).is_err());
        assert_eq!(b.stats.transitions, limit);
    }
    for limit in [0, FIXED - 1, FIXED, FIXED + 1] {
        let l = Limits {
            profiles: 1,
            transitions: 1,
            allocation_bytes: limit,
        };
        assert_eq!(Budget::new(&l).is_ok(), limit >= FIXED);
    }
    for extra in [0, 1, 2] {
        let l = Limits {
            profiles: 1,
            transitions: 1,
            allocation_bytes: FIXED + 100 + 4 + extra,
        };
        let mut b = Budget::new(&l).unwrap();
        assert_eq!(b.row(101, 0).is_ok(), extra >= 1);
        assert_eq!(b.retained, if extra >= 1 { 105 } else { 0 });
    }
}
#[test]
fn preflight_rejection_has_no_candidate_materialization_or_capture() {
    super::super::kernel::test_preflight_order(&config());
}
#[test]
fn streamed_lengths_match_all_independent_boundaries_before_retention() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/kernel-reference.json")).unwrap();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let c: Config = serde_json::from_value(case["config"].clone()).unwrap();
        let p: Profile = serde_json::from_value(case["profile"].clone()).unwrap();
        let expected = case["boundaries"].as_array().unwrap();
        let mut i = 0;
        super::super::kernel::run_budgeted(&c, p, &mut |e, _| {
            let n = e.json_len()?;
            assert_eq!(n, super::super::budget::json_len(e)?);
            assert_eq!(
                super::super::budget::scratch(n)?,
                expected[i]["scratch_bytes"].as_u64().unwrap()
            );
            i += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(i, expected.len());
        count += i;
    }
    assert_eq!(count, 304);
}
#[test]
fn registered_limits_reject_wrong_versions_extra_stats_and_duplicates() {
    let text = include_str!("fixtures/limits-reference.json");
    let v: serde_json::Value = serde_json::from_str(text).unwrap();
    let mut bad = v.clone();
    bad["version"] = "wrong".into();
    assert!(super::super::budget::Limits::from_reference(&bad.to_string()).is_err());
    let mut bad = v.clone();
    bad["panels"][0]["stats"]["extra"] = 1.into();
    assert!(super::super::budget::Limits::from_reference(&bad.to_string()).is_err());
    let duplicate = text.replacen("\"version\":", "\"version\":\"wrong\",\"version\":", 1);
    assert!(super::super::budget::Limits::from_reference(&duplicate).is_err());
}
