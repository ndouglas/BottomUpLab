//! Bounded persistent-notes transport. Selected runs are engineering episodes.
use crate::Failure;
use clap::{Args, Subcommand};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};
use sugarscape_core::{
    config::FieldError,
    persistent_notes::{self as notes, Error, Limits, ReaderRoutine},
};
#[derive(Debug, Args)]
pub struct PersistentNotesArgs {
    #[command(subcommand)]
    mode: Mode,
}
#[derive(Debug, Subcommand)]
enum Mode {
    /// Export one labeled engineering episode, with registered settings.
    Run {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Collect the frozen full diagnostic; authorized collection only.
    Diagnose {
        #[arg(long)]
        out: PathBuf,
    },
    /// Reconstruct retained actions, views, ledgers, reductions and identities.
    Validate {
        #[arg(long)]
        input: PathBuf,
    },
    /// Freshly validate a retained report and preserve every exact aggregate.
    Summary {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}
fn operational(error: impl std::fmt::Display) -> Failure {
    Failure::Io(error.to_string())
}
fn scientific(error: Error) -> Failure {
    match error {
        Error::InvalidInput(_) | Error::InvalidHistory(_) | Error::InvalidReport(_) => {
            Failure::Invalid(vec![FieldError::new("persistent-notes", error.to_string())])
        }
        Error::ResourceLimit { .. } | Error::ArithmeticOverflow | Error::Io(_) => {
            operational(error)
        }
    }
}
fn create(path: &Path) -> Result<BufWriter<File>, Failure> {
    // Exclusive creation occurs before input parsing, replay or full collection.
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map(|file| BufWriter::with_capacity(65536, file))
        .map_err(operational)
}
fn config(path: &Path) -> Result<notes::Config, Failure> {
    let file = File::open(path).map_err(operational)?;
    if file.metadata().map_err(operational)?.len() > 4096 {
        return Err(operational(Error::ResourceLimit {
            resource: "config_bytes".into(),
            limit: 4096,
        }));
    }
    let mut text = String::new();
    file.take(4097)
        .read_to_string(&mut text)
        .map_err(operational)?;
    let config = notes::config_wire_from_json(&text).map_err(operational)?;
    config.validate().map_err(scientific)?;
    Ok(config)
}
fn report(path: &Path) -> Result<notes::Report, Failure> {
    let file = File::open(path).map_err(operational)?;
    let limits = Limits::registered().map_err(operational)?;
    if file.metadata().map_err(operational)?.len() > limits.report_bytes {
        return Err(operational(Error::ResourceLimit {
            resource: "report_bytes".into(),
            limit: limits.report_bytes,
        }));
    }
    notes::report_from_reader(BufReader::with_capacity(65536, file)).map_err(operational)
}
pub fn run(args: PersistentNotesArgs) -> Result<(), Failure> {
    match args.mode {
        Mode::Run { input, out } => {
            let mut writer = create(&out)?;
            let config = config(&input)?;
            eprintln!("engineering selected episode; not a diagnostic collection");
            if config.routine == ReaderRoutine::Oracle {
                eprintln!("privileged Oracle: request supplied privately at the initial boundary");
            }
            let episode = notes::run_episode(&config).map_err(scientific)?;
            notes::write_episode(&mut writer, &episode).map_err(operational)
        }
        Mode::Diagnose { out } => {
            let mut writer = create(&out)?;
            let limits = Limits::registered().map_err(operational)?;
            let report = notes::diagnose(&limits).map_err(scientific)?;
            notes::write_report(&mut writer, &report).map_err(operational)?;
            Ok(())
        }
        Mode::Validate { input } => {
            let report = report(&input)?;
            notes::validate_report(&report, &Limits::registered().map_err(operational)?)
                .map_err(scientific)?;
            println!("valid persistent-notes report");
            Ok(())
        }
        Mode::Summary { input, out } => {
            let mut writer = create(&out)?;
            let report = report(&input)?;
            let summary = notes::summary_json(&report).map_err(scientific)?;
            for chunk in summary.as_bytes().chunks(65536) {
                writer.write_all(chunk).map_err(operational)?;
            }
            writer.flush().map_err(operational)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scientific_and_operational_errors_remain_distinct() {
        for error in [
            Error::InvalidInput("settings".into()),
            Error::InvalidHistory("history".into()),
            Error::InvalidReport("integrity".into()),
        ] {
            assert!(matches!(scientific(error), Failure::Invalid(_)));
        }
        for error in [
            Error::ResourceLimit {
                resource: "report_bytes".into(),
                limit: 467585152,
            },
            Error::ArithmeticOverflow,
            Error::Io("flush".into()),
        ] {
            assert!(matches!(scientific(error), Failure::Io(_)));
        }
    }
}
