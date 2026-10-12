use super::{Clock, Error, Profile};
use serde::Serialize;
wire_object!(Limits { pub profiles:u64, pub transitions:u64, pub allocation_bytes:u64 });
wire_object!(CompileStats { pub profiles:u64, pub transitions:u64, pub peak_bytes:u64 });
wire_object!(PeakWitness {pub phase:String,pub profile:Option<Profile>,pub clock:Option<Clock>,pub retained_rows:u64,pub pure_profiles:u64,pub scratch_bytes:u64,pub live_bytes:u64});
impl Limits {
    pub fn registered() -> Result<Self, Error> {
        Self::from_reference(include_str!("tests/fixtures/limits-reference.json"))
    }
    pub(super) fn from_reference(s: &str) -> Result<Self, Error> {
        #[derive(Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Panel {
            stratum: super::Stratum,
            stats: CompileStats,
            peak: PeakWitness,
            boundary_receipt: serde_json::Value,
        }
        #[derive(Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Reference {
            version: String,
            inputs: serde_json::Value,
            panels: Vec<Panel>,
            limits: Limits,
            independent_artifacts: Vec<serde_json::Value>,
        }
        let mut deserializer = serde_json::Deserializer::from_str(s);
        let value = super::validation::strict_value(&mut deserializer)
            .map_err(|e| Error::InvalidReport(e.to_string()))?;
        deserializer
            .end()
            .map_err(|e| Error::InvalidReport(e.to_string()))?;
        let reference: Reference = super::validation::decode::<Reference, serde_json::Error>(value)
            .map_err(|e| Error::InvalidReport(e.to_string()))?;
        if reference.version != "key-delegation-limits-reference-v1"
            || reference.panels.len() != 36
            || reference.independent_artifacts.is_empty()
        {
            return Err(Error::InvalidReport(
                "limits reference version, panels or artifacts".into(),
            ));
        }
        let limits = reference.limits;
        let mut maxima = [0; 3];
        for (panel, stratum) in reference.panels.iter().zip(super::game::strata()) {
            if panel.stratum != stratum {
                return Err(Error::InvalidReport("limits stratum order".into()));
            }
            if panel.stats.profiles != 46656 || panel.peak.live_bytes != panel.stats.peak_bytes {
                return Err(Error::InvalidReport(
                    "limits profile count or peak witness".into(),
                ));
            }
            for (i, n) in [
                panel.stats.profiles,
                panel.stats.transitions,
                panel.stats.peak_bytes,
            ]
            .into_iter()
            .enumerate()
            {
                if n == 0 {
                    return Err(Error::InvalidReport("limits stats".into()));
                }
                maxima[i] = maxima[i].max(n);
            }
        }
        let expected = Self {
            profiles: mul(maxima[0], 2)?,
            transitions: mul(maxima[1], 2)?,
            allocation_bytes: mul(maxima[2], 2)?,
        };
        if limits != expected {
            return Err(Error::InvalidReport(
                "registered limits do not equal double maxima".into(),
            ));
        }
        Ok(limits)
    }
}
pub(super) fn add(a: u64, b: u64) -> Result<u64, Error> {
    a.checked_add(b).ok_or(Error::ArithmeticOverflow)
}
pub(super) fn mul(a: u64, b: u64) -> Result<u64, Error> {
    a.checked_mul(b).ok_or(Error::ArithmeticOverflow)
}
/// Streaming length of compact JSON is independent of object key order. No payload buffer is allocated.
pub(super) fn json_len(v: &impl Serialize) -> Result<u64, Error> {
    struct Counter(u64);
    impl std::io::Write for Counter {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len() as u64)
                .ok_or_else(|| std::io::Error::other("length overflow"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut c = Counter(0);
    serde_json::to_writer(&mut c, v).map_err(|e| Error::InvalidReport(e.to_string()))?;
    Ok(c.0)
}
pub(super) fn scratch(n: u64) -> Result<u64, Error> {
    #[derive(Serialize)]
    struct Receipt<'a> {
        path: &'a str,
        bytes: u64,
        sha256: &'a str,
    }
    let r = [Receipt {
        path: "key-delegation-trace-v1",
        bytes: n,
        sha256: "0000000000000000000000000000000000000000000000000000000000000000",
    }];
    add(add(mul(2, n)?, json_len(&r)?)?, 64)
}
pub(super) const FIXED: u64 = 2 * (64 + 729) + 9;
pub(super) struct Budget<'a> {
    pub limits: &'a Limits,
    pub stats: CompileStats,
    pub retained: u64,
    pub peak: Option<PeakWitness>,
}
impl<'a> Budget<'a> {
    pub fn new(limits: &'a Limits) -> Result<Self, Error> {
        let mut b = Self {
            limits,
            stats: CompileStats {
                profiles: 0,
                transitions: 0,
                peak_bytes: 0,
            },
            retained: 0,
            peak: None,
        };
        b.bytes(FIXED)?;
        Ok(b)
    }
    fn check(value: u64, limit: u64, resource: &str) -> Result<(), Error> {
        if value > limit {
            Err(Error::ResourceLimit {
                resource: resource.into(),
                limit,
            })
        } else {
            Ok(())
        }
    }
    pub fn profile(&mut self) -> Result<(), Error> {
        let n = add(self.stats.profiles, 1)?;
        Self::check(n, self.limits.profiles, "profiles")?;
        self.stats.profiles = n;
        Ok(())
    }
    pub fn boundary(&mut self, n: u64, events: u64) -> Result<u64, Error> {
        let t = add(self.stats.transitions, events)?;
        Self::check(t, self.limits.transitions, "transitions")?;
        let s = scratch(n)?;
        self.bytes(add(add(FIXED, self.retained)?, s)?)?;
        self.stats.transitions = t;
        Ok(s)
    }
    pub fn bytes(&mut self, n: u64) -> Result<(), Error> {
        Self::check(n, self.limits.allocation_bytes, "allocation_bytes")?;
        self.stats.peak_bytes = self.stats.peak_bytes.max(n);
        Ok(())
    }
    pub fn row(&mut self, n: u64, s: u64) -> Result<(), Error> {
        let next = add(self.retained, add(n, 4)?)?;
        self.bytes(add(add(FIXED, next)?, s)?)?;
        self.retained = next;
        Ok(())
    }
    pub fn witness(&mut self, w: PeakWitness) {
        if self
            .peak
            .as_ref()
            .is_none_or(|p| w.live_bytes > p.live_bytes)
        {
            self.peak = Some(w);
        }
    }
}
