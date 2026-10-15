use super::super::*;
#[test]
fn strict_config_rejects_duplicates_unknowns_omissions_and_invalid_ids() {
    let c = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Left);
    let raw = serde_json::to_string(&c).unwrap();
    assert_eq!(config_from_json(&raw).unwrap(), c);
    for bad in [
        raw.replacen("{", "{\"arm\":\"Erased\",", 1),
        raw.replacen("{", "{\"extra\":0,", 1),
        raw.replace("Agent-A", "bad id"),
        raw.replace("Agent-B", "Agent-A"),
        raw.replace("\"acquisition_cost\":1", "\"acquisition_cost\":1.0"),
        raw.replace("\"acquisition_cost\":1", "\"acquisition_cost\":3"),
        raw.replace(
            "\"acquisition_cost\":1",
            "\"acquisition_cost\":18446744073709551616",
        ),
    ] {
        assert!(config_from_json(&bad).is_err(), "{bad}");
    }
}
#[test]
fn primitive_wire_rejects_mutation_and_keeps_integer_overflow_operational() {
    for raw in [
        r#"{"kind":"Wait","kind":"Move","cell":null,"note":null,"good":null}"#,
        r#"{"kind":"Wait","cell":null,"note":null,"good":null,"extra":0}"#,
        r#"{"kind":"Wait","cell":null,"note":null}"#,
        r#"{"kind":"WriteNote","cell":null,"note":"Blank","good":null}"#,
        r#"{"kind":"Move","cell":null,"note":null,"good":null}"#,
    ] {
        assert!(primitive_from_json(raw).is_err());
    }
    assert_eq!(
        super::super::validation::parse::<Stratum>(r#"{"acquisition_cost":18446744073709551616}"#)
            .unwrap_err(),
        Error::ArithmeticOverflow
    );
}
#[test]
fn inconsistent_custody_ledgers_and_duplicate_goods_are_rejected() {
    let c = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Left);
    let p = super::super::kernel::World::new(&c)
        .unwrap()
        .checkpoint()
        .clone();
    let mut bad = p.clone();
    bad.physical.participants[1].key = true;
    assert!(super::super::validation::validate_checkpoint(&bad).is_err());
    let mut bad = p.clone();
    bad.physical.participants[0].credits = 31;
    assert!(super::super::validation::validate_checkpoint(&bad).is_err());
    let mut bad = p;
    bad.physical.participants[1].cargo = Some(Good::Left);
    bad.own[1].cargo = Some(Good::Left);
    assert!(super::super::validation::validate_checkpoint(&bad).is_err());
}

#[test]
fn every_sealed_negative_wire_is_rejected_with_its_operational_kind() {
    let f: serde_json::Value =
        serde_json::from_slice(super::super::source_inputs::REFERENCE_INPUTS[1].1).unwrap();
    let mut config = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Left);
    config.stratum.manifest_inspection_cost = 2;
    let world = super::super::kernel::World::new(&config).unwrap();
    let mut initial = Vec::new();
    super::super::canonical::write_record(&mut initial, world.checkpoint(), 98766).unwrap();
    let context = super::super::sha256::digest(&initial);
    for input in f["payload"]["invalid_inputs"].as_array().unwrap() {
        let raw = input["raw"].as_str().unwrap();
        let error = if raw.contains("acquisition_cost") {
            super::super::validation::parse::<Stratum>(raw).unwrap_err()
        } else {
            primitive_from_json(raw).unwrap_err()
        };
        assert_eq!(context, input["unchanged_sha256"].as_str().unwrap());
        if let Error::ResourceLimit { resource, limit } = &error {
            assert_eq!(
                resource,
                input["expected_error"]["resource"].as_str().unwrap()
            );
            assert_eq!(*limit, input["expected_error"]["limit"].as_u64().unwrap());
        } else {
            assert!(input["expected_error"]["resource"].is_null());
            assert!(input["expected_error"]["limit"].is_null());
        }
        assert_eq!(world.checkpoint().clock.boundary, 0);
        assert_eq!(
            error.kind(),
            input["expected_error"]["kind"].as_str().unwrap(),
            "{}",
            input["name"]
        );
    }
}
