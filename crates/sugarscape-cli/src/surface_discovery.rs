use crate::Failure;
use clap::{Args, Subcommand};
use std::io::Write;
use sugarscape_core::config::FieldError;
use sugarscape_core::surface_discovery::{
    diagnose, report_integrity, DiagnosticReport, Error, Limits,
};
#[derive(Debug, Args)]
pub struct SurfaceDiscoveryArgs {
    #[command(subcommand)]
    pub mode: SurfaceDiscoveryMode,
}
#[derive(Debug, Subcommand)]
pub enum SurfaceDiscoveryMode {
    Diagnose,
}
pub fn run(args: SurfaceDiscoveryArgs) -> Result<(), Failure> {
    match args.mode {
        SurfaceDiscoveryMode::Diagnose => {
            let limits = Limits::registered().map_err(invalid)?;
            let mut report = diagnose(&limits).map_err(invalid)?;
            emit(
                &mut report,
                &limits,
                &mut std::io::BufWriter::new(std::io::stdout().lock()),
            )
        }
    }
}
fn invalid(error: Error) -> crate::Failure {
    Failure::Invalid(vec![FieldError::new(
        "surface_discovery",
        error.to_string(),
    )])
}
fn emit<W: Write>(
    report: &mut DiagnosticReport,
    limits: &Limits,
    writer: &mut W,
) -> Result<(), Failure> {
    emit_checked(report, limits, writer, report_integrity)
}
fn emit_checked<W: Write>(
    report: &mut DiagnosticReport,
    limits: &Limits,
    writer: &mut W,
    validate: impl FnOnce(&DiagnosticReport, &Limits) -> Result<bool, Error>,
) -> Result<(), Failure> {
    report.passed = validate(report, limits).map_err(invalid)?;
    for check in &mut report.checks {
        check.passed = report.passed;
    }
    serde_json::to_writer(&mut *writer, report).map_err(|e| crate::Failure::Io(e.to_string()))?;
    writer
        .write_all(b"\n")
        .map_err(|e| crate::Failure::Io(e.to_string()))?;
    writer
        .flush()
        .map_err(|e| crate::Failure::Io(e.to_string()))?;
    if !report.passed {
        return Err(crate::Failure::Invalid(vec![FieldError::new(
            "surface_discovery",
            "diagnostic report failed current-payload validation",
        )]));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};
    fn incomplete() -> DiagnosticReport {
        serde_json::from_value(serde_json::json!({"version":"surface-discovery-diagnostic-v1","protocol_version":1,"supplied_structure":[],"source_hash":"","sources":[],"reference_artifacts":[],"histories":[],"panels":[],"comparisons":[],"role_expectations":[],"role_projections":[],"role_contrasts":[],"compilations":[],"checks":[{"name":"forged","passed":true}],"passed":true})).unwrap()
    }
    #[test]
    fn parsing_exposes_only_fixed_diagnose_and_help() {
        assert!(
            crate::Cli::try_parse_from(["sugarscape", "surface-discovery", "diagnose"]).is_ok()
        );
        for flag in [
            "--limit",
            "--states",
            "--sequence",
            "--policy",
            "--seed",
            "--out",
        ] {
            assert!(crate::Cli::try_parse_from([
                "sugarscape",
                "surface-discovery",
                "diagnose",
                flag,
                "1"
            ])
            .is_err());
        }
        assert!(crate::Cli::try_parse_from(["sugarscape", "surface-discovery"]).is_err());
        let help = crate::Cli::command()
            .find_subcommand_mut("surface-discovery")
            .unwrap()
            .render_help()
            .to_string();
        assert!(help.contains("diagnose"));
    }
    #[test]
    fn emission_rejects_forged_success_and_retains_failed_json_with_newline() {
        let mut report = incomplete();
        let mut bytes = Vec::new();
        assert!(matches!(
            emit(&mut report, &Limits::registered().unwrap(), &mut bytes),
            Err(Failure::Invalid(_))
        ));
        let value: DiagnosticReport = serde_json::from_slice(&bytes).unwrap();
        assert!(!value.passed);
        assert!(value.checks.iter().all(|c| !c.passed));
        assert_eq!(bytes.last(), Some(&b'\n'));
    }
    struct Broken {
        late_flush: bool,
    }
    impl Write for Broken {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.late_flush {
                Ok(bytes.len())
            } else {
                Err(std::io::Error::other("write failed"))
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("late flush failed"))
        }
    }
    #[test]
    fn write_and_late_flush_errors_are_io_failures() {
        for late_flush in [false, true] {
            assert!(matches!(
                emit(
                    &mut incomplete(),
                    &Limits::registered().unwrap(),
                    &mut Broken { late_flush }
                ),
                Err(Failure::Io(_))
            ));
        }
    }
    #[test]
    fn checked_emission_flushes_distinct_validation_outcomes_deterministically() {
        struct Writer {
            bytes: Vec<u8>,
            flushed: bool,
        }
        impl Write for Writer {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.bytes.extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.flushed = true;
                Ok(())
            }
        }
        for accepted in [false, true] {
            let mut outputs = Vec::new();
            for _ in 0..2 {
                let mut writer = Writer {
                    bytes: vec![],
                    flushed: false,
                };
                let mut called = 0;
                let result = emit_checked(
                    &mut incomplete(),
                    &Limits::registered().unwrap(),
                    &mut writer,
                    |_, _| {
                        called += 1;
                        Ok(accepted)
                    },
                );
                assert_eq!(result.is_ok(), accepted);
                assert_eq!(called, 1);
                assert!(writer.flushed);
                let emitted: DiagnosticReport = serde_json::from_slice(&writer.bytes).unwrap();
                assert_eq!(emitted.passed, accepted);
                assert!(emitted.checks.iter().all(|c| c.passed == accepted));
                assert_eq!(writer.bytes.last(), Some(&b'\n'));
                outputs.push(writer.bytes);
            }
            assert_eq!(outputs[0], outputs[1]);
        }
    }
    #[test]
    fn validator_operational_error_preserves_context_and_emits_no_success() {
        let mut bytes = Vec::new();
        let result = emit_checked(
            &mut incomplete(),
            &Limits::registered().unwrap(),
            &mut bytes,
            |_, _| {
                Err(Error::ResourceLimit {
                    resource: "states".into(),
                    limit: 1,
                })
            },
        );
        let Err(Failure::Invalid(fields)) = result else {
            panic!("contextual validation failure expected");
        };
        assert!(format!("{fields:?}").contains("surface_discovery"));
        assert!(format!("{fields:?}").contains("states limit 1 exhausted"));
        assert!(bytes.is_empty());
    }
}
