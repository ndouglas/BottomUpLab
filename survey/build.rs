//! Build-time source binding for the F5 collector; no Git or simulation is needed to build.
#[path = "build_support/f5_source_identity.rs"]
mod source_identity;
use std::{fs, path::Path};

fn read_inputs(
    root: &Path,
    relative: &Path,
    entries: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    let path = root.join(relative);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    if metadata.is_dir() {
        for entry in fs::read_dir(&path).map_err(|e| format!("{}: {e}", path.display()))? {
            let entry = entry.map_err(|e| format!("{}: {e}", path.display()))?;
            read_inputs(root, &relative.join(entry.file_name()), entries)?;
        }
    } else if metadata.is_file() {
        let name = relative
            .to_str()
            .ok_or("source path must be UTF-8")?
            .replace('\\', "/");
        if source_identity::selected(&name) {
            entries.push((
                name,
                fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
            ));
        }
    } else {
        return Err(format!(
            "source input must be a regular file or directory: {}",
            path.display()
        ));
    }
    Ok(())
}

fn main() {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory");
    let root = Path::new(&manifest).parent().expect("repository parent");
    let mut entries = Vec::new();
    for relative in source_identity::DIRECTORIES
        .iter()
        .chain(source_identity::FILES)
    {
        println!("cargo:rerun-if-changed={}", root.join(relative).display());
        read_inputs(root, Path::new(relative), &mut entries).expect("F5 build source identity");
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.dedup_by(|a, b| a.0 == b.0);
    let identity =
        source_identity::fingerprint(entries.into_iter().map(Ok)).expect("F5 source hash");
    let out = std::env::var_os("OUT_DIR").expect("Cargo output directory");
    fs::write(
        Path::new(&out).join("f5_source_identity.rs"),
        format!("const COMPILED_SOURCE_IDENTITY: &str = {identity:?};\n"),
    )
    .expect("write F5 compiled source identity");
    let entries = bt_inputs::read(root).expect("BT compiled inputs");
    for relative in bt_inputs::DIRECTORIES.iter().chain(bt_inputs::FILES) {
        println!("cargo:rerun-if-changed={}", root.join(relative).display());
    }
    let present = entries.iter().any(|e| e.0 == bt_inputs::PROTOCOL);
    let paths: Vec<_> = entries.iter().map(|e| e.0.as_str()).collect();
    let identity =
        bt_inputs::fingerprint(entries.clone().into_iter().map(Ok)).expect("BT source hash");
    let compiler = std::process::Command::new(std::env::var_os("RUSTC").expect("RUSTC"))
        .arg("-vV")
        .output()
        .expect("BT compiler identity");
    assert!(compiler.status.success(), "BT rustc -vV failed");
    let compiler = String::from_utf8(compiler.stdout).expect("UTF-8 compiler identity");
    let flags: Vec<_> = [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "TARGET",
        "HOST",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
    ]
    .iter()
    .map(|name| ((*name).to_owned(), std::env::var(name).unwrap_or_default()))
    .collect();
    fs::write(Path::new(&out).join("bt_compiled_inputs.rs"), format!(
        "const COMPILED_BT_INPUTS_SHA256: &str = {identity:?};\nconst COMPILED_BT_INPUT_PATHS: &[&str] = &{paths:?};\nconst COMPILED_BT_PROTOCOL_PRESENT: bool = {present};\nconst COMPILED_BT_TOOLCHAIN: &str = {compiler:?};\nconst COMPILED_BT_FLAGS: &[(&str,&str)] = &{flags:?};\n"
    )).expect("write BT compiled inputs");
    #[cfg(feature = "war-benchmarks")]
    war1_stamp(root, Path::new(&out));
    #[cfg(feature = "war-studies")]
    war1_study_stamp(root, Path::new(&out));
}

#[path = "build_support/bt_source_identity.rs"]
mod bt_inputs;

#[cfg(feature = "war-benchmarks")]
#[path = "build_support/war1_source_identity.rs"]
mod war1_inputs;

#[cfg(feature = "war-benchmarks")]
fn war1_stamp(root: &Path, out: &Path) {
    let entries = war1_inputs::read(root).expect("W1 compiled inputs");
    for relative in war1_inputs::DIRECTORIES.iter().chain(war1_inputs::FILES) {
        println!("cargo:rerun-if-changed={}", root.join(relative).display());
    }
    let paths: Vec<_> = entries.iter().map(|entry| entry.0.as_str()).collect();
    let identity =
        war1_inputs::fingerprint(entries.clone().into_iter().map(Ok)).expect("W1 source hash");
    let compiler = std::process::Command::new(std::env::var_os("RUSTC").expect("RUSTC"))
        .arg("-vV")
        .output()
        .expect("W1 compiler identity");
    assert!(compiler.status.success(), "W1 rustc -vV failed");
    let compiler = String::from_utf8(compiler.stdout).expect("UTF-8 W1 compiler identity");
    let flags: Vec<_> = [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "TARGET",
        "HOST",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
    ]
    .iter()
    .map(|name| ((*name).to_owned(), std::env::var(name).unwrap_or_default()))
    .collect();
    fs::write(out.join("war1_compiled_inputs.rs"), format!(
        "const COMPILED_W1_INPUTS_SHA256: &str = {identity:?};\nconst COMPILED_W1_INPUT_PATHS: &[&str] = &{paths:?};\nconst COMPILED_W1_TOOLCHAIN: &str = {compiler:?};\nconst COMPILED_W1_FLAGS: &[(&str,&str)] = &{flags:?};\n"
    )).expect("write W1 compiled inputs");
}

#[cfg(feature = "war-studies")]
#[path = "build_support/war1_study_source_identity.rs"]
mod war1_study_inputs;
#[cfg(feature = "war-studies")]
fn war1_study_stamp(root: &Path, out: &Path) {
    use sha2::{Digest, Sha256};
    println!("cargo:rerun-if-env-changed=WAR1_STUDY_MANIFEST");
    let selected = std::env::var("WAR1_STUDY_MANIFEST")
        .unwrap_or_else(|_| war1_study_inputs::DEFAULT_MANIFEST.into());
    war1_study_inputs::validate_selection(&selected).expect("study manifest selection");
    let entries = war1_study_inputs::read(root, &selected).expect("study compiled inputs");
    let bytes = &entries
        .iter()
        .find(|(path, _)| path == &selected)
        .expect("selected input present")
        .1;
    if selected != war1_study_inputs::DEFAULT_MANIFEST {
        let tracked = std::process::Command::new("git")
            .current_dir(root)
            .args(["ls-files", "--error-unmatch", "--", &selected])
            .output()
            .expect("check committed study manifest");
        assert!(
            tracked.status.success(),
            "selected scientific manifest must be committed"
        );
        let committed = war1_study_inputs::read_committed_manifest(root, &selected)
            .expect("read bounded committed study manifest");
        assert!(
            committed == *bytes,
            "selected manifest bytes must match HEAD"
        );
    }
    for (path, _) in &entries {
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }
    for path in war1_study_inputs::DIRECTORIES
        .iter()
        .copied()
        .chain(["survey/war1_study"])
    {
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }
    let paths: Vec<_> = entries.iter().map(|e| e.0.as_str()).collect();
    let identity = war1_study_inputs::fingerprint(&entries, &selected).expect("study source hash");
    let manifest_hash = format!("{:x}", Sha256::digest(bytes));
    let purpose = if selected == war1_study_inputs::DEFAULT_MANIFEST {
        "construction_fixture"
    } else {
        "prospective_seed_population"
    };
    let compiler = std::process::Command::new(std::env::var_os("RUSTC").expect("RUSTC"))
        .arg("-vV")
        .output()
        .expect("study rustc");
    assert!(compiler.status.success());
    let compiler = String::from_utf8(compiler.stdout).expect("rustc UTF8");
    let flags: Vec<_> = [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "TARGET",
        "HOST",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
    ]
    .into_iter()
    .map(|name| (name, std::env::var(name).unwrap_or_default()))
    .collect();
    let mut features: Vec<_> = std::env::vars()
        .filter(|(k, _)| k.starts_with("CARGO_FEATURE_"))
        .map(|(k, _)| k)
        .collect();
    features.sort();
    let root = root
        .canonicalize()
        .expect("canonical source root")
        .to_str()
        .expect("UTF8 root")
        .to_owned();
    fs::write(out.join("war1_study_manifest.json"), bytes).expect("embed selected manifest");
    fs::write(out.join("war1_study_stamp.rs"),format!("pub const ROOT:&str={root:?};\npub const MANIFEST_PATH:&str={selected:?};\npub const MANIFEST_SHA256:&str={manifest_hash:?};\npub const PURPOSE:&str={purpose:?};\npub const INPUTS_SHA256:&str={identity:?};\npub const INPUT_PATHS:&[&str]=&{paths:?};\npub const TOOLCHAIN:&str={compiler:?};\npub const FLAGS:&[(&str,&str)]=&{flags:?};\npub const FEATURES:&[&str]=&{features:?};\n")).expect("study stamp");
}
