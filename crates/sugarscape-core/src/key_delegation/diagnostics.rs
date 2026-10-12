//! Source-bound exhaustive reports. Imported panels never mint trusted GameTables.
use super::{
    canonical::canonical_bytes, protocol::*, references, sha256::Digest, source_inputs, *,
};
use serde::{
    de::{Error as _, MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::io::Write;

// Decode large containers directly: duplicate detection precedes field retention, while
// nested scientific atoms use the strict native decoders. Never materialize a whole Value.
macro_rules! report_object {
    ($name:ident {$(pub $field:ident:$ty:ty),* $(,)?}) => {
        #[derive(Clone,Debug,PartialEq,Eq,Serialize)]
        pub struct $name {$(pub $field:$ty),*}
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{
                struct Object;
                impl<'de> Visitor<'de> for Object {
                    type Value=$name;
                    fn expecting(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str(stringify!($name))}
                    fn visit_map<A:MapAccess<'de>>(self,mut map:A)->Result<$name,A::Error>{
                        $(let mut $field:Option<$ty>=None;)*
                        while let Some(key)=map.next_key::<String>()? {
                            match key.as_str(){
                                $(stringify!($field)=>{if $field.is_some(){return Err(A::Error::duplicate_field(stringify!($field)));}$field=Some(map.next_value()?);},)*
                                _=>return Err(A::Error::unknown_field(&key,&[$(stringify!($field)),*])),
                            }
                        }
                        Ok($name{$($field:$field.ok_or_else(||A::Error::missing_field(stringify!($field)))?),*})
                    }
                }
                d.deserialize_map(Object)
            }
        }
    }
}
wire_object!(Check {pub name:String,pub passed:bool,pub detail:String});
report_object!(ReportPanel {pub stratum:Stratum,pub rows:Vec<Row>,pub analysis:Analysis,pub controls:Vec<ControlRecord>,pub stats:CompileStats});
report_object!(Report {pub version:String,pub source_hash:String,pub source_receipts:Vec<Receipt>,pub reference_receipts:Vec<Receipt>,pub limits:Limits,pub panels:Vec<ReportPanel>,pub contrasts:Vec<Contrast>,pub checks:Vec<Check>,pub passed:bool});
wire_object!(Summary {pub version:String,pub report_hash:String,pub source_hash:String,pub source_receipts:Vec<Receipt>,pub reference_receipts:Vec<Receipt>,pub limits:Limits,pub settings:Vec<SummaryStratum>,pub contrasts:Vec<Contrast>,pub checks:Vec<Check>,pub passed:bool});
fn invalid(s: impl Into<String>) -> Error {
    Error::InvalidReport(s.into())
}
fn registered(limits: &Limits) -> Result<(), Error> {
    if *limits != Limits::registered()? {
        return Err(invalid("report limits differ from registered caps"));
    }
    Ok(())
}
fn metadata(report: &Report, limits: &Limits) -> Result<(), Error> {
    registered(limits)?;
    if report.version != REPORT_VERSION {
        return Err(invalid("report version"));
    }
    if report.source_receipts != source_receipts()? || report.source_hash != source_identity()? {
        return Err(invalid("stale source receipts or identity"));
    }
    if report.reference_receipts != reference_receipts()? {
        return Err(invalid("stale reference receipts"));
    }
    if report.limits != *limits {
        return Err(invalid(
            "report limits differ from requested registered caps",
        ));
    }
    Ok(())
}
pub(super) fn check_controls(stratum: &Stratum, controls: &[ControlRecord]) -> Result<(), Error> {
    let selectors = [
        Selector::SoloAttempt,
        Selector::SuppliedHandoff,
        Selector::TeamOptimal,
        Selector::NashLex,
        Selector::NashBestGroup,
        Selector::InitiallyEquipped,
    ];
    if controls.len() != selectors.len() {
        return Err(invalid("six controls required"));
    }
    for (control, selector) in controls.iter().zip(selectors) {
        if control.selector != selector {
            return Err(invalid("control order"));
        }
        let initial = selector == Selector::InitiallyEquipped;
        let scope = if initial {
            ClassificationScope::ChangedInitialControl
        } else {
            ClassificationScope::Main
        };
        if control.classification_scope != scope {
            return Err(invalid("control classification scope"));
        }
        match (&control.selection, &control.episode, &control.unavailable) {
            (Some(s), Some(e), None) => {
                let owner = if initial {
                    InitialOwner::Receiver
                } else {
                    InitialOwner::Giver
                };
                if s.initial_owner != owner
                    || e.config.initial_owner() != owner
                    || e.config.stratum() != stratum
                    || e.config.ids() != &Ids::default()
                    || e.profile != s.profile
                {
                    return Err(invalid("control ownership/config/profile"));
                }
                if s.classification.is_none() != initial
                    || (initial && s.profile != Profile::new(0, 60)?)
                {
                    return Err(invalid(
                        "control classification or InitiallyEquipped profile",
                    ));
                }
                e.validate()?;
                if s.classification
                    .as_ref()
                    .is_some_and(|c| c.outcome != e.outcome)
                {
                    return Err(invalid("control classification outcome"));
                }
            }
            (None, None, Some(reason))
                if matches!(selector, Selector::NashLex | Selector::NashBestGroup)
                    && reason == "no_pure_equilibrium" => {}
            _ => return Err(invalid("control availability")),
        }
    }
    Ok(())
}
pub(super) fn structure(report: &Report) -> Result<(), Error> {
    check_structure(&report.checks)?;
    if report.panels.len() != 36 {
        return Err(invalid("registered census requires 36 ordered panels"));
    }
    for (i, (panel, stratum)) in report.panels.iter().zip(strata()).enumerate() {
        if panel.stratum != stratum {
            return Err(invalid(format!("panel {i} stratum order")));
        }
    }
    for (i, panel) in report.panels.iter().enumerate() {
        if panel.rows.len() != 46656 {
            return Err(invalid(format!("panel {i} requires 46656 ordered rows")));
        }
        check_controls(&panel.stratum, &panel.controls)?;
    }
    let pairs = contrast_pairs();
    if report.contrasts.len() != pairs.len() {
        return Err(invalid("84 ordered contrasts required"));
    }
    for (c, (axis, l, u)) in report.contrasts.iter().zip(pairs) {
        if c.axis != axis || c.lower != l || c.upper != u {
            return Err(invalid("contrast endpoint order"));
        }
    }
    Ok(())
}
pub(super) fn check_structure(checks: &[Check]) -> Result<(), Error> {
    if checks.len() != CHECK_NAMES.len() || checks.iter().zip(CHECK_NAMES).any(|(c, n)| c.name != n)
    {
        return Err(invalid("check name order"));
    }
    if checks.iter().any(|c| c.detail.is_empty()) {
        return Err(invalid("empty check detail"));
    }
    Ok(())
}
pub(super) fn checks_match(reported: &[Check], actual: &[Check]) -> Result<bool, Error> {
    check_structure(reported)?;
    check_structure(actual)?;
    Ok(reported
        .iter()
        .zip(actual)
        .all(|(reported, actual)| reported.name == actual.name && reported.passed == actual.passed))
}
pub(super) fn panel_matches(
    reported: &ReportPanel,
    rows: &[Row],
    analysis: &Analysis,
    controls: &[ControlRecord],
    stats: &CompileStats,
) -> bool {
    reported.rows == rows
        && reported.analysis == *analysis
        && reported.controls == controls
        && reported.stats == *stats
}
pub(super) fn contrasts_match(reported: &[Contrast], actual: &[Contrast]) -> bool {
    reported == actual
}

