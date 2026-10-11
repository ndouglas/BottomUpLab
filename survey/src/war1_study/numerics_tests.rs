use super::{numerics, wire};
#[test]
fn exact_dyadic_inputs_and_registered_ids() {
    let manifest = wire::decode_manifest(wire::FIXTURE).unwrap();
    let data = numerics::export_numeric_inputs(&manifest).unwrap();
    let state = data
        .states
        .iter()
        .find(|s| s.arm_id == "oracle.2v1.N1.aimed_fire.dt1-64" && s.blue == 2)
        .unwrap();
    assert_eq!(state.targets[0].hazard_bits, "3fe0000000000000");
    assert_eq!(state.targets[0].dose_bits, "3f80000000000000");
    assert_eq!(state.targets[1].dose_bits, "3fa0000000000000");
    assert_eq!(
        data.manifest_sha256,
        "12ad58fd65e353c1f09d02a6b7dea712bbf921ab4be100cf4f2274215c51074b"
    );
}
#[test]
fn any_mutation_is_refused_before_resolution() {
    for bytes in [
        b"{}".as_slice(),
        b"{\"a\":1,\"a\":2}",
        b"{\"n\":1e-999}",
        b"{\"n\":NaN}",
    ] {
        assert!(wire::decode_manifest(bytes).is_err());
    }
    let mut bytes = wire::FIXTURE.to_vec();
    bytes.push(b' ');
    assert!(wire::decode_manifest(&bytes).is_err());
}
#[test]
fn fabricated_manifest_cannot_claim_fixture_identity() {
    let mut manifest = wire::decode_manifest(wire::FIXTURE).unwrap();
    manifest.purpose = "prospective_seed_population".into();
    assert!(numerics::export_numeric_inputs(&manifest).is_err());
}
