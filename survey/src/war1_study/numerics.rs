//! Pure arithmetic exports: no simulation or random stream is constructed.
use super::wire;
use serde::{Deserialize, Serialize};
use sugarscape_core::war::{
    config::{Geometry, RULES_ID},
    math,
};
#[derive(PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericTarget {
    pub hazard_bits: String,
    pub dose_bits: String,
    pub p_bits: String,
    pub q_numerator: u64,
}
#[derive(PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericState {
    pub arm_id: String,
    pub blue: u32,
    pub red: u32,
    pub targets: [NumericTarget; 2],
}
#[derive(PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericExport {
    pub schema: String,
    pub purpose: String,
    pub manifest_sha256: String,
    pub rules_id: String,
    pub binding_state: String,
    pub compiled_inputs_sha256: Option<String>,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,18>")]
    pub states: Vec<NumericState>,
}
pub fn export_numeric_inputs(manifest: &wire::Manifest) -> Result<NumericExport, String> {
    if manifest != &wire::decode_manifest(wire::SELECTED)? {
        return Err("numeric export requires the exact compiled selected manifest".into());
    }
    let proposal: serde_json::Value =
        serde_json::from_slice(wire::PROPOSAL).map_err(|e| e.to_string())?;
    let arms: Vec<wire::Arm> =
        serde_json::from_value(proposal["arms"].clone()).map_err(|e| e.to_string())?;
    let mut states = Vec::new();
    for arm in arms.iter().filter(|a| a.group == "oracle") {
        let c = &arm.config;
        for blue in 1..=c.blue {
            for red in 1..=c.red {
                // Same public-kernel operator order: rate * source count / target count.
                let hazards = match c.geometry {
                    Geometry::DuelContact => [c.red_rate, c.blue_rate],
                    Geometry::AimedFire => [
                        c.red_rate * f64::from(red) / f64::from(blue),
                        c.blue_rate * f64::from(blue) / f64::from(red),
                    ],
                };
                let make_target = |hazard| -> Result<NumericTarget, String> {
                    let p = math::probability(hazard, c.dt)
                        .map_err(|e| format!("{}: {}", e.field, e.detail))?;
                    Ok(NumericTarget {
                        hazard_bits: format!("{:016x}", p.hazard.to_bits()),
                        dose_bits: format!("{:016x}", p.dose.to_bits()),
                        p_bits: format!("{:016x}", p.ideal.to_bits()),
                        q_numerator: (p.realized * 9_007_199_254_740_992.0) as u64,
                    })
                };
                states.push(NumericState {
                    arm_id: arm.arm_id.clone(),
                    blue,
                    red,
                    targets: [make_target(hazards[0])?, make_target(hazards[1])?],
                });
            }
        }
    }
    Ok(NumericExport {
        schema: "war1-study-numerics-v1".into(),
        purpose: manifest.purpose.clone(),
        manifest_sha256: wire::sha256(wire::SELECTED),
        rules_id: RULES_ID.into(),
        binding_state: "source_bound".into(),
        compiled_inputs_sha256: Some(super::source::verify()?.compiled_inputs_sha256),
        states,
    })
}
