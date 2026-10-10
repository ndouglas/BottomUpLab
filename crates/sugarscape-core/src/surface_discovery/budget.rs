use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchStats {
    pub candidate_worlds: u64,
    pub decision_states: u64,
    pub branches: u64,
    pub allocation_bytes: u64,
}

impl Limits {
    pub fn registered() -> Result<Self, Error> {
        // Compact reviewed registration is also consumed by the independent tests.
        // Decode only its limits here; diagnostic provenance validation is separate.
        #[derive(Deserialize)]
        struct Registration {
            registered_limits: Limits,
        }
        let registration: Registration =
            serde_json::from_str(include_str!("tests/fixtures/limits-reference.json"))
                .map_err(|e| Error::InvalidProtocol(format!("invalid limits registration: {e}")))?;
        registration.registered_limits.validate()?;
        Ok(registration.registered_limits)
    }
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.states == 0 || self.branches == 0 || self.allocation_bytes == 0 {
            return Err(Error::InvalidProtocol(
                "resource limits must be positive".into(),
            ));
        }
        Ok(())
    }
}

/// One compilation's accounting, carried forward when replay/panels are added.
/// Bytes conservatively cover retained cache nodes and a reserved, bounded
/// recursion/inference workspace, rather than cumulative temporary allocations.
#[derive(Clone)]
pub(crate) struct Budget {
    pub(crate) limits: Limits,
    pub(crate) stats: SearchStats,
    live_bytes: u64,
}
impl Budget {
    pub(crate) fn new(limits: &Limits) -> Result<Self, Error> {
        limits.validate()?;
        Ok(Self {
            limits: limits.clone(),
            stats: SearchStats::default(),
            live_bytes: 0,
        })
    }
    pub(crate) fn allocate(&mut self, bytes: u64) -> Result<(), Error> {
        let next = checked_limit(
            self.live_bytes,
            bytes,
            self.limits.allocation_bytes,
            "allocation_bytes",
        )?;
        self.live_bytes = next;
        self.stats.allocation_bytes = self.stats.allocation_bytes.max(next);
        Ok(())
    }
    pub(crate) fn release(&mut self, bytes: u64) -> Result<(), Error> {
        self.live_bytes = self
            .live_bytes
            .checked_sub(bytes)
            .ok_or(Error::ArithmeticOverflow)?;
        Ok(())
    }
    pub(crate) fn state(&mut self, bytes: u64) -> Result<(), Error> {
        let states = checked_limit(self.stats.decision_states, 1, self.limits.states, "states")?;
        self.allocate(bytes)?;
        self.stats.decision_states = states;
        Ok(())
    }
    pub(crate) fn branch(&mut self) -> Result<(), Error> {
        self.stats.branches =
            checked_limit(self.stats.branches, 1, self.limits.branches, "branches")?;
        Ok(())
    }
}
fn checked_limit(value: u64, increment: u64, limit: u64, resource: &str) -> Result<u64, Error> {
    let next = value
        .checked_add(increment)
        .ok_or(Error::ArithmeticOverflow)?;
    if next > limit {
        return Err(Error::ResourceLimit {
            resource: resource.into(),
            limit,
        });
    }
    Ok(next)
}
