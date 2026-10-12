#[cfg(test)]
mod tests {
    use super::*;
    fn incomplete() -> Report {
        Report {
            version: "key-delegation-diagnostic-v1".into(),
            source_hash: String::new(),
            source_receipts: vec![],
            reference_receipts: vec![],
            limits: Limits::registered().unwrap(),
            panels: vec![],
            contrasts: vec![],
            checks: vec![],
            passed: false,
        }
    }
    struct Broken {
        flush: bool,
    }
    impl Write for Broken {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.flush {
                Ok(bytes.len())
            } else {
                Err(std::io::Error::other("write failed"))
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("flush failed"))
        }
    }
    #[test]
    fn emission_checks_write_and_flush_errors() {
        for flush in [false, true] {
            assert!(matches!(
                write_report(&incomplete(), &mut Broken { flush }),
                Err(Failure::Io(_))
            ));
        }
    }
    #[test]
    fn transport_emits_compact_struct_json_and_exactly_one_newline() {
        let report = incomplete();
        let mut bytes = Vec::new();
        write_report(&report, &mut bytes).unwrap();
        assert_eq!(bytes.last(), Some(&b'\n'));
        assert_ne!(bytes.get(bytes.len() - 2), Some(&b'\n'));
        assert_eq!(
            &bytes[..bytes.len() - 1],
            serde_json::to_vec(&report).unwrap()
        );
        assert_eq!(serde_json::from_slice::<Report>(&bytes).unwrap(), report);
    }
}
use crate::Failure;
use clap::{Args, Subcommand};
use std::io::Write;
use sugarscape_core::{
    config::FieldError,
    key_delegation::{diagnose, report_integrity, Limits, Report},
};
#[derive(Debug, Args)]
pub struct KeyDelegationArgs {
    #[command(subcommand)]
    pub mode: KeyDelegationMode,
}
#[derive(Debug, Subcommand)]
pub enum KeyDelegationMode {
    Diagnose,
}
pub fn run(args: KeyDelegationArgs) -> Result<(), Failure> {
    match args.mode {
        KeyDelegationMode::Diagnose => {
            let limits = Limits::registered().map_err(|e| Failure::Io(e.to_string()))?;
            let mut report = diagnose(&limits).map_err(|e| Failure::Io(e.to_string()))?;
            let passed =
                report_integrity(&report, &limits).map_err(|e| Failure::Io(e.to_string()))?;
            let stdout = std::io::stdout();
            emit_validated(
                &mut report,
                passed,
                &mut std::io::BufWriter::new(stdout.lock()),
            )
        }
    }
}
fn write_report<W: Write>(report: &Report, writer: &mut W) -> Result<(), Failure> {
    let bytes = serde_json::to_vec(report).map_err(|e| Failure::Io(e.to_string()))?;
    writer
        .write_all(&bytes)
        .map_err(|e| Failure::Io(e.to_string()))?;
    writer
        .write_all(b"\n")
        .map_err(|e| Failure::Io(e.to_string()))?;
    writer.flush().map_err(|e| Failure::Io(e.to_string()))
}

#[cfg(test)]
mod status_tests {
    use super::*;
    #[test]
    fn scientific_failure_is_emitted_before_validation_exit_classification() {
        let mut report = Report {
            version: "key-delegation-diagnostic-v1".into(),
            source_hash: String::new(),
            source_receipts: vec![],
            reference_receipts: vec![],
            limits: Limits::registered().unwrap(),
            panels: vec![],
            contrasts: vec![],
            checks: vec![],
            passed: false,
        };
        let mut bytes = Vec::new();
        assert!(matches!(
            emit_validated(&mut report, false, &mut bytes),
            Err(Failure::Invalid(_))
        ));
        assert!(!serde_json::from_slice::<Report>(&bytes).unwrap().passed);
    }
}

fn emit_validated<W: Write>(
    report: &mut Report,
    passed: bool,
    writer: &mut W,
) -> Result<(), Failure> {
    report.passed = passed;
    write_report(report, writer)?;
    if passed {
        Ok(())
    } else {
        Err(Failure::Invalid(vec![FieldError::new(
            "key_delegation",
            "diagnostic scientific checks failed",
        )]))
    }
}
#[cfg(test)]
mod dispatch_tests {
    use clap::Parser;
    #[test]
    fn strict_dispatch_accepts_only_registered_diagnose_shape() {
        assert!(crate::Cli::try_parse_from(["sugarscape", "key-delegation", "diagnose"]).is_ok());
        for args in [
            vec!["sugarscape", "key-delegation"],
            vec!["sugarscape", "key-delegation", "other"],
            vec!["sugarscape", "key-delegation", "diagnose", "extra"],
            vec!["sugarscape", "key-delegation", "diagnose", "--seed", "1"],
            vec!["sugarscape", "key-delegation", "diagnose", "--limits", "1"],
        ] {
            assert!(crate::Cli::try_parse_from(args).is_err());
        }
    }
}

#[cfg(test)]
mod newline_tests {
    use super::*;
    struct NewlineFailure {
        writes: usize,
    }
    impl Write for NewlineFailure {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.writes += 1;
            if self.writes == 2 {
                Err(std::io::Error::other("newline failed"))
            } else {
                Ok(b.len())
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            panic!("flush after failed newline")
        }
    }
    #[test]
    fn newline_write_failure_is_operational() {
        let report = Report {
            version: "key-delegation-diagnostic-v1".into(),
            source_hash: String::new(),
            source_receipts: vec![],
            reference_receipts: vec![],
            limits: Limits::registered().unwrap(),
            panels: vec![],
            contrasts: vec![],
            checks: vec![],
            passed: false,
        };
        assert!(matches!(
            write_report(&report, &mut NewlineFailure { writes: 0 }),
            Err(Failure::Io(_))
        ));
    }
}
