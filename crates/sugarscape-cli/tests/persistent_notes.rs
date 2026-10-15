use std::{fs, process::Command};
fn dir(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("persistent-notes-{name}-{}", std::process::id()));
    fs::create_dir_all(&p).unwrap();
    p
}
fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn malformed_and_existing_outputs_do_not_collect() {
    let p = dir("precheck");
    let input = p.join("input.json");
    let out = p.join("out.json");
    fs::write(&input, "{").unwrap();
    let result = cli(&[
        "persistent-notes",
        "run",
        "--input",
        input.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    fs::write(&out, "preserved").unwrap();
    let result = cli(&[
        "persistent-notes",
        "diagnose",
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(fs::read_to_string(&out).unwrap(), "preserved");
    fs::remove_dir_all(p).unwrap();
}
#[test]
fn selected_right_ledger_and_oracle_label() {
    let p = dir("selected");
    let input = p.join("input.json");
    let out = p.join("out.json");
    let config = sugarscape_core::persistent_notes::Config::example(
        sugarscape_core::persistent_notes::Arm::Retained,
        sugarscape_core::persistent_notes::ReaderRoutine::ReadAdaptive,
        sugarscape_core::persistent_notes::Good::Right,
    );
    fs::write(&input, serde_json::to_vec(&config).unwrap()).unwrap();
    let result = cli(&[
        "persistent-notes",
        "run",
        "--input",
        input.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
    assert_eq!(value["outcome"]["spent"], serde_json::json!([4, 5]));
    assert_eq!(value["outcome"]["net"], serde_json::json!([-4, 7]));
    fs::remove_file(&out).unwrap();
    let mut oracle = config;
    oracle.routine = sugarscape_core::persistent_notes::ReaderRoutine::Oracle;
    fs::write(&input, serde_json::to_vec(&oracle).unwrap()).unwrap();
    let result = cli(&[
        "persistent-notes",
        "run",
        "--input",
        input.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&result.stderr).contains("privileged Oracle"));
    fs::remove_dir_all(p).unwrap();
}
#[test]
fn invalid_scientific_config_has_distinct_status() {
    let p = dir("invalid");
    let input = p.join("input.json");
    let out = p.join("out.json");
    let mut config = sugarscape_core::persistent_notes::Config::example(
        sugarscape_core::persistent_notes::Arm::Retained,
        sugarscape_core::persistent_notes::ReaderRoutine::ReadAdaptive,
        sugarscape_core::persistent_notes::Good::Right,
    );
    config.stratum.acquisition_cost = 2;
    fs::write(&input, serde_json::to_vec(&config).unwrap()).unwrap();
    let result = cli(&[
        "persistent-notes",
        "run",
        "--input",
        input.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    fs::remove_dir_all(p).unwrap();
}
/// Engineering-only transport fixture: sealed arrays with deliberately invalid source receipts.
/// This does not run any episode, policy or collector and never materializes the report as a Value.
fn invalid_saved_fixture(path: &std::path::Path) {
    use std::io::{Read, Seek, SeekFrom, Write};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../sugarscape-core/src/persistent_notes/tests/fixtures");
    let small: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("limits-reference.json")).unwrap()).unwrap();
    let mut fixture = fs::File::open(root.join("evaluation-reference.json")).unwrap();
    let mut prefix = [0u8; 4096];
    fixture.read_exact(&mut prefix).unwrap();
    let marker = b"\"payload\":{";
    let start = prefix
        .windows(marker.len())
        .position(|w| w == marker)
        .unwrap()
        + marker.len();
    fixture.seek(SeekFrom::Start(start as u64)).unwrap();
    let mut output = std::io::BufWriter::with_capacity(65536, fs::File::create(path).unwrap());
    let zero = "0".repeat(64);
    let header = serde_json::json!({"version":"persistent-notes-diagnostic-v1","protocol_version":1,"protocol_id":"persistent-notes-v1","source":{"domain":"persistent-notes-source-v1","receipts":[],"sha256":zero},"reference":{"domain":"persistent-notes-reference-v1","receipts":[],"sha256":"0".repeat(64)},"limits":small["payload"]["limits"],"counters":small["payload"]["counters"],"payload_sha256":"0".repeat(64)});
    let mut header = serde_json::to_vec(&header).unwrap();
    assert_eq!(header.pop(), Some(b'}'));
    output.write_all(&header).unwrap();
    output.write_all(b",").unwrap();
    let mut reader = std::io::BufReader::with_capacity(65536, fixture);
    let mut depth = 1;
    let (mut quoted, mut escaped) = (false, false);
    let mut chunk = Vec::with_capacity(65536);
    for byte in reader.by_ref().bytes() {
        let byte = byte.unwrap();
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else {
            match byte {
                b'"' => quoted = true,
                b'{' | b'[' => depth += 1,
                b'}' | b']' => depth -= 1,
                _ => {}
            }
        }
        if depth == 0 {
            break;
        }
        chunk.push(byte);
        if chunk.len() == 65536 {
            output.write_all(&chunk).unwrap();
            chunk.clear();
        }
    }
    assert_eq!(depth, 0);
    output.write_all(&chunk).unwrap();
    output.write_all(b"}").unwrap();
    output.flush().unwrap();
}
#[test]
fn saved_scientific_invalidity_and_summary_failure_have_status_two() {
    let p = dir("saved-invalid");
    let input = p.join("report.json");
    let out = p.join("summary.json");
    invalid_saved_fixture(&input);
    let result = cli(&[
        "persistent-notes",
        "validate",
        "--input",
        input.to_str().unwrap(),
    ]);
    assert_eq!(
        result.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result = cli(&[
        "persistent-notes",
        "summary",
        "--input",
        input.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(fs::read(&out).unwrap().is_empty());
    fs::remove_dir_all(p).unwrap();
}
#[test]
fn duplicate_config_fields_have_operational_parse_status() {
    let p = dir("duplicate");
    let input = p.join("input.json");
    let out = p.join("out.json");
    let config = sugarscape_core::persistent_notes::Config::example(
        sugarscape_core::persistent_notes::Arm::Retained,
        sugarscape_core::persistent_notes::ReaderRoutine::ReadAdaptive,
        sugarscape_core::persistent_notes::Good::Right,
    );
    let json = serde_json::to_string(&config)
        .unwrap()
        .replacen('{', "{\"arm\":\"Erased\",", 1);
    fs::write(&input, json).unwrap();
    let result = cli(&[
        "persistent-notes",
        "run",
        "--input",
        input.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(1));
    fs::remove_dir_all(p).unwrap();
}
