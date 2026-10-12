//! Strict decoding of the three independently sealed reference fixtures.
use super::*;
wire_object!(GoverningInputs {pub protocol:Receipt,pub design:Receipt,pub interfaces:Receipt});
wire_object!(KernelCase {pub name:String,pub config:Config,pub profile:Profile,pub episode:Episode,pub trace_hash:String,pub boundaries:Vec<BoundaryWitness>});
wire_object!(BoundaryWitness {pub clock:Clock,pub event_count:u8,pub history_lengths:[u16;2],pub views:[View;2],pub physical:PhysicalState,pub scratch_bytes:u64});
wire_object!(KernelReference {pub version:String,pub inputs:GoverningInputs,pub cases:Vec<KernelCase>,pub independent_artifacts:Vec<Receipt>});
wire_object!(GameReferencePanel {pub stratum:Stratum,pub row_receipt:Receipt,pub row_identity:String,pub analysis:Analysis,pub controls:Vec<ControlRecord>,pub stats:CompileStats});
wire_object!(GameReference {pub version:String,pub inputs:GoverningInputs,pub panels:Vec<GameReferencePanel>,pub contrasts:Vec<Contrast>,pub independent_artifacts:Vec<Receipt>});
wire_object!(LimitPanel {pub stratum:Stratum,pub stats:CompileStats,pub peak:PeakWitness,pub boundary_receipt:Receipt});
wire_object!(LimitsReference {pub version:String,pub inputs:GoverningInputs,pub panels:Vec<LimitPanel>,pub limits:Limits,pub independent_artifacts:Vec<Receipt>});
pub(super) struct References {
    pub kernel: KernelReference,
    pub game: GameReference,
    pub limits: LimitsReference,
}
fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(bytes)
        .map_err(|e| Error::InvalidReport(format!("sealed reference: {e}")))
}
fn artifacts(items: &[Receipt]) -> Result<(), Error> {
    if items.is_empty() || items.windows(2).any(|p| p[0].path >= p[1].path) {
        return Err(Error::InvalidReport("reference artifact order".into()));
    }
    super::source_inputs::identity(items)?;
    Ok(())
}
pub(super) fn load() -> Result<References, Error> {
    let r = References {
        kernel: parse(REFERENCE_INPUTS[0].1)?,
        game: parse(REFERENCE_INPUTS[1].1)?,
        limits: parse(REFERENCE_INPUTS[2].1)?,
    };
    if r.kernel.version != "key-delegation-kernel-reference-v1"
        || r.game.version != "key-delegation-game-reference-v1"
        || r.limits.version != "key-delegation-limits-reference-v1"
        || r.game.panels.len() != 36
        || r.limits.panels.len() != 36
        || r.game.contrasts.len() != 84
        || r.kernel.cases.is_empty()
        || r.kernel.inputs != r.game.inputs
        || r.kernel.inputs != r.limits.inputs
        || r.limits.limits != Limits::registered()?
    {
        return Err(Error::InvalidReport("sealed reference registration".into()));
    }
    for inputs in [&r.kernel.inputs, &r.game.inputs, &r.limits.inputs] {
        for receipt in [&inputs.protocol, &inputs.design, &inputs.interfaces] {
            receipt.validate()?;
        }
    }
    for a in [
        &r.kernel.independent_artifacts,
        &r.game.independent_artifacts,
        &r.limits.independent_artifacts,
    ] {
        artifacts(a)?;
    }
    for ((g, l), s) in r.game.panels.iter().zip(&r.limits.panels).zip(strata()) {
        if g.stratum != s
            || l.stratum != s
            || g.stats != l.stats
            || l.peak.live_bytes != l.stats.peak_bytes
        {
            return Err(Error::InvalidReport(
                "reference panel order or accounting".into(),
            ));
        }
        g.row_receipt.validate()?;
        l.boundary_receipt.validate()?;
    }
    Ok(r)
}
