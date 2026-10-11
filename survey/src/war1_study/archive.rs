//! Compact fixed-block archive; a candidate index is never artifact success.
use super::{
    gate::{self, AuthorizedCampaign, CampaignBinding},
    io,
    observer::{self, CaseFailure, FrameSink, Outcome},
    source, wire,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    cell::Cell,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    rc::Rc,
};
use sugarscape_core::war::{Checkpoint, Frame};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveReceipt {
    pub sha256: String,
    pub bytes: u64,
    pub acknowledged_cases: u64,
}
#[derive(Debug, Serialize)]
pub struct CampaignFailure {
    pub detail: String,
    pub census: Census,
    pub receipt: ArchiveReceipt,
    pub case_failure: Option<CaseFailure>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Census {
    pub planned: u64,
    pub attempted: u64,
    pub complete_valid: u64,
    pub failed: u64,
    pub invalid: u64,
    pub partial: u64,
    pub pending: u64,
    pub unstarted: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    schema: String,
    mode: String,
    binding: CampaignBinding,
    planned: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(clippy::large_enum_variant)]
enum Row {
    Start {
        key: wire::Key,
        seed: u64,
        case_input_sha256: String,
    },
    Outcome {
        outcome: Outcome,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum AuditRow {
    Header {
        schema: String,
        key: wire::Key,
        case_json: String,
    },
    Initial {
        checkpoint_json: String,
    },
    Frame {
        step: u64,
        cause: String,
        frame_json: String,
    },
    Terminal {
        checkpoint_json: String,
        completed_steps: u64,
        observed_steps: u64,
        active_steps: u64,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BlockDescriptor {
    path: String,
    sha256: String,
    bytes: u64,
    first_ordinal: u64,
    acknowledged_cases: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    schema: String,
    mode: String,
    binding: CampaignBinding,
    census: Census,
    receipt: ArchiveReceipt,
    blocks: Vec<BlockDescriptor>,
    monitor: String,
}
#[derive(Clone)]
struct Budget(Rc<Cell<u64>>);
impl Budget {
    fn new() -> Self {
        Self(Rc::new(Cell::new(0)))
    }
    fn reserve(&self, n: u64) -> Result<(), String> {
        let next = self
            .0
            .get()
            .checked_add(n)
            .ok_or("campaign budget overflow")?;
        if next > io::CAMPAIGN_LIMIT - io::FAILURE_RESERVE {
            return Err("campaign budget including failure reserve exceeded".into());
        }
        self.0.set(next);
        Ok(())
    }
}
fn save(out: &Path, name: &str, value: &impl Serialize, budget: &Budget) -> Result<(), String> {
    let bytes = io::encode(value, io::METADATA_LIMIT, true)?;
    budget.reserve(bytes.len() as u64)?;
    io::save_bytes(&out.join(name), &bytes)
}
fn append(
    j: &mut io::Journal,
    value: &impl Serialize,
    limit: usize,
    budget: &Budget,
) -> Result<Vec<u8>, String> {
    let bytes = io::encode(value, limit, true)?;
    budget.reserve(bytes.len() as u64)?;
    j.append(value, limit)?;
    Ok(bytes)
}
struct Audit {
    journal: io::Journal,
    budget: Budget,
    steps: u64,
    path: String,
}
impl FrameSink for Audit {
    fn initial(&mut self, p: &Checkpoint) -> Result<(), String> {
        append(
            &mut self.journal,
            &AuditRow::Initial {
                checkpoint_json: String::from_utf8(observer::semantic_bytes(p)?)
                    .map_err(|e| e.to_string())?,
            },
            io::ROW_LIMIT,
            &self.budget,
        )?;
        Ok(())
    }
    fn frame(&mut self, f: &Frame) -> Result<(), String> {
        append(
            &mut self.journal,
            &AuditRow::Frame {
                step: f.step,
                cause: "benchmark_exposure".into(),
                frame_json: String::from_utf8(observer::semantic_bytes(f)?)
                    .map_err(|e| e.to_string())?,
            },
            io::ROW_LIMIT,
            &self.budget,
        )?;
        self.steps += 1;
        Ok(())
    }
    fn terminal(&mut self, p: &Checkpoint, observed: u64) -> Result<(), String> {
        append(
            &mut self.journal,
            &AuditRow::Terminal {
                checkpoint_json: String::from_utf8(observer::semantic_bytes(p)?)
                    .map_err(|e| e.to_string())?,
                completed_steps: p.step,
                observed_steps: observed,
                active_steps: p.active_steps,
            },
            io::ROW_LIMIT,
            &self.budget,
        )?;
        Ok(())
    }
}
fn empty_failure(detail: String, planned: u64) -> CampaignFailure {
    CampaignFailure {
        detail,
        census: Census {
            planned,
            unstarted: planned,
            ..Default::default()
        },
        receipt: ArchiveReceipt {
            sha256: wire::sha256(b""),
            bytes: 0,
            acknowledged_cases: 0,
        },
        case_failure: None,
    }
}
#[allow(clippy::result_large_err)] // Frozen receipt API owns census and actual checkpoint evidence.
pub fn collect(
    authorized: AuthorizedCampaign,
    _out: &Path,
) -> Result<ArchiveReceipt, CampaignFailure> {
    let (manifest, _) = authorized.parts();
    Err(empty_failure("scientific collector unready: Task5 process supervisor handshake and actual resource validation are required before output or construction".into(),manifest.cases.len() as u64))
}
#[allow(clippy::result_large_err)] // Frozen receipt API owns census and actual checkpoint evidence.
pub fn collect_fixture(
    manifest_path: &Path,
    out: &Path,
) -> Result<ArchiveReceipt, CampaignFailure> {
    let authorized = gate::fixture(manifest_path).map_err(|e| empty_failure(e, 0))?;
    collect_construction(authorized, out)
}
#[allow(clippy::result_large_err)] // Frozen receipt API owns census and actual checkpoint evidence.
fn collect_construction(
    authorized: AuthorizedCampaign,
    out: &Path,
) -> Result<ArchiveReceipt, CampaignFailure> {
    let (manifest, binding) = authorized.parts();
    let planned = manifest.cases.len() as u64;
    let mut census = Census {
        planned,
        unstarted: planned,
        ..Default::default()
    };
    let mut prefix = Sha256::new();
    let mut receipt = ArchiveReceipt {
        sha256: wire::sha256(b""),
        bytes: 0,
        acknowledged_cases: 0,
    };
    let mut case_failure = None;
    let budget = Budget::new();
    let mut created = false;
    let result = (|| -> Result<(), String> {
        if manifest.purpose != "construction_fixture"
            || binding.declaration_sha256.is_some()
            || binding.acceptance_sha256.is_some()
            || binding.execution_approval_sha256.is_some()
        {
            return Err("construction-only archive authorization required".into());
        }
        let entries = source::inputs::read(Path::new(source::ROOT), source::MANIFEST_PATH)?;
        let source_bytes: u64 = entries.iter().map(|(_, bytes)| bytes.len() as u64).sum();
        let audit_bound: u64 = manifest
            .arms
            .iter()
            .map(|arm| (arm.config.max_steps + 3) * io::ROW_LIMIT as u64)
            .sum();
        let reserved = source_bytes
            + audit_bound
            + planned * (io::ROW_LIMIT as u64 + 1024)
            + 4 * io::METADATA_LIMIT as u64
            + io::FAILURE_RESERVE;
        if reserved > io::CAMPAIGN_LIMIT {
            return Err("fixture reservation exceeds campaign cap".into());
        }
        io::create_directory(out)?;
        created = true;
        io::create_directory(&out.join("blocks"))?;
        io::create_directory(&out.join("audits"))?;
        io::create_directory(&out.join("source"))?;
        save(
            out,
            "header.json",
            &Header {
                schema: "war1-study-archive-v1".into(),
                mode: "construction_fixture".into(),
                binding: binding.clone(),
                planned,
            },
            &budget,
        )?;
        budget.reserve(wire::SELECTED.len() as u64)?;
        io::save_bytes(&out.join("manifest.json"), wire::SELECTED)?;
        let mut inventory = vec![];
        for (i, (path, bytes)) in entries.iter().enumerate() {
            budget.reserve(bytes.len() as u64)?;
            io::save_bytes(&out.join(format!("source/{i:06}.bin")), bytes)?;
            inventory.push(gate::Artifact {
                path: path.clone(),
                sha256: wire::sha256(bytes),
                bytes: bytes.len() as u64,
            });
        }
        save(
            out,
            "source-inventory.json",
            &gate::SourceInventory {
                schema: "war1-study-source-inventory-v1".into(),
                compiled_inputs_sha256: binding.compiled_inputs_sha256.clone(),
                entries: inventory,
            },
            &budget,
        )?;
        let mut blocks = vec![];
        for (block_index, cases) in manifest.cases.chunks(128).enumerate() {
            let path = format!("blocks/block-{block_index:06}.jsonl");
            let mut journal = io::Journal::new(&out.join(&path), io::CAMPAIGN_LIMIT)?;
            for case in cases {
                let arm = manifest
                    .arms
                    .iter()
                    .find(|a| a.arm_id == case.arm_id)
                    .ok_or("case arm absent")?;
                let input = observer::semantic_bytes(&observer::CaseInput {
                    config: arm.config.clone(),
                    seed: case.seed,
                })?;
                let start = Row::Start {
                    key: case.key(),
                    seed: case.seed,
                    case_input_sha256: wire::sha256(&input),
                };
                census.attempted += 1;
                census.unstarted -= 1;
                census.partial = 1;
                case_failure = Some(CaseFailure {
                    kind: "archive_start".into(),
                    detail: "start not acknowledged".into(),
                    key: case.key(),
                    attempted_step: None,
                    semantic_steps: 0,
                    observed_steps: 0,
                    durable_audit_steps: 0,
                    final_checkpoint: None,
                    acknowledged_archive_prefix: Some(receipt.clone()),
                });
                let bytes = append(&mut journal, &start, 1024, &budget)?;
                prefix.update(&bytes);
                receipt.bytes += bytes.len() as u64;
                receipt.sha256 = format!("{:x}", prefix.clone().finalize());
                census.partial = 0;
                census.pending = 1;
                let mut audit = if case.replicate_index == 0 {
                    let path = format!("audits/{}.jsonl", case.ordinal);
                    let mut a = Audit {
                        journal: io::Journal::new(&out.join(&path), io::CAMPAIGN_LIMIT)?,
                        budget: budget.clone(),
                        steps: 0,
                        path,
                    };
                    append(
                        &mut a.journal,
                        &AuditRow::Header {
                            schema: "war1-study-audit-v1".into(),
                            key: case.key(),
                            case_json: String::from_utf8(input).map_err(|e| e.to_string())?,
                        },
                        io::ROW_LIMIT,
                        &budget,
                    )?;
                    Some(a)
                } else {
                    None
                };
                let result = observer::execute_case(
                    case,
                    arm,
                    audit.as_mut().map(|a| a as &mut dyn FrameSink),
                );
                let mut outcome = match result {
                    Ok(value) => value,
                    Err(mut e) => {
                        e.acknowledged_archive_prefix = Some(receipt.clone());
                        let detail = e.detail.clone();
                        case_failure = Some(e);
                        return Err(detail);
                    }
                };
                case_failure = Some(CaseFailure {
                    kind: "audit_finish".into(),
                    detail: "audit closure not acknowledged".into(),
                    key: case.key(),
                    attempted_step: None,
                    semantic_steps: outcome.completed_steps,
                    observed_steps: outcome.observed_steps,
                    durable_audit_steps: audit.as_ref().map_or(0, |a| a.steps),
                    final_checkpoint: Some(outcome.final_checkpoint.clone()),
                    acknowledged_archive_prefix: Some(receipt.clone()),
                });
                if let Some(a) = audit.as_mut() {
                    let p = a.journal.finish()?;
                    outcome.audit_descriptor = Some(observer::AuditDescriptor {
                        path: a.path.clone(),
                        sha256: p.sha256,
                        bytes: p.bytes,
                        acknowledged_steps: a.steps,
                    });
                }
                case_failure = Some(CaseFailure {
                    kind: "archive_outcome".into(),
                    detail: "outcome not acknowledged".into(),
                    key: case.key(),
                    attempted_step: None,
                    semantic_steps: outcome.completed_steps,
                    observed_steps: outcome.observed_steps,
                    durable_audit_steps: outcome
                        .audit_descriptor
                        .as_ref()
                        .map_or(0, |d| d.acknowledged_steps),
                    final_checkpoint: Some(outcome.final_checkpoint.clone()),
                    acknowledged_archive_prefix: Some(receipt.clone()),
                });
                let bytes = append(
                    &mut journal,
                    &Row::Outcome { outcome },
                    io::ROW_LIMIT,
                    &budget,
                )?;
                prefix.update(&bytes);
                receipt.bytes += bytes.len() as u64;
                receipt.acknowledged_cases += 1;
                receipt.sha256 = format!("{:x}", prefix.clone().finalize());
                census.pending = 0;
                census.complete_valid += 1;
                case_failure = None;
            }
            let p = journal.finish()?;
            let descriptor = BlockDescriptor {
                path,
                sha256: p.sha256,
                bytes: p.bytes,
                first_ordinal: cases[0].ordinal,
                acknowledged_cases: cases.len() as u64,
            };
            save(
                out,
                &format!("blocks/block-{block_index:06}.descriptor.json"),
                &descriptor,
                &budget,
            )?;
            blocks.push(descriptor);
        }
        if source::verify()? != binding {
            return Err("source/binary changed before candidate publication".into());
        }
        validate_saved(out, &manifest, &binding, &blocks, &receipt)?;
        for dir in ["source", "audits", "blocks", ""] {
            io::sync_directory(&out.join(dir))?;
        }
        let candidate = Candidate {
            schema: "war1-study-candidate-index-v1".into(),
            mode: "construction_fixture".into(),
            binding,
            census: census.clone(),
            receipt: receipt.clone(),
            blocks,
            monitor: "construction_deterministic_no_scientific_resource_decision".into(),
        };
        if let Err(error) = save(out, "candidate-index.json", &candidate, &budget) {
            let path = out.join("candidate-index.json");
            if path.exists() {
                std::fs::remove_file(path)
                    .map_err(|e| format!("{error}; candidate revocation failed: {e}"))?;
                io::sync_directory(out)
                    .map_err(|e| format!("{error}; candidate revocation sync failed: {e}"))?;
            }
            return Err(error);
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(receipt),
        Err(mut detail) => {
            if census.pending > 0 {
                census.pending = 0;
                census.partial = 1
            }
            if let Some(f) = &mut case_failure {
                f.detail = detail.chars().take(1024).collect();
                f.acknowledged_archive_prefix = Some(receipt.clone());
            }
            let mut failure = CampaignFailure {
                detail: detail.clone(),
                census,
                receipt,
                case_failure,
            };
            if created {
                if let Err(e) = io::save_json(&out.join("failure.json"), &failure) {
                    detail.push_str(&format!("; failure receipt also failed: {e}"));
                    failure.detail = detail;
                }
            }
            Err(failure)
        }
    }
}
fn lines<T: serde::de::DeserializeOwned>(
    path: &Path,
    mut visit: impl FnMut(T) -> Result<(), String>,
) -> Result<(), String> {
    io::check_path(path)?;
    let f = File::open(path).map_err(|e| e.to_string())?;
    if !f.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("nonregular archive".into());
    }
    let mut reader = BufReader::new(f);
    loop {
        let mut line = Vec::new();
        let n = reader
            .by_ref()
            .take(io::ROW_LIMIT as u64 + 1)
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        if n > io::ROW_LIMIT || line.last() != Some(&b'\n') {
            return Err("oversized/torn archive row".into());
        }
        wire::reject_numeric_underflow(&line)?;
        visit(serde_json::from_slice(&line).map_err(|e| e.to_string())?)?;
    }
    Ok(())
}
fn validate_saved(
    out: &Path,
    manifest: &wire::Manifest,
    binding: &CampaignBinding,
    blocks: &[BlockDescriptor],
    receipt: &ArchiveReceipt,
) -> Result<(), String> {
    let header: Header = serde_json::from_slice(&io::read_input(&out.join("header.json"))?)
        .map_err(|e| e.to_string())?;
    if header.schema != "war1-study-archive-v1"
        || header.mode != "construction_fixture"
        || header.binding != *binding
        || header.planned != manifest.cases.len() as u64
        || io::read_input(&out.join("manifest.json"))? != wire::SELECTED
    {
        return Err("saved header/manifest binding mismatch".into());
    }
    let packet: gate::SourceInventory =
        serde_json::from_slice(&io::read_input(&out.join("source-inventory.json"))?)
            .map_err(|e| e.to_string())?;
    packet.validate(&binding.compiled_inputs_sha256)?;
    let inventory = packet.entries;
    if inventory.len() != source::INPUT_PATHS.len() {
        return Err("saved source inventory size".into());
    }
    let mut entries = vec![];
    for (i, item) in inventory.iter().enumerate() {
        let bytes = io::read_input(&out.join(format!("source/{i:06}.bin")))?;
        if item.path != source::INPUT_PATHS[i]
            || wire::sha256(&bytes) != item.sha256
            || bytes.len() as u64 != item.bytes
        {
            return Err("saved source packet mismatch".into());
        }
        entries.push((item.path.clone(), bytes));
    }
    if source::inputs::fingerprint(&entries, source::MANIFEST_PATH)?
        != binding.compiled_inputs_sha256
    {
        return Err("saved closure mismatch".into());
    }
    let mut ordinal = 0;
    let mut started = false;
    let mut hash = Sha256::new();
    let mut total = 0;
    for (i, block) in blocks.iter().enumerate() {
        let raw = io::read_bounded(&out.join(&block.path), 128 * (io::ROW_LIMIT + 1024))?;
        if wire::sha256(&raw) != block.sha256 || raw.len() as u64 != block.bytes {
            return Err("block descriptor mismatch".into());
        }
        hash.update(&raw);
        total += raw.len() as u64;
        let saved: BlockDescriptor = serde_json::from_slice(&io::read_input(
            &out.join(format!("blocks/block-{i:06}.descriptor.json")),
        )?)
        .map_err(|e| e.to_string())?;
        if saved.path != block.path
            || saved.sha256 != block.sha256
            || saved.bytes != block.bytes
            || saved.first_ordinal != ordinal as u64
            || saved.acknowledged_cases != manifest.cases[ordinal..].len().min(128) as u64
        {
            return Err("saved block closure mismatch".into());
        }
        lines::<Row>(&out.join(&block.path), |row| {
            let case = manifest.cases.get(ordinal).ok_or("extra archive key")?;
            let arm = manifest
                .arms
                .iter()
                .find(|a| a.arm_id == case.arm_id)
                .ok_or("missing arm")?;
            let input = observer::semantic_bytes(&observer::CaseInput {
                config: arm.config.clone(),
                seed: case.seed,
            })?;
            match row {
                Row::Start {
                    key,
                    seed,
                    case_input_sha256,
                } => {
                    if started
                        || key != case.key()
                        || seed != case.seed
                        || case_input_sha256 != wire::sha256(&input)
                    {
                        return Err("wrong/duplicate start key".into());
                    }
                    started = true
                }
                Row::Outcome { outcome: o } => {
                    if !started
                        || o.key != case.key()
                        || o.seed != case.seed
                        || o.manifest_sha256 != binding.manifest_sha256
                        || o.case_input_sha256 != wire::sha256(&input)
                    {
                        return Err("wrong outcome binding/order".into());
                    }
                    validate_outcome(&o, &arm.config)?;
                    if case.replicate_index == 0 {
                        validate_audit(out, &o, &input)?
                    } else if o.audit_descriptor.is_some() {
                        return Err("undeclared audit subset".into());
                    }
                    ordinal += 1;
                    started = false
                }
            }
            Ok(())
        })?;
    }
    if started
        || ordinal != manifest.cases.len()
        || receipt.acknowledged_cases != ordinal as u64
        || receipt.bytes != total
        || receipt.sha256 != format!("{:x}", hash.finalize())
    {
        return Err("incomplete archive census/prefix".into());
    }
    Ok(())
}
fn validate_outcome(
    o: &Outcome,
    c: &sugarscape_core::war::config::EngagementConfig,
) -> Result<(), String> {
    observer::validate_checkpoint(&o.final_checkpoint)?;
    if o.schema != "war1-study-outcome-v1"
        || o.final_checkpoint.config != *c
        || o.final_checkpoint.seed != o.seed
        || o.initial_counts != [c.blue, c.red]
        || o.final_counts
            != [
                o.final_checkpoint.blue_alive.len() as u32,
                o.final_checkpoint.red_alive.len() as u32,
            ]
        || o.completed_steps != o.final_checkpoint.step
        || o.observed_steps != o.completed_steps
        || o.active_steps != o.final_checkpoint.active_steps
        || o.calendar_time != o.completed_steps as f64 * c.dt
        || o.active_time != o.active_steps as f64 * c.dt
        || o.final_checkpoint.ending.as_ref() != Some(&o.ending)
        || o.observations.len() != 4
        || o.transcript_sha256.len() != 64
    {
        return Err("compact outcome clock/state mismatch".into());
    }
    for side in 0..2 {
        if o.final_counts[side].checked_add(o.casualty_totals[side]) != Some(o.initial_counts[side])
            || !o.contributed_exposure_totals[side].is_finite()
            || o.contributed_exposure_totals[side] < 0.0
        {
            return Err("outcome accounting mismatch".into());
        }
    }
    for (obs, t) in o.observations.iter().zip([1.0, 2.0, 4.0, 8.0]) {
        if obs.step != (t / c.dt) as u64
            || obs.counts[0] > c.blue
            || obs.counts[1] > c.red
            || obs.reference.is_some() == obs.reference_error.is_some()
            || if obs.step > o.completed_steps {
                obs.origin != "derived_absorbing"
                    || obs.reason.is_none()
                    || obs.counts != o.final_counts
            } else {
                obs.origin != "native" || obs.reason.is_some()
            }
        {
            return Err("observation origin/count/time mismatch".into());
        }
    }
    Ok(())
}
fn validate_audit(out: &Path, o: &Outcome, input: &[u8]) -> Result<(), String> {
    let d = o
        .audit_descriptor
        .as_ref()
        .ok_or("missing declared audit")?;
    if d.path != format!("audits/{}.jsonl", o.key.ordinal) {
        return Err("audit path mismatch".into());
    }
    let (hash, bytes) = io::hash_file(&out.join(&d.path), io::CAMPAIGN_LIMIT)?;
    if hash != d.sha256 || bytes != d.bytes || d.acknowledged_steps != o.completed_steps {
        return Err("audit descriptor mismatch".into());
    }
    let mut stage = 0;
    let mut count = 0;
    let mut semantic = Sha256::new();
    semantic.update(b"war1-study-transcript-v1\0");
    let mut state = observer::Accounting::new(&o.final_checkpoint.config);
    lines::<AuditRow>(&out.join(&d.path), |row| {
        match row {
            AuditRow::Header {
                schema,
                key,
                case_json,
            } => {
                let case: observer::CaseInput =
                    serde_json::from_str(&case_json).map_err(|e| e.to_string())?;
                if stage != 0
                    || schema != "war1-study-audit-v1"
                    || key != o.key
                    || case_json.as_bytes() != input
                    || case.config != o.final_checkpoint.config
                    || case.seed != o.seed
                {
                    return Err("audit header mismatch".into());
                }
                observer::hash_part(&mut semantic, case_json.as_bytes());
                stage = 1;
            }
            AuditRow::Initial { checkpoint_json } => {
                let p = observer::read_checkpoint(&checkpoint_json)?;
                observer::validate_checkpoint(&p)?;
                if stage != 1
                    || p.step != 0
                    || p.config != o.final_checkpoint.config
                    || p.seed != o.seed
                    || p.blue_alive.len() != o.initial_counts[0] as usize
                    || p.red_alive.len() != o.initial_counts[1] as usize
                {
                    return Err("audit initial mismatch".into());
                }
                observer::hash_part(&mut semantic, checkpoint_json.as_bytes());
                stage = 2;
            }
            AuditRow::Frame {
                step,
                cause,
                frame_json,
            } => {
                let f = observer::read_frame(&frame_json)?;
                if stage != 2
                    || step != count + 1
                    || f.step != step
                    || cause != "benchmark_exposure"
                {
                    return Err("audit frame ordering/cause".into());
                }
                state.accept(&o.final_checkpoint.config, &f)?;
                observer::hash_part(&mut semantic, frame_json.as_bytes());
                count += 1;
            }
            AuditRow::Terminal {
                checkpoint_json,
                completed_steps,
                observed_steps,
                active_steps,
            } => {
                let p = observer::read_checkpoint(&checkpoint_json)?;
                if stage != 2
                    || p != o.final_checkpoint
                    || completed_steps != o.completed_steps
                    || observed_steps != o.observed_steps
                    || active_steps != o.active_steps
                    || count != o.completed_steps
                {
                    return Err("audit terminal mismatch".into());
                }
                observer::hash_part(&mut semantic, checkpoint_json.as_bytes());
                stage = 3;
            }
        }
        Ok(())
    })?;
    if stage != 3
        || format!("{:x}", semantic.finalize()) != o.transcript_sha256
        || state.casualties != o.casualty_totals
        || state.exposure != o.contributed_exposure_totals
    {
        return Err("audit transcript/accounting mismatch".into());
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn construction_test_validate_saved(out: &Path) -> Result<(), String> {
    let candidate: Candidate =
        serde_json::from_slice(&io::read_input(&out.join("candidate-index.json"))?)
            .map_err(|e| e.to_string())?;
    validate_saved(
        out,
        &wire::decode_manifest(wire::SELECTED)?,
        &candidate.binding,
        &candidate.blocks,
        &candidate.receipt,
    )
}
