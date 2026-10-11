//! Strict selected-manifest wire.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sugarscape_core::war::config::EngagementConfig;

pub const LIMIT: usize = 16 * 1024 * 1024;
pub const SELECTED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/war1_study_manifest.json"));
#[cfg(test)]
pub const FIXTURE: &[u8] = SELECTED;
pub const PROPOSAL: &[u8] =
    include_bytes!("../../../docs/superpowers/specs/2026-10-10-war-1-measurement-proposal.json");
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub purpose: String,
    pub protocol_sha256: String,
    pub proposal_sha256: String,
    pub roster_sha256: String,
    pub sampler: Sampler,
    #[serde(deserialize_with = "bounded_vec::<_,_,66>")]
    pub arms: Vec<Arm>,
    #[serde(deserialize_with = "bounded_vec::<_,_,52224>")]
    pub cases: Vec<Case>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sampler {
    pub method: String,
    pub bytes_drawn: u64,
    pub seed_bytes_sha256: String,
    #[serde(deserialize_with = "bounded_vec::<_,_,32>")]
    pub command: Vec<String>,
    #[serde(deserialize_with = "bounded_text::<_,128>")]
    pub version: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arm {
    #[serde(deserialize_with = "bounded_text::<_,192>")]
    pub arm_id: String,
    pub group: String,
    pub profile: String,
    pub scale: u32,
    pub mirror: bool,
    pub config: EngagementConfig,
    pub replicates: u32,
    #[serde(deserialize_with = "bounded_text::<_,192>")]
    pub seed_namespace: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub ordinal: u64,
    #[serde(deserialize_with = "bounded_text::<_,192>")]
    pub arm_id: String,
    pub replicate_index: u32,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub ordinal: u64,
    #[serde(deserialize_with = "bounded_text::<_,192>")]
    pub arm_id: String,
    pub replicate_index: u32,
    pub seed: u64,
}
impl Case {
    pub fn key(&self) -> Key {
        Key {
            ordinal: self.ordinal,
            arm_id: self.arm_id.clone(),
            replicate_index: self.replicate_index,
        }
    }
}
pub fn decode_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    if bytes.len() > LIMIT {
        return Err("manifest exceeds 16 MiB".into());
    }
    // Compute the raw identity before resolving anything. Byte equality rejects
    // duplicate keys, unknown fields, noncanonical types and every other mutation.
    let digest = sha256(bytes);
    if bytes != SELECTED {
        return Err(format!(
            "compiled selected manifest identity mismatch: {digest}"
        ));
    }
    let manifest: Manifest = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    validate_fields(&manifest, bytes)?;
    for arm in &manifest.arms {
        arm.config
            .validate()
            .map_err(|e| format!("invalid fixture config: {e:?}"))?;
    }
    Ok(manifest)
}

pub fn approved_oracle_ids() -> Result<Vec<String>, String> {
    let proposal: serde_json::Value =
        serde_json::from_slice(PROPOSAL).map_err(|e| e.to_string())?;
    let arms: Vec<Arm> =
        serde_json::from_value(proposal["arms"].clone()).map_err(|e| e.to_string())?;
    Ok(arms
        .into_iter()
        .filter(|a| a.group == "oracle")
        .map(|a| a.arm_id)
        .collect())
}

