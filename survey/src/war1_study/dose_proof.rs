//! Fixed-roster domain proof; no Engine or RNG. See the independent Python
//! dose_proof module for the monotonic rectangle and gamma_3 derivations.
use super::wire;
use serde::{Deserialize, Serialize};
use sugarscape_core::war::{config::Geometry, math};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmProof {
    arm_id: String,
    config_sha256: String,
    interior_counts: [u32; 2],
    interior_target_states: u64,
    targets: [[String; 2]; 2],
    case_work: u64,
    quantization_only_path_bound: String,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DoseProof {
    schema: String,
    method: String,
    protocol_sha256: String,
    proposal_sha256: String,
    approved_roster_sha256: String,
    manifest_sha256: String,
    compiled_inputs_sha256: String,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,66>")]
    arms: Vec<ArmProof>,
    interior_target_states: u64,
    positive_dose_extrema: [String; 2],
    native_relative_error_bound: String,
    native_dose_enclosure: [String; 2],
    probability_lower_bound: String,
    survival_lower_bound: String,
    sampler_resolution: String,
    maximum_case_work: u64,
    quantization_only_path_bound: String,
}

#[derive(Clone, Copy)]
struct Rational(u128, u128);
impl Rational {
    fn new(n: u128, d: u128) -> Self {
        let (mut a, mut b) = (n, d);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        Self(n / a, d / a)
    }
    fn times(self, other: Self) -> Self {
        Self::new(self.0 * other.0, self.1 * other.1)
    }
    fn wire(self) -> String {
        format!("{}/{}", self.0, self.1)
    }
    fn less(self, other: Self) -> bool {
        self.0 * other.1 < other.0 * self.1
    }
}
fn canonical_hash(value: &serde_json::Value) -> Result<String, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(wire::sha256(&bytes))
}
fn approved() -> Result<(serde_json::Value, Vec<wire::Arm>), String> {
    let value: serde_json::Value =
        serde_json::from_slice(wire::PROPOSAL).map_err(|e| e.to_string())?;
    let arms: Vec<wire::Arm> =
        serde_json::from_value(value["arms"].clone()).map_err(|e| e.to_string())?;
    if arms.len() != 66 {
        return Err("dose proof approved census".into());
    }
    Ok((value, arms))
}
// The immutable approved roster uses rates in sixteenths and dt in sixty-fourths.
fn exact(value: f64, scale: u128) -> Result<Rational, String> {
    let n = value * scale as f64;
    if !n.is_finite() || n < 1.0 || n > scale as f64 || n.fract() != 0.0 {
        return Err("dose proof dyadic input domain".into());
    }
    Ok(Rational::new(n as u128, scale))
}
pub fn proof(manifest_sha256: &str, compiled_inputs_sha256: &str) -> Result<DoseProof, String> {
    let (value, approved) = approved()?;
    let mut arms = Vec::new();
    let (mut minimum, mut maximum) = (Rational(32, 1), Rational(0, 1));
    for (index, arm) in approved.iter().enumerate() {
        let c = &arm.config;
        if !(1..=512).contains(&c.blue)
            || !(1..=512).contains(&c.red)
            || !(0.25..=1.0).contains(&c.blue_rate)
            || !(0.25..=1.0).contains(&c.red_rate)
            || !(1.0 / 64.0..=1.0 / 16.0).contains(&c.dt)
        {
            return Err("dose proof normal arithmetic domain".into());
        }
        let mut targets = [
            [String::new(), String::new()],
            [String::new(), String::new()],
        ];
        for (side, (rate, attackers, defenders)) in
            [(c.red_rate, c.red, c.blue), (c.blue_rate, c.blue, c.red)]
                .into_iter()
                .enumerate()
        {
            let base = exact(rate, 16)?.times(exact(c.dt, 64)?);
            let (lo, hi) = match c.geometry {
                Geometry::AimedFire => (
                    base.times(Rational(1, defenders.into())),
                    base.times(Rational(attackers.into(), 1)),
                ),
                Geometry::DuelContact => (base, base),
            };
            if lo.less(minimum) {
                minimum = lo;
            }
            if maximum.less(hi) {
                maximum = hi;
            }
            targets[side] = [lo.wire(), hi.wire()];
        }
        let work = u64::from(c.blue + c.red) * c.max_steps;
        arms.push(ArmProof {
            arm_id: arm.arm_id.clone(),
            config_sha256: canonical_hash(&value["arms"][index]["config"])?,
            interior_counts: [c.blue, c.red],
            interior_target_states: 2 * u64::from(c.blue) * u64::from(c.red),
            targets,
            case_work: work,
            quantization_only_path_bound: Rational::new(work.into(), 1 << 53).wire(),
        });
    }
    let lower = minimum.times(Rational((1 << 50) - 1, 1 << 50));
    let upper = maximum.times(Rational((1 << 50) + 1, 1 << 50));
    let p_lower = Rational::new(lower.0, lower.0 + lower.1);
    let survival = Rational(1, 19683);
    let resolution = Rational(1, 1 << 53);
    if !upper.less(Rational(9, 1)) || !resolution.less(p_lower) || !resolution.less(survival) {
        return Err("dose proof resolution/saturation margin".into());
    }
    let work = arms
        .iter()
        .map(|a| a.case_work)
        .max()
        .ok_or("dose proof empty")?;
    Ok(DoseProof {
        schema: "war1-study-dose-proof-v1".into(),
        method: "monotone-interior-rectangle-gamma3-v1".into(),
        protocol_sha256: "a649e960ee574a2d4f5112b760f57701e22c2cbcf2e89d4915a31557ecb8348f".into(),
        proposal_sha256: wire::sha256(wire::PROPOSAL),
        approved_roster_sha256: canonical_hash(&value["arms"])?,
        manifest_sha256: manifest_sha256.into(),
        compiled_inputs_sha256: compiled_inputs_sha256.into(),
        interior_target_states: arms.iter().map(|a| a.interior_target_states).sum(),
        arms,
        positive_dose_extrema: [minimum.wire(), maximum.wire()],
        native_relative_error_bound: Rational(1, 1 << 50).wire(),
        native_dose_enclosure: [lower.wire(), upper.wire()],
        probability_lower_bound: p_lower.wire(),
        survival_lower_bound: survival.wire(),
        sampler_resolution: resolution.wire(),
        maximum_case_work: work,
        quantization_only_path_bound: Rational::new(work.into(), 1 << 53).wire(),
    })
}

