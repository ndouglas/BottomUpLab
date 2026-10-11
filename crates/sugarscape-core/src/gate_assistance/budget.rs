//! Portable logical live payload, independently of allocator capacity or RSS.
use super::{canonical::json_len, types::wire_struct, *};
use serde::Serialize;

wire_struct!(Limits {
    profiles: u64,
    transitions: u64,
    allocation_bytes: u64
});
wire_struct!(CompileStats {
    profiles: u64,
    transitions: u64,
    peak_bytes: u64
});
impl Limits {
    pub fn registered() -> Result<Self, Error> {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("tests/fixtures/limits-reference.json"))
                .map_err(|e| Error::InvalidReport(format!("registered limits: {e}")))?;
        if fixture["finalization"]["status"] != "independently_approved" {
            return Err(Error::InvalidReport("unapproved limits".into()));
        }
        let panels = fixture["panels"]
            .as_array()
            .ok_or_else(|| Error::InvalidReport("missing limit panels".into()))?;
        if panels.len() != 18 {
            return Err(Error::InvalidReport("limit panel count".into()));
        }
        let mut maxima = [0; 3];
        for panel in panels {
            for (i, key) in ["profiles", "transitions", "peak_bytes"].iter().enumerate() {
                maxima[i] = maxima[i].max(
                    panel["stats"][key]
                        .as_u64()
                        .ok_or_else(|| Error::InvalidReport("invalid limit counter".into()))?,
                );
            }
        }
        let expected = Self {
            profiles: mul(maxima[0], 2)?,
            transitions: mul(maxima[1], 2)?,
            allocation_bytes: mul(maxima[2], 2)?,
        };
        let declared: Self = serde_json::from_value(fixture["provisional_limits"].clone())
            .map_err(|e| Error::InvalidReport(e.to_string()))?;
        if declared != expected {
            return Err(Error::InvalidReport("limit derivation mismatch".into()));
        }
        Ok(expected)
    }
}
pub(super) fn add(a: u64, b: u64) -> Result<u64, Error> {
    a.checked_add(b).ok_or(Error::ArithmeticOverflow)
}
fn mul(a: u64, b: u64) -> Result<u64, Error> {
    a.checked_mul(b).ok_or(Error::ArithmeticOverflow)
}
pub(super) struct Budget {
    limits: Limits,
    pub(super) stats: CompileStats,
    retained: u64,
    scratch: u64,
}
impl Budget {
    pub(super) fn new(limits: &Limits) -> Result<Self, Error> {
        let mut budget = Self {
            limits: limits.clone(),
            stats: CompileStats {
                profiles: 0,
                transitions: 0,
                peak_bytes: 0,
            },
            retained: 0,
            scratch: 0,
        };
        budget.reserve(0, 0, 1595, 0)?; // 2*(64+729) response bytes + three raw profiles.
        Ok(budget)
    }
    // Every candidate counter is checked before any state is changed.
    pub(super) fn reserve(
        &mut self,
        profiles: u64,
        transitions: u64,
        retained: u64,
        scratch: u64,
    ) -> Result<(), Error> {
        let p = add(self.stats.profiles, profiles)?;
        let t = add(self.stats.transitions, transitions)?;
        let r = add(self.retained, retained)?;
        let live = add(r, scratch)?;
        for (resource, value, limit) in [
            ("profiles", p, self.limits.profiles),
            ("transitions", t, self.limits.transitions),
            ("allocation_bytes", live, self.limits.allocation_bytes),
        ] {
            if value > limit {
                return Err(Error::ResourceLimit {
                    resource: resource.into(),
                    limit,
                });
            }
        }
        self.stats.profiles = p;
        self.stats.transitions = t;
        self.stats.peak_bytes = self.stats.peak_bytes.max(live);
        self.retained = r;
        self.scratch = scratch;
        Ok(())
    }
    pub(super) fn begin_profile(&mut self) -> Result<(), Error> {
        self.reserve(1, 0, 0, 0)
    }
    pub(super) fn retain(&mut self, n: u64) -> Result<(), Error> {
        self.reserve(0, 0, n, self.scratch)
    }
    pub(super) fn release_episode(&mut self) {
        self.scratch = 0;
    }
}

// Keep lengths of the append-only arrays. CountWriter never allocates canonical
// prefixes or clones histories. Field ordering changes bytes, but never length.
pub(super) struct EpisodeCharge<'a> {
    pub(super) budget: &'a mut Budget,
    base: u64,
    event_bytes: u64,
    history_bytes: [u64; 2],
    event_count: u64,
    history_counts: [u64; 2],
}
impl<'a> EpisodeCharge<'a> {
    pub(super) fn new(budget: &'a mut Budget) -> Self {
        Self {
            budget,
            base: 0,
            event_bytes: 0,
            history_bytes: [0; 2],
            event_count: 0,
            history_counts: [0; 2],
        }
    }
    pub(super) fn initial(
        &mut self,
        config: &Config,
        profile: Profile,
        histories: &[Vec<LocalEntry>; 2],
        outcome: &Outcome,
    ) -> Result<(), Error> {
        #[derive(Serialize)]
        struct Empty<'a> {
            version: u16,
            config: &'a Config,
            profile: Profile,
            events: [Event; 0],
            histories: [Vec<LocalEntry>; 2],
            outcome: &'a Outcome,
        }
        self.base = json_len(&Empty {
            version: 1,
            config,
            profile,
            events: [],
            histories: [vec![], vec![]],
            outcome,
        })?
        .checked_sub(json_len(outcome)?)
        .ok_or(Error::ArithmeticOverflow)?;
        self.boundary(None, histories, outcome)
    }
    pub(super) fn boundary(
        &mut self,
        event: Option<&Event>,
        entries: &[Vec<LocalEntry>; 2],
        outcome: &Outcome,
    ) -> Result<(), Error> {
        let mut eb = self.event_bytes;
        let mut ec = self.event_count;
        if let Some(event) = event {
            eb = add(eb, add(json_len(event)?, u64::from(ec > 0))?)?;
            ec = add(ec, 1)?;
        }
        let mut hb = self.history_bytes;
        let mut hc = self.history_counts;
        for i in 0..2 {
            for entry in &entries[i] {
                hb[i] = add(hb[i], add(json_len(entry)?, u64::from(hc[i] > 0))?)?;
                hc[i] = add(hc[i], 1)?;
            }
        }
        let length = add(
            add(add(add(self.base, eb)?, hb[0])?, hb[1])?,
            json_len(outcome)?,
        )?;
        #[derive(Serialize)]
        struct Receipt<'a> {
            path: &'a str,
            bytes: u64,
            sha256: &'a str,
        }
        let receipt = json_len(&[Receipt {
            path: "gate-assistance-trace-v1",
            bytes: length,
            sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        }])?;
        let scratch = add(add(mul(length, 2)?, receipt)?, 64)?;
        self.budget
            .reserve(0, u64::from(event.is_some()), 0, scratch)?;
        self.event_bytes = eb;
        self.event_count = ec;
        self.history_bytes = hb;
        self.history_counts = hc;
        Ok(())
    }
}
