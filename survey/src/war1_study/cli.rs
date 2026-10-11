use super::{archive, gate, metadata, numerics, source, wire};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
pub struct CliFailure {
    pub exit_code: i32,
    pub message: String,
}
fn failure(message: impl ToString) -> CliFailure {
    CliFailure {
        exit_code: 1,
        message: message.to_string(),
    }
}
fn input_failure(message: impl ToString) -> CliFailure {
    CliFailure {
        exit_code: 2,
        message: message.to_string(),
    }
}
fn check_path(path: &Path) -> Result<(), CliFailure> {
    for part in path.ancestors() {
        match fs::symlink_metadata(part) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(failure("symlink path forbidden"))
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(failure(e)),
        }
    }
    Ok(())
}
fn load(path: &str) -> Result<wire::Manifest, CliFailure> {
    let path = Path::new(path);
    check_path(path).map_err(|e| input_failure(e.message))?;
    let meta = fs::symlink_metadata(path).map_err(input_failure)?;
    if !meta.is_file() || meta.len() > wire::LIMIT as u64 {
        return Err(input_failure("manifest must be a bounded regular file"));
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(input_failure)?
        .take(wire::LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(input_failure)?;
    wire::decode_manifest(&bytes).map_err(input_failure)
}
pub fn validate_command(path: &str) -> Result<(), CliFailure> {
    load(path)?;
    source::verify().map_err(input_failure)?;
    Ok(())
}
pub fn numerics_command(path: &str, output: &str) -> Result<(), CliFailure> {
    let manifest = load(path)?;
    let export = numerics::export_numeric_inputs(&manifest).map_err(input_failure)?;
    let mut bytes = serde_json::to_vec(&export).map_err(failure)?;
    bytes.push(b'\n');
    if bytes.len() > wire::LIMIT {
        return Err(failure("numeric export exceeds 16 MiB"));
    }
    let path = Path::new(output);
    check_path(path)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(failure)?;
    file.write_all(&bytes).map_err(failure)?;
    file.sync_all().map_err(failure)?;
    File::open(
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )
    .map_err(failure)?
    .sync_all()
    .map_err(failure)?;
    Ok(())
}
pub fn main_args(args: &[String]) -> Result<(), CliFailure> {
    match args {
        [] => {
            println!(
                "war1-study metadata | validate --manifest FILE | numerics --manifest FILE --out NEW_FILE | fixture --manifest FILE --out NEW_DIR | collect --declaration FILE --acceptance FILE --execution FILE --out NEW_DIR"
            );
            Ok(())
        }
        [flag] if flag == "--help" || flag == "-h" => {
            println!(
                "war1-study metadata | validate --manifest FILE | numerics --manifest FILE --out NEW_FILE | fixture --manifest FILE --out NEW_DIR | collect --declaration FILE --acceptance FILE --execution FILE --out NEW_DIR"
            );
            Ok(())
        }
        [cmd] if cmd == "metadata" => std::io::stdout()
            .write_all(&metadata::bytes().map_err(input_failure)?)
            .map_err(failure),
        [cmd, flag, path] if cmd == "validate" && flag == "--manifest" => validate_command(path),
        [cmd, flag, path, outflag, output]
            if cmd == "numerics" && flag == "--manifest" && outflag == "--out" =>
        {
            numerics_command(path, output)
        }
        [cmd, flag, path, outflag, output]
            if cmd == "fixture" && flag == "--manifest" && outflag == "--out" =>
        {
            archive::collect_fixture(Path::new(path), Path::new(output)).map_err(|e| {
                if e.census.planned == 0 {
                    input_failure(e.detail)
                } else {
                    failure(e.detail)
                }
            })?;
            Ok(())
        }
        [cmd, dflag, declaration, aflag, acceptance, eflag, execution, oflag, output]
            if cmd == "collect"
                && dflag == "--declaration"
                && aflag == "--acceptance"
                && eflag == "--execution"
                && oflag == "--out" =>
        {
            let authorized = gate::preflight(
                Path::new(declaration),
                Path::new(acceptance),
                Path::new(execution),
            )
            .map_err(input_failure)?;
            let _ = (authorized, output);
            Err(input_failure(
                "scientific collection requires the documented process supervisor control channel",
            ))
        }
        [cmd, flag, path, outflag, output, socketflag, socket]
            if cmd == "fixture"
                && flag == "--manifest"
                && outflag == "--out"
                && socketflag == "--control-socket" =>
        {
            archive::collect_fixture_monitored(
                Path::new(path),
                Path::new(output),
                Path::new(socket),
            )
            .map_err(|e| {
                if e.census.planned == 0 {
                    input_failure(e.detail)
                } else {
                    failure(e.detail)
                }
            })?;
            Ok(())
        }
        [cmd, dflag, declaration, aflag, acceptance, eflag, execution, oflag, output, sflag, socket]
            if cmd == "collect"
                && dflag == "--declaration"
                && aflag == "--acceptance"
                && eflag == "--execution"
                && oflag == "--out"
                && sflag == "--control-socket" =>
        {
            let authorized = gate::preflight(
                Path::new(declaration),
                Path::new(acceptance),
                Path::new(execution),
            )
            .map_err(input_failure)?;
            archive::collect(authorized, Path::new(output), Path::new(socket)).map_err(|e| {
                if e.census.planned == 0 {
                    input_failure(e.detail)
                } else {
                    failure(e.detail)
                }
            })?;
            Ok(())
        }
        _ => Err(CliFailure {
            exit_code: 2,
            message: "unsupported war1-study command or argument order".into(),
        }),
    }
}