fn contrasts(tables: &[GameTable]) -> Result<Vec<Contrast>, Error> {
    contrast_pairs()
        .into_iter()
        .map(|(axis, l, u)| {
            let find = |s: &Stratum| {
                tables
                    .iter()
                    .find(|t| t.stratum() == s)
                    .ok_or_else(|| invalid("missing contrast endpoint"))
            };
            contrast(find(&l)?, find(&u)?, axis)
        })
        .collect()
}
/// Streaming raw length/hash accumulator, with checked byte accounting.
#[derive(Default)]
struct HashWriter {
    digest: Digest,
    bytes: u64,
}
impl Write for HashWriter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(b.len() as u64)
            .ok_or_else(|| std::io::Error::other("report byte overflow"))?;
        self.digest.write_all(b)?;
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn write(out: &mut impl Write, bytes: &[u8]) -> Result<(), Error> {
    out.write_all(bytes).map_err(|e| Error::Io(e.to_string()))
}
fn atom(out: &mut impl Write, value: &impl Serialize) -> Result<(), Error> {
    write(out, &canonical_bytes(value)?)
}
fn array<T: Serialize>(out: &mut impl Write, items: &[T]) -> Result<(), Error> {
    write(out, b"[")?;
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            write(out, b",")?;
        }
        atom(out, item)?;
    }
    write(out, b"]")
}
pub(super) fn raw_rows(path: &str, rows: &[Row]) -> Result<Receipt, Error> {
    let mut out = HashWriter::default();
    array(&mut out, rows)?;
    Ok(Receipt {
        path: path.into(),
        bytes: out.bytes,
        sha256: out.digest.finish(),
    })
}
fn checks(
    tables: &[GameTable],
    controls: &[Vec<ControlRecord>],
    contrasts: &[Contrast],
    limits: &Limits,
) -> Result<Vec<Check>, Error> {
    let refs = references::load()?;
    let mut flags = [true; 11];
    flags[0] = tables.len() == 36 && tables.iter().all(|t| t.rows().len() == 46656);
    for (i, table) in tables.iter().enumerate() {
        let expected = &refs.game.panels[i];
        let analysis = table.analysis();
        let receipt = raw_rows(&expected.row_receipt.path, table.rows())?;
        let rows = receipt == expected.row_receipt
            && source_inputs::identity(&[receipt])? == expected.row_identity;
        flags[1] &= rows;
        flags[2] &= rows;
        flags[3] &= controls[i] == expected.controls;
        flags[4] &= analysis.giver_best == expected.analysis.giver_best
            && analysis.receiver_best == expected.analysis.receiver_best;
        flags[5] &= analysis.pure == expected.analysis.pure;
        flags[6] &= analysis.team == expected.analysis.team
            && analysis.lex_pure == expected.analysis.lex_pure
            && analysis.best_group_pure == expected.analysis.best_group_pure
            && analysis.solo_best == expected.analysis.solo_best;
        flags[9] &= *table.stats() == expected.stats
            && *table.stats() == refs.limits.panels[i].stats
            && table.stats().profiles <= limits.profiles
            && table.stats().transitions <= limits.transitions
            && table.stats().peak_bytes <= limits.allocation_bytes;
    }
    for case in &refs.kernel.cases {
        let actual = run_episode(&case.config, case.profile)?;
        flags[1] &= actual == case.episode;
        flags[2] &= trace_hash(&actual)? == case.trace_hash;
    }
    flags[7] = contrasts == refs.game.contrasts;
    flags[8] = contrasts
        .iter()
        .filter(|c| c.axis == ContrastAxis::GiverShare)
        .all(|c| c.fixed_words.share_rule_count == Some(46656));
    CHECK_NAMES
        .into_iter()
        .zip(flags)
        .map(|(name, passed)| {
            Ok(Check {
                name: name.into(),
                passed,
                detail: if passed {
                    "complete current-source reconstruction agrees"
                } else {
                    "complete current-source reconstruction disagrees"
                }
                .into(),
            })
        })
        .collect()
}
pub fn diagnose(limits: &Limits) -> Result<Report, Error> {
    registered(limits)?;
    let tables = strata()
        .into_iter()
        .map(|s| compile(s, limits))
        .collect::<Result<Vec<_>, _>>()?;
    let controls = tables
        .iter()
        .map(super::controls)
        .collect::<Result<Vec<_>, _>>()?;
    let contrasts = contrasts(&tables)?;
    let checks = checks(&tables, &controls, &contrasts, limits)?;
    let passed = checks.iter().all(|c| c.passed);
    let panels = tables
        .into_iter()
        .zip(controls)
        .map(|(table, controls)| {
            let (stratum, rows, analysis, stats) = table.into_parts();
            ReportPanel {
                stratum,
                rows,
                analysis,
                controls,
                stats,
            }
        })
        .collect();
    Ok(Report {
        version: REPORT_VERSION.into(),
        source_hash: source_identity()?,
        source_receipts: source_receipts()?,
        reference_receipts: reference_receipts()?,
        limits: limits.clone(),
        panels,
        contrasts,
        checks,
        passed,
    })
}
fn reconstruct(report: &Report, limits: &Limits) -> Result<(bool, Vec<SummaryStratum>), Error> {
    metadata(report, limits)?;
    structure(report)?;
    let mut tables = Vec::with_capacity(36);
    let mut native_controls = Vec::with_capacity(36);
    let mut settings = Vec::with_capacity(36);
    for panel in &report.panels {
        let table = compile(panel.stratum.clone(), limits)?;
        let controls = super::controls(&table)?;
        if !panel_matches(
            panel,
            table.rows(),
            table.analysis(),
            &controls,
            table.stats(),
        ) {
            return Ok((false, vec![]));
        }
        settings.push(summarize(&table)?);
        native_controls.push(controls);
        tables.push(table);
    }
    let actual_contrasts = contrasts(&tables)?;
    if !contrasts_match(&report.contrasts, &actual_contrasts) {
        return Ok((false, vec![]));
    }
    let actual_checks = checks(&tables, &native_controls, &actual_contrasts, limits)?;
    let passed = actual_checks.iter().all(|c| c.passed);
    Ok((
        passed && report.passed == passed && checks_match(&report.checks, &actual_checks)?,
        settings,
    ))
}
pub fn report_integrity(report: &Report, limits: &Limits) -> Result<bool, Error> {
    reconstruct(report, limits).map(|r| r.0)
}
// Sorted full-report canonicalization streams panels and rows separately. Largest temporary
// Value is one metadata item (Analysis/Contrast), never the complete 1,679,616-row report.
pub(super) fn canonical_report(report: &Report, out: &mut impl Write) -> Result<(), Error> {
    write(out, b"{\"checks\":")?;
    array(out, &report.checks)?;
    write(out, b",\"contrasts\":")?;
    array(out, &report.contrasts)?;
    write(out, b",\"limits\":")?;
    atom(out, &report.limits)?;
    write(out, b",\"panels\":[")?;
    for (i, p) in report.panels.iter().enumerate() {
        if i > 0 {
            write(out, b",")?;
        }
        write(out, b"{\"analysis\":")?;
        atom(out, &p.analysis)?;
        write(out, b",\"controls\":")?;
        array(out, &p.controls)?;
        write(out, b",\"rows\":")?;
        array(out, &p.rows)?;
        write(out, b",\"stats\":")?;
        atom(out, &p.stats)?;
        write(out, b",\"stratum\":")?;
        atom(out, &p.stratum)?;
        write(out, b"}")?;
    }
    write(out, b"],\"passed\":")?;
    atom(out, &report.passed)?;
    write(out, b",\"reference_receipts\":")?;
    array(out, &report.reference_receipts)?;
    write(out, b",\"source_hash\":")?;
    atom(out, &report.source_hash)?;
    write(out, b",\"source_receipts\":")?;
    array(out, &report.source_receipts)?;
    write(out, b",\"version\":")?;
    atom(out, &report.version)?;
    write(out, b"}")
}
pub fn summary_json(report: &Report, limits: &Limits) -> Result<String, Error> {
    let (valid, settings) = reconstruct(report, limits)?;
    if !valid {
        return Err(invalid(
            "summary requires successful fresh report integrity",
        ));
    }
    let mut out = HashWriter::default();
    canonical_report(report, &mut out)?;
    let report_hash = source_inputs::identity(&[Receipt {
        path: "key-delegation-report-v1".into(),
        bytes: out.bytes,
        sha256: out.digest.finish(),
    }])?;
    let summary = Summary {
        version: SUMMARY_VERSION.into(),
        report_hash,
        source_hash: report.source_hash.clone(),
        source_receipts: report.source_receipts.clone(),
        reference_receipts: report.reference_receipts.clone(),
        limits: report.limits.clone(),
        settings,
        contrasts: report.contrasts.clone(),
        checks: report.checks.clone(),
        passed: report.passed,
    };
    serde_json::to_string(&summary).map_err(|e| Error::InvalidReport(e.to_string()))
}