/// Exhaustively check the actual native libm/sampler domain over 1,367,076
/// target states. This proves positivity/resolution/non-saturation only, not
/// general libm accuracy. Called at pure preparation and once at native gate.
pub fn validate_native_domain() -> Result<(), String> {
    let (_, arms) = approved()?;
    let mut checked = 0_u64;
    for arm in arms {
        let c = &arm.config;
        for blue in 1..=c.blue {
            for red in 1..=c.red {
                let hazards = match c.geometry {
                    Geometry::AimedFire => [
                        c.red_rate * f64::from(red) / f64::from(blue),
                        c.blue_rate * f64::from(blue) / f64::from(red),
                    ],
                    Geometry::DuelContact => [c.red_rate, c.blue_rate],
                };
                for hazard in hazards {
                    let p = math::probability(hazard, c.dt)
                        .map_err(|e| format!("dose proof {}: {}", e.field, e.detail))?;
                    if p.saturated || p.realized >= 1.0 || p.realized < 2.0_f64.powi(-53) {
                        return Err(format!(
                            "dose proof native sampler margin: {} ({blue},{red})",
                            arm.arm_id
                        ));
                    }
                    checked += 1;
                }
            }
        }
    }
    if checked != 1_367_076 {
        return Err("dose proof native coverage".into());
    }
    Ok(())
}
