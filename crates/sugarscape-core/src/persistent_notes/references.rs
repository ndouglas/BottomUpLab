//! Sealed raw-byte fixtures. Engineering reconstruction never evaluates an episode.
#[cfg(test)]
use super::{diagnostics::Array, Evaluation};
use super::{
    source_inputs::{Receipt, REFERENCE_INPUTS},
    Counters, Error, Limits,
};
use serde::{
    de::{DeserializeSeed, Error as _, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackedSize {
    name: String,
    bytes: u64,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitsPayload {
    limits: Limits,
    counters: Counters,
    packed_sizes: Vec<PackedSize>,
    canonical_maxima: Vec<PackedSize>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitsFixture {
    protocol_version: u64,
    kind: String,
    author_receipt: Receipt,
    reviewer_receipt: Receipt,
    complete_reference_receipt: Receipt,
    payload: LimitsPayload,
}
fn parse_limits_fixture() -> Result<LimitsFixture, Error> {
    let bytes = REFERENCE_INPUTS[2].1;
    if bytes.len() > 65536 {
        return Err(Error::ResourceLimit {
            resource: "reference_header".into(),
            limit: 65536,
        });
    }
    let fixture: LimitsFixture =
        serde_json::from_slice(bytes).map_err(|e| Error::InvalidReport(e.to_string()))?;
    if fixture.protocol_version != 1
        || fixture.kind != "limits"
        || fixture.payload.limits != Limits::default()
    {
        return Err(Error::InvalidReport("sealed limits header".into()));
    }
    for sizes in [
        &fixture.payload.packed_sizes,
        &fixture.payload.canonical_maxima,
    ] {
        if sizes.windows(2).any(|w| w[0].name >= w[1].name) || sizes.iter().any(|s| s.bytes == 0) {
            return Err(Error::InvalidReport("sealed size table".into()));
        }
    }
    // These are original pre-metadata protocol-author/reviewer raw receipts, preserved verbatim.
    if fixture.author_receipt.bytes == 0
        || fixture.reviewer_receipt.bytes == 0
        || fixture.complete_reference_receipt.bytes == 0
    {
        return Err(Error::InvalidReport("derivation receipt".into()));
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Header {
        protocol_version: u64,
        kind: String,
        author_receipt: Receipt,
        reviewer_receipt: Receipt,
        complete_reference_receipt: Receipt,
        payload: serde::de::IgnoredAny,
    }
    for (index, kind) in [(0, "evaluation"), (1, "kernel")] {
        let header: Header = serde_json::from_slice(REFERENCE_INPUTS[index].1)
            .map_err(|e| Error::InvalidReport(e.to_string()))?;
        let _ = header.payload;
        if header.protocol_version != 1
            || header.kind != kind
            || header.author_receipt != fixture.author_receipt
            || header.reviewer_receipt != fixture.reviewer_receipt
            || header.complete_reference_receipt != fixture.complete_reference_receipt
        {
            return Err(Error::InvalidReport(
                "inconsistent sealed derivation receipts".into(),
            ));
        }
    }
    Ok(fixture)
}
fn limits_fixture() -> Result<LimitsFixture, Error> {
    static VALUE: std::sync::OnceLock<Result<LimitsFixture, Error>> = std::sync::OnceLock::new();
    VALUE.get_or_init(parse_limits_fixture).clone()
}
impl Limits {
    pub fn registered() -> Result<Self, Error> {
        Ok(limits_fixture()?.payload.limits)
    }
}
pub(crate) fn counters() -> Result<Counters, Error> {
    Ok(limits_fixture()?.payload.counters)
}
#[cfg(test)]
struct Payload;
#[cfg(test)]
impl<'de> DeserializeSeed<'de> for Payload {
    type Value = Evaluation;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Evaluation, D::Error> {
        d.deserialize_map(self)
    }
}
#[cfg(test)]
impl<'de> Visitor<'de> for Payload {
    type Value = Evaluation;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("sealed complete evaluation")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Evaluation, A::Error> {
        let (mut rows, mut means, mut request_contrasts, mut contrasts, mut envelopes) =
            (None, None, None, None, None);
        let mut seen = std::collections::BTreeSet::new();
        while let Some(k) = a.next_key::<String>()? {
            if !seen.insert(k.clone()) {
                return Err(A::Error::custom("duplicate reference field"));
            }
            match k.as_str() {
                "rows" => rows = Some(a.next_value_seed(Array::new(4608, 98766))?),
                "means" => means = Some(a.next_value_seed(Array::new(2304, 1325))?),
                "request_contrasts" => {
                    request_contrasts = Some(a.next_value_seed(Array::new(4608, 1342))?)
                }
                "contrasts" => contrasts = Some(a.next_value_seed(Array::new(2304, 1334))?),
                "envelopes" => envelopes = Some(a.next_value_seed(Array::new(576, 141))?),
                _ => return Err(A::Error::custom("unknown reference field")),
            }
        }
        macro_rules! required {
            ($n:ident) => {
                $n.ok_or_else(|| A::Error::missing_field(stringify!($n)))?
            };
        }
        Ok(Evaluation {
            rows: required!(rows),
            means: required!(means),
            request_contrasts: required!(request_contrasts),
            contrasts: required!(contrasts),
            envelopes: required!(envelopes),
            counters: counters().map_err(A::Error::custom)?,
        })
    }
}
#[cfg(test)]
pub(crate) fn engineering_evaluation() -> Result<Evaluation, Error> {
    struct Fixture;
    impl<'de> DeserializeSeed<'de> for Fixture {
        type Value = Evaluation;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Evaluation, D::Error> {
            d.deserialize_map(self)
        }
    }
    impl<'de> Visitor<'de> for Fixture {
        type Value = Evaluation;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("sealed fixture container")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Evaluation, A::Error> {
            let mut payload = None;
            let mut receipt = None;
            let mut seen = std::collections::BTreeSet::new();
            while let Some(k) = a.next_key::<String>()? {
                if !seen.insert(k.clone()) {
                    return Err(A::Error::custom("duplicate fixture field"));
                }
                match k.as_str() {
                    "payload" => payload = Some(a.next_value_seed(Payload)?),
                    "complete_reference_receipt" => receipt = Some(a.next_value::<Receipt>()?),
                    "protocol_version" => {
                        if a.next_value::<u64>()? != 1 {
                            return Err(A::Error::custom("fixture protocol"));
                        }
                    }
                    "kind" => {
                        if a.next_value::<String>()? != "evaluation" {
                            return Err(A::Error::custom("fixture kind"));
                        }
                    }
                    "author_receipt" | "reviewer_receipt" => {
                        a.next_value::<Receipt>()?;
                    }
                    _ => return Err(A::Error::custom("unknown fixture field")),
                }
            }
            if seen.len() != 6
                || receipt
                    != Some(
                        limits_fixture()
                            .map_err(A::Error::custom)?
                            .complete_reference_receipt,
                    )
            {
                return Err(A::Error::custom("fixture receipt mismatch"));
            }
            payload.ok_or_else(|| A::Error::missing_field("payload"))
        }
    }
    let mut d = serde_json::Deserializer::from_slice(REFERENCE_INPUTS[0].1);
    let evaluation = Fixture
        .deserialize(&mut d)
        .map_err(|e| Error::InvalidReport(e.to_string()))?;
    d.end().map_err(|e| Error::InvalidReport(e.to_string()))?;
    Ok(evaluation)
}
pub(super) struct Compare<'a, T>(pub(super) &'a [T], pub(super) u64);
impl<'de, T: serde::de::DeserializeOwned + serde::Serialize + PartialEq> DeserializeSeed<'de>
    for Compare<'_, T>
{
    type Value = ();
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        d.deserialize_seq(self)
    }
}
impl<'de, T: serde::de::DeserializeOwned + serde::Serialize + PartialEq> Visitor<'de>
    for Compare<'_, T>
{
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("complete saved/reference equality")
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
        for expected in self.0 {
            let actual = a
                .next_element_seed(super::diagnostics::Record::<T>::new(self.1))?
                .ok_or_else(|| A::Error::custom("missing reference row"))?;
            if &actual != expected {
                return Err(A::Error::custom("saved/reference mismatch"));
            }
        }
        if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(A::Error::custom("extra reference row"));
        }
        Ok(())
    }
}
pub(crate) fn compare(report: &super::Report) -> Result<(), Error> {
    struct Check<'a>(&'a super::Report, bool);
    impl<'de> DeserializeSeed<'de> for Check<'_> {
        type Value = ();
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
            d.deserialize_map(self)
        }
    }
    impl<'de> Visitor<'de> for Check<'_> {
        type Value = ();
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("sealed reference comparison")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
            let mut seen = std::collections::BTreeSet::new();
            while let Some(k) = a.next_key::<String>()? {
                if !seen.insert(k.clone()) {
                    return Err(A::Error::custom("duplicate reference field"));
                }
                if self.1 {
                    match k.as_str() {
                        "rows" => a.next_value_seed(Compare(&self.0.rows, 98766))?,
                        "means" => a.next_value_seed(Compare(&self.0.means, 1325))?,
                        "request_contrasts" => {
                            a.next_value_seed(Compare(&self.0.request_contrasts, 1342))?
                        }
                        "contrasts" => a.next_value_seed(Compare(&self.0.contrasts, 1334))?,
                        "envelopes" => a.next_value_seed(Compare(&self.0.envelopes, 141))?,
                        _ => return Err(A::Error::custom("unknown reference array")),
                    }
                } else if k == "payload" {
                    a.next_value_seed(Check(self.0, true))?;
                } else {
                    a.next_value::<serde::de::IgnoredAny>()?;
                }
            }
            if seen.len() != if self.1 { 5 } else { 6 } {
                return Err(A::Error::custom("reference completeness"));
            }
            Ok(())
        }
    }
    let mut d = serde_json::Deserializer::from_slice(REFERENCE_INPUTS[0].1);
    Check(report, false)
        .deserialize(&mut d)
        .map_err(|e| Error::InvalidReport(e.to_string()))?;
    d.end().map_err(|e| Error::InvalidReport(e.to_string()))
}
