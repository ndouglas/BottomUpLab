//! Typed reproducibility gate. Receipt syntax never substitutes for owner authorization.
use super::{io, source, wire};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CampaignBinding {
    pub manifest_sha256: String,
    pub compiled_inputs_sha256: String,
    pub executable_sha256: String,
    #[serde(deserialize_with = "wire::required_nullable")]
    pub declaration_sha256: Option<String>,
    #[serde(deserialize_with = "wire::required_nullable")]
    pub acceptance_sha256: Option<String>,
    #[serde(deserialize_with = "wire::required_nullable")]
    pub execution_approval_sha256: Option<String>,
}
pub struct AuthorizedCampaign {
    manifest: wire::Manifest,
    binding: CampaignBinding,
    retained: Vec<(String, Artifact)>,
}
impl AuthorizedCampaign {
    pub(super) fn parts(&self) -> (&wire::Manifest, &CampaignBinding) {
        (&self.manifest, &self.binding)
    }
    pub(super) fn retained(&self) -> &[(String, Artifact)] {
        &self.retained
    }
    pub(super) fn permits(&self, case: &wire::Case, arm: &wire::Arm) -> bool {
        self.binding.execution_approval_sha256.is_some()
            && self.manifest.purpose == "prospective_seed_population"
            && self
                .manifest
                .cases
                .get(case.ordinal as usize)
                .is_some_and(|c| {
                    c.arm_id == case.arm_id
                        && c.seed == case.seed
                        && c.replicate_index == case.replicate_index
                })
            && self
                .manifest
                .arms
                .iter()
                .any(|a| a == arm && arm.arm_id == case.arm_id)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInventory {
    pub schema: String,
    pub compiled_inputs_sha256: String,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,4096>")]
    pub entries: Vec<Artifact>,
}
impl SourceInventory {
    pub(super) fn validate(&self, expected: &str) -> Result<(), String> {
        if self.schema != "war1-study-source-inventory-v1"
            || self.compiled_inputs_sha256 != expected
            || self.entries.is_empty()
            || self.entries.len() > source::inputs::MAX_ENTRIES
            || self
                .entries
                .iter()
                .try_fold(0u64, |sum, a| sum.checked_add(a.bytes))
                .is_none_or(|sum| sum > source::inputs::MAX_SOURCE_BYTES as u64)
            || self.entries.windows(2).any(|w| w[0].path >= w[1].path)
            || self.entries.iter().any(|a| {
                !source::inputs::valid_relative(&a.path)
                    || !hash(&a.sha256)
                    || a.bytes > io::METADATA_LIMIT as u64
            })
        {
            return Err("invalid source inventory schema/digest/ordered entries".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Identities {
    pub protocol_sha256: String,
    pub proposal_sha256: String,
    pub manifest_sha256: String,
    pub compiled_inputs_sha256: String,
    pub source_inventory_sha256: String,
    pub resolved_roster_sha256: String,
    pub binary_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub workers: u64,
    pub rss_bytes: u64,
    pub wall_seconds: u64,
    pub campaign_bytes: u64,
    pub compact_row_bytes: u64,
    pub start_row_bytes: u64,
    pub audit_frame_bytes: u64,
    pub metadata_bytes: u64,
}
impl Limits {
    pub fn required() -> Self {
        Self {
            workers: 1,
            rss_bytes: 2 * 1024 * 1024 * 1024,
            wall_seconds: 12 * 60 * 60,
            campaign_bytes: io::CAMPAIGN_LIMIT,
            compact_row_bytes: 32768,
            start_row_bytes: 1024,
            audit_frame_bytes: 32768,
            metadata_bytes: io::METADATA_LIMIT as u64,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Population {
    pub arms: u64,
    pub cases: u64,
    pub settlements: u64,
    pub initial_actor_steps: u64,
    pub maximum_population: u64,
    pub maximum_case_work: u64,
}
impl Population {
    pub fn required() -> Self {
        Self {
            arms: 66,
            cases: 52224,
            settlements: 17563648,
            initial_actor_steps: 2054160384,
            maximum_population: 768,
            maximum_case_work: 393216,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub schema: String,
    pub purpose: String,
    pub identities: Identities,
    pub build: Artifact,
    pub numeric: Artifact,
    pub resources: Artifact,
    pub source_inventory: Artifact,
    pub manifest: Artifact,
    pub binary: Artifact,
    pub normative_frozen_revision: String,
    pub declaration_revision: String,
    pub registered_histories: u64,
    pub census: Population,
    pub budgets: Limits,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acceptance {
    pub schema: String,
    pub declaration_sha256: String,
    pub reviewer_identity: String,
    pub reviewer_role: String,
    pub decision: String,
    pub registered_histories: u64,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,32>")]
    pub open_required_findings: Vec<String>,
    pub build_sha256: String,
    pub numeric_sha256: String,
    pub resources_sha256: String,
    pub binary_sha256: String,
    pub manifest_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Execution {
    pub schema: String,
    pub declaration_sha256: String,
    pub acceptance_sha256: String,
    pub owner_identity: String,
    pub action: String,
    pub census: Population,
    pub budgets: Limits,
    pub authorization_quote: String,
    pub authorized_at: String,
    pub compiled_inputs_sha256: String,
    pub source_inventory_sha256: String,
    pub binary_sha256: String,
    pub manifest_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildReceipt {
    pub schema: String,
    pub identities: Identities,
    pub selected_path: String,
    pub selected_purpose: String,
    pub binary: Artifact,
    pub rustc: String,
    pub cargo: String,
    pub platform: String,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,128>")]
    pub features: Vec<String>,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,16>")]
    pub flags: Vec<[String; 2]>,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,32>")]
    pub build_command: Vec<String>,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,4096>")]
    pub inventory_before: Vec<Artifact>,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,4096>")]
    pub inventory_after: Vec<Artifact>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericReceipt {
    pub schema: String,
    pub manifest_sha256: String,
    pub compiled_inputs_sha256: String,
    pub binary_sha256: String,
    pub numeric_export: Artifact,
    pub oracle_packet: Artifact,
    pub certificate_valid: bool,
    pub mass_valid: bool,
    pub width_valid: bool,
    pub ulp_valid: bool,
    pub ctmc_valid: bool,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,32>")]
    pub unresolved_findings: Vec<String>,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,12>")]
    pub oracle_ids: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub artifact: Artifact,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,32>")]
    pub command: Vec<String>,
    pub compiled_inputs_sha256: String,
    pub binary_sha256: String,
    pub wall_seconds: f64,
    pub peak_rss_bytes: u64,
    pub fixture_purpose: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceReceipt {
    pub schema: String,
    pub manifest_sha256: String,
    pub compiled_inputs_sha256: String,
    pub binary_sha256: String,
    pub budgets: Limits,
    pub census: Population,
    pub reserved_bytes: u64,
    pub failure_reserve_bytes: u64,
    pub schema_bounds_valid: bool,
    pub aggregate_proof_valid: bool,
    pub forecast_valid: bool,
    pub forecast_method: String,
    pub forecast_inputs: Artifact,
    pub forecast_wall_seconds: f64,
    pub forecast_peak_rss_bytes: u64,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,32>")]
    pub fixture_probes: Vec<Probe>,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,32>")]
    pub unavailable: Vec<String>,
}
fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<(T, Vec<u8>), String> {
    let bytes = io::read_input(path)?;
    wire::reject_numeric_underflow(&bytes)?;
    let value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    Ok((value, bytes))
}
fn artifact(a: &Artifact) -> Result<Vec<u8>, String> {
    if !hash(&a.sha256) || a.bytes > io::METADATA_LIMIT as u64 {
        return Err("invalid artifact descriptor".into());
    }
    let bytes = io::read_input(Path::new(&a.path))?;
    if bytes.len() as u64 != a.bytes || wire::sha256(&bytes) != a.sha256 {
        return Err("artifact bytes/hash mismatch".into());
    }
    Ok(bytes)
}
fn decode<T: serde::de::DeserializeOwned>(a: &Artifact) -> Result<T, String> {
    let bytes = artifact(a)?;
    wire::reject_numeric_underflow(&bytes)?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
fn hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn text(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 192
}
fn command(v: &[String]) -> bool {
    !v.is_empty() && v.len() <= 32 && v.iter().all(|s| !s.is_empty() && s.len() <= 1024)
}
pub fn preflight(
    declaration: &Path,
    acceptance: &Path,
    execution: &Path,
) -> Result<AuthorizedCampaign, String> {
    // Context creation is deferred until after the mandatory execution read.
    preflight_with_context(
        declaration,
        acceptance,
        execution,
        source::VerificationContext::compiled,
        "prospective_seed_population",
    )
}
fn preflight_with_context(
    declaration: &Path,
    acceptance: &Path,
    execution: &Path,
    context: impl FnOnce() -> Result<source::VerificationContext, String>,
    required_purpose: &str,
) -> Result<AuthorizedCampaign, String> {
    // The absent execution dependency is reported even when other receipts also fail.
    let (e, eb): (Execution, _) = read(execution).map_err(|e| format!("execution receipt: {e}"))?;
    let (a, ab): (Acceptance, _) =
        read(acceptance).map_err(|e| format!("readiness receipt: {e}"))?;
    let (d, db): (Declaration, _) =
        read(declaration).map_err(|e| format!("declaration receipt: {e}"))?;
    validate_receipt_links(&d, &a, &e, &db, &ab)?;
    let context = context()?;
    let mut binding = context.verify()?;
    let m = wire::decode_manifest(wire::SELECTED)?;
    if source::PURPOSE != required_purpose || m.purpose != required_purpose {
        return Err("construction fixture cannot authorize science".into());
    }
    let ids = &d.identities;
    if ids.manifest_sha256 != binding.manifest_sha256
        || ids.compiled_inputs_sha256 != binding.compiled_inputs_sha256
        || ids.binary_sha256 != binding.executable_sha256
        || ids.protocol_sha256 != m.protocol_sha256
        || ids.proposal_sha256 != m.proposal_sha256
        || ids.resolved_roster_sha256 != m.roster_sha256
    {
        return Err("source/binary/protocol/manifest binding mismatch".into());
    }
    if artifact(&d.manifest)? != wire::SELECTED
        || d.source_inventory.sha256 != ids.source_inventory_sha256
    {
        return Err("manifest/inventory descriptor mismatch".into());
    }
    let (binary_hash, binary_bytes) = io::hash_file(Path::new(&d.binary.path), 1024 * 1024 * 1024)?;
    if binary_hash != ids.binary_sha256
        || binary_hash != d.binary.sha256
        || binary_bytes != d.binary.bytes
    {
        return Err("actual executable mismatch".into());
    }
    let packet: SourceInventory = decode(&d.source_inventory)?;
    packet.validate(&ids.compiled_inputs_sha256)?;
    let inventory = packet.entries;
    let current = source::inputs::read(context.root(), source::MANIFEST_PATH)?;
    let expected: Vec<_> = current
        .iter()
        .map(|(path, bytes)| Artifact {
            path: path.clone(),
            sha256: wire::sha256(bytes),
            bytes: bytes.len() as u64,
        })
        .collect();
    if inventory != expected {
        return Err("frozen source inventory/path mismatch".into());
    }
    for item in &inventory {
        let status = std::process::Command::new("git")
            .current_dir(context.root())
            .args([
                "status",
                "--porcelain",
                "--untracked-files=all",
                "--",
                &item.path,
            ])
            .output()
            .map_err(|e| e.to_string())?;
        if !status.status.success() || !status.stdout.is_empty() {
            return Err(format!("source-clean requirement failed: {}", item.path));
        }
    }
    let b: BuildReceipt = decode(&d.build)?;
    if b.schema != "war1-study-build-v1"
        || b.identities != *ids
        || b.selected_path != source::MANIFEST_PATH
        || b.selected_purpose != source::PURPOSE
        || b.binary != d.binary
        || b.inventory_before != inventory
        || b.inventory_after != inventory
        || b.rustc != source::TOOLCHAIN
        || b.features
            != source::FEATURES
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        || b.flags
            != source::FLAGS
                .iter()
                .map(|(a, b)| [a.to_string(), b.to_string()])
                .collect::<Vec<_>>()
        || !command(&b.build_command)
        || !text(&b.cargo)
        || !text(&b.platform)
    {
        return Err("invalid build evidence".into());
    }
    let n: NumericReceipt = decode(&d.numeric)?;
    let oracle_ids = wire::approved_oracle_ids()?;
    if n.schema != "war1-study-numeric-v1"
        || n.manifest_sha256 != ids.manifest_sha256
        || n.compiled_inputs_sha256 != ids.compiled_inputs_sha256
        || n.binary_sha256 != ids.binary_sha256
        || !(n.certificate_valid && n.mass_valid && n.width_valid && n.ulp_valid && n.ctmc_valid)
        || !n.unresolved_findings.is_empty()
        || n.oracle_ids != oracle_ids
    {
        return Err("numeric readiness failed".into());
    }
    let exported: super::numerics::NumericExport = decode(&n.numeric_export)?;
    if exported != super::numerics::export_verified_numeric_inputs(&m, &binding)? {
        return Err("numeric export differs from source-bound native certificate inputs".into());
    }
    artifact(&n.oracle_packet)?;
    let r: ResourceReceipt = decode(&d.resources)?;
    if r.schema != "war1-study-resources-v1"
        || r.manifest_sha256 != ids.manifest_sha256
        || r.compiled_inputs_sha256 != ids.compiled_inputs_sha256
        || r.binary_sha256 != ids.binary_sha256
        || r.budgets != d.budgets
        || r.census != d.census
        || !(r.schema_bounds_valid && r.aggregate_proof_valid && r.forecast_valid)
        || r.reserved_bytes > io::CAMPAIGN_LIMIT
        || r.failure_reserve_bytes < io::FAILURE_RESERVE
        || r.reserved_bytes < r.failure_reserve_bytes
        || !r.unavailable.is_empty()
        || r.fixture_probes.is_empty()
        || r.fixture_probes.len() > 32
        || !text(&r.forecast_method)
        || !r.forecast_wall_seconds.is_finite()
        || r.forecast_wall_seconds <= 0.0
        || r.forecast_wall_seconds > d.budgets.wall_seconds as f64
        || r.forecast_peak_rss_bytes == 0
        || r.forecast_peak_rss_bytes > d.budgets.rss_bytes
    {
        return Err("resource readiness failed".into());
    }
    artifact(&r.forecast_inputs)?;
    for p in &r.fixture_probes {
        if p.fixture_purpose != "construction_fixture"
            || !hash(&p.compiled_inputs_sha256)
            || !hash(&p.binary_sha256)
            || !command(&p.command)
            || !p.wall_seconds.is_finite()
            || p.wall_seconds <= 0.0
            || p.peak_rss_bytes == 0
            || p.peak_rss_bytes > d.budgets.rss_bytes
        {
            return Err("invalid fixture resource probe".into());
        }
        artifact(&p.artifact)?;
    }
    binding.declaration_sha256 = Some(wire::sha256(&db));
    binding.acceptance_sha256 = Some(wire::sha256(&ab));
    binding.execution_approval_sha256 = Some(wire::sha256(&eb));
    let descriptor = |path: &Path, bytes: &[u8]| Artifact {
        path: path.to_string_lossy().into_owned(),
        sha256: wire::sha256(bytes),
        bytes: bytes.len() as u64,
    };
    let mut retained = vec![
        ("declaration.json".into(), descriptor(declaration, &db)),
        ("acceptance.json".into(), descriptor(acceptance, &ab)),
        ("execution.json".into(), descriptor(execution, &eb)),
        ("build.json".into(), d.build.clone()),
        ("numeric.json".into(), d.numeric.clone()),
        ("resources.json".into(), d.resources.clone()),
        ("source-inventory.json".into(), d.source_inventory.clone()),
        ("numeric-export.json".into(), n.numeric_export.clone()),
        ("oracle-packet.json".into(), n.oracle_packet.clone()),
        (
            "resource-forecast-inputs.json".into(),
            r.forecast_inputs.clone(),
        ),
    ];
    for (i, probe) in r.fixture_probes.iter().enumerate() {
        retained.push((
            format!("fixture-probes/{i:06}.json"),
            probe.artifact.clone(),
        ));
    }
    Ok(AuthorizedCampaign {
        manifest: m,
        binding,
        retained,
    })
}
pub(super) fn fixture(path: &Path) -> Result<AuthorizedCampaign, String> {
    let bytes = io::read_input(path)?;
    let manifest = wire::decode_manifest(&bytes)?;
    if source::PURPOSE != "construction_fixture"
        || manifest.purpose != "construction_fixture"
        || manifest
            .cases
            .iter()
            .any(|c| ![7, 8].contains(&c.seed) || !c.arm_id.starts_with("fixture."))
    {
        return Err("fixed construction fixture authorization required".into());
    }
    let binding = source::verify()?;
    Ok(AuthorizedCampaign {
        manifest,
        binding,
        retained: vec![],
    })
}

pub(super) fn validate_receipt_links(
    d: &Declaration,
    a: &Acceptance,
    e: &Execution,
    db: &[u8],
    ab: &[u8],
) -> Result<(), String> {
    if e.schema != "war1-study-execution-v1"
        || a.schema != "war1-study-readiness-v1"
        || d.schema != "war1-study-declaration-v1"
    {
        return Err("unsupported activation schema".into());
    }
    if d.purpose != "prospective_frozen_not_blind"
        || a.decision != "accepted"
        || e.action != "execute_exact_population"
        || d.registered_histories != 0
        || a.registered_histories != 0
        || !a.open_required_findings.is_empty()
    {
        return Err("readiness/execution action or prior histories refused".into());
    }
    if !text(&a.reviewer_identity)
        || a.reviewer_role != "independent_reviewer"
        || !text(&e.owner_identity)
        || a.reviewer_identity == e.owner_identity
        || e.authorization_quote.trim().is_empty()
        || e.authorization_quote.len() > 4096
        || !text(&e.authorized_at)
    {
        return Err("independent review/explicit owner authorization evidence required".into());
    }
    if d.census != Population::required()
        || e.census != d.census
        || d.budgets != Limits::required()
        || e.budgets != d.budgets
    {
        return Err("exact census/budgets mismatch".into());
    }
    if d.normative_frozen_revision.len() != 40
        || d.declaration_revision.len() != 40
        || d.normative_frozen_revision == d.declaration_revision
        || !d
            .normative_frozen_revision
            .bytes()
            .chain(d.declaration_revision.bytes())
            .all(|b| b.is_ascii_hexdigit())
    {
        return Err("distinct frozen/declaration revisions required".into());
    }
    if a.declaration_sha256 != wire::sha256(db)
        || e.declaration_sha256 != a.declaration_sha256
        || e.acceptance_sha256 != wire::sha256(ab)
    {
        return Err("activation receipt hash mismatch".into());
    }

    let ids = &d.identities;
    if a.build_sha256 != d.build.sha256
        || a.numeric_sha256 != d.numeric.sha256
        || a.resources_sha256 != d.resources.sha256
        || a.binary_sha256 != ids.binary_sha256
        || a.manifest_sha256 != ids.manifest_sha256
        || e.binary_sha256 != ids.binary_sha256
        || e.manifest_sha256 != ids.manifest_sha256
        || e.compiled_inputs_sha256 != ids.compiled_inputs_sha256
        || e.source_inventory_sha256 != ids.source_inventory_sha256
    {
        return Err("readiness/execution evidence binding mismatch".into());
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn construction_test_preflight(
    context: source::VerificationContext,
    declaration: &Path,
    acceptance: &Path,
    execution: &Path,
) -> Result<AuthorizedCampaign, String> {
    let m = wire::decode_manifest(wire::SELECTED)?;
    if m.purpose != "construction_fixture" || m.cases.iter().any(|c| ![7, 8].contains(&c.seed)) {
        return Err("construction_test requires seeds 7/8 fixture".into());
    }
    preflight_with_context(
        declaration,
        acceptance,
        execution,
        || Ok(context),
        "construction_fixture",
    )
}