pub(super) fn validate_fields(m: &Manifest, raw: &[u8]) -> Result<(), String> {
    let fixture = m.purpose == "construction_fixture";
    if fixture && sha256(raw) != "12ad58fd65e353c1f09d02a6b7dea712bbf921ab4be100cf4f2274215c51074b"
    {
        return Err("fixed construction manifest bytes changed".into());
    }
    if m.schema != "war1-study-manifest-v1"
        || (!fixture && m.purpose != "prospective_seed_population")
        || m.protocol_sha256 != "a649e960ee574a2d4f5112b760f57701e22c2cbcf2e89d4915a31557ecb8348f"
        || m.proposal_sha256 != sha256(PROPOSAL)
    {
        return Err("manifest schema/purpose/approved binding mismatch".into());
    }
    let approved: serde_json::Value =
        serde_json::from_slice(PROPOSAL).map_err(|e| e.to_string())?;
    let arms: Vec<Arm> =
        serde_json::from_value(approved["arms"].clone()).map_err(|e| e.to_string())?;
    let expected = if fixture {
        let mut expected = vec![];
        for (group, profile, scale) in [
            ("oracle", "1v1", 1),
            ("oracle", "2v1", 1),
            ("core", "E", 64),
            ("scale", "C", 256),
        ] {
            for geometry in ["aimed_fire", "duel_contact"] {
                let id = format!("{group}.{profile}.N{scale}.{geometry}.dt1-64");
                let mut arm = arms
                    .iter()
                    .find(|a| a.arm_id == id)
                    .ok_or("approved fixture arm absent")?
                    .clone();
                arm.arm_id = format!("fixture.{profile}.N{scale}.{geometry}.dt1-64");
                arm.group = "construction_fixture".into();
                arm.replicates = 2;
                arm.seed_namespace = format!("construction_fixture/{}", arm.arm_id);
                expected.push(arm)
            }
        }
        expected
    } else {
        arms
    };
    if m.arms != expected {
        return Err("manifest arms differ from approved complete roster".into());
    }
    let value: serde_json::Value = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
    let mut canonical = serde_json::to_vec(&value["arms"]).map_err(|e| e.to_string())?;
    canonical.push(b'\n');
    if m.roster_sha256 != sha256(&canonical) {
        return Err("manifest actual roster hash mismatch".into());
    }
    let total: usize = expected.iter().map(|a| a.replicates as usize).sum();
    if m.cases.len() != total {
        return Err("manifest exact case census mismatch".into());
    }
    if m.sampler.method
        != if fixture {
            "construction-seeds-v1"
        } else {
            "os.urandom-u64le-v1"
        }
        || m.sampler.version != "1"
        || m.sampler.bytes_drawn != if fixture { 0 } else { total as u64 * 8 }
        || m.sampler.command.is_empty()
        || m.sampler.command.len() > 32
        || m.sampler
            .command
            .iter()
            .any(|s| s.is_empty() || s.len() > 1024)
        || fixture && m.sampler.command != ["construction-fixture-factory"]
    {
        return Err("sampler method/version/census/command mismatch".into());
    }
    let mut ordinal = 0;
    let mut seed_hash = Sha256::new();
    for arm in &expected {
        for index in 0..arm.replicates {
            let case = &m.cases[ordinal];
            if case.ordinal != ordinal as u64
                || case.arm_id != arm.arm_id
                || case.replicate_index != index
                || fixture && case.seed != [7, 8][index as usize]
            {
                return Err("manifest duplicate/wrong key, order or fixture seed".into());
            }
            if !fixture {
                seed_hash.update(case.seed.to_le_bytes())
            }
            ordinal += 1;
        }
    }
    if m.sampler.seed_bytes_sha256 != format!("{:x}", seed_hash.finalize()) {
        return Err("sampler seed transcript hash mismatch".into());
    }
    Ok(())
}

/// Refuse the (N+1)th value without allocating or deserializing its typed payload.
pub(super) fn bounded_vec<'de, D, T, const N: usize>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Visitor<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for Visitor<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "array with at most {N} entries")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut items = Vec::new();
            for _ in 0..N {
                match seq.next_element()? {
                    Some(item) => items.push(item),
                    None => return Ok(items),
                }
            }
            if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(serde::de::Error::custom("array bound exceeded"));
            }
            Ok(items)
        }
    }
    deserializer.deserialize_seq(Visitor::<T, N>(std::marker::PhantomData))
}
pub(super) fn bounded_text<'de, D, const N: usize>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor<const N: usize>;
    impl<const N: usize> serde::de::Visitor<'_> for Visitor<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "text within {N} UTF-8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > N {
                return Err(E::custom("text bound exceeded"));
            }
            Ok(s.to_owned())
        }
    }
    deserializer.deserialize_str(Visitor::<N>)
}

/// Typed serde decoding rejects malformed grammar/duplicates. This additional
/// lexical check rejects nonzero literals that binary64 would silently erase.
/// Quoted content (including exact PCG u128 state strings) is never converted.
pub(super) fn reject_numeric_underflow(bytes: &[u8]) -> Result<(), String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let mut i = 0;
    let mut quoted = false;
    while i < bytes.len() {
        if quoted {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == b'"' {
                quoted = false
            }
            i += 1;
            continue;
        }
        if bytes[i] == b'"' {
            quoted = true;
            i += 1;
            continue;
        }
        if bytes[i] == b'-' || bytes[i].is_ascii_digit() {
            let start = i;
            i += 1;
            while i < bytes.len()
                && matches!(bytes[i], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
            {
                i += 1
            }
            let literal = &text[start..i];
            let mantissa = literal.split(['e', 'E']).next().unwrap();
            if mantissa.bytes().any(|b| matches!(b, b'1'..=b'9'))
                && literal.parse::<f64>().is_ok_and(|value| value == 0.0)
            {
                return Err("nonzero JSON number underflows binary64".into());
            }
        } else {
            i += 1
        }
    }
    Ok(())
}
