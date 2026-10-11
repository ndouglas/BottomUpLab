"""Construction-only numerical/resource evidence; never collects science."""
import hashlib
import math
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from .archive_provenance import LIMITS, POPULATION, oracle_packet, validate_oracle_packet
from .contracts import Json, Manifest
from .dose_proof import dose_proof, validate_dose_proof
from .io import METADATA_LIMIT, _checked_path, canonical_bytes, create_bytes, read_regular
from .manifest import sample_manifest, validate_manifest
from .oracle import prepare_oracles
from .registration import artifact, load, native_metadata, save, source_inventory
from .resource_proof import forecast, schema_proof
from .supervisor import supervise, peak_bytes

PRESEED = Path("survey/war1_study/preparation/preseed-resource-readiness.json")


def _same_sources(fixture, scientific):
    def without_selected(metadata):
        return [entry for entry in metadata["inventory"]["entries"] if entry["path"] != metadata["selected_path"]]
    if without_selected(fixture) != without_selected(scientific):
        raise ValueError("fixture/scientific sources differ beyond selected manifest")
    for metadata in (fixture, scientific):
        if dict(metadata["flags"])["PROFILE"] != "release" or metadata["reference_calls_checked"] != 264:
            raise ValueError("release build and full pure reference-vocabulary proof required")
    if fixture["selected_purpose"] != "construction_fixture":
        raise ValueError("fixed construction binary required")


def _synthetic_probe(root: Path, out: Path):
    """Child workload: full synthetic manifest and bounded worst-byte archive.

    No OS sampler draws, no Engine. Content intentionally uses maximum u64
    duplicate seeds and labels this as synthetic engineering evidence.
    """
    started = time.monotonic()
    raw = sample_manifest(root, lambda n: b"\xff" * n, command=("construction_test_synthetic_resource_probe",))
    parsed = validate_manifest(raw, root)
    if len(parsed.cases) != 52224:
        raise ValueError("synthetic full manifest census")
    proof = schema_proof()
    # Maximum allowed compact/start encoded byte workloads; not scientific rows.
    start = b'{"construction_test":"' + b"s" * (1024 - 25) + b'"}\n'
    row = b'{"construction_test":"' + b"o" * (32768 - 25) + b'"}\n'
    beginning = time.monotonic()
    total = 0
    for i in range(128):
        path = out / f"synthetic-{i:03}.jsonl"
        data = start + row
        create_bytes(path, data, 33792)
        total += len(read_regular(path, 33792))
    archive_seconds = time.monotonic() - beginning
    # Exercise the full64MiB retained source bound in bounded increments.
    source_started = time.monotonic()
    for i in range(4):
        path = out / f"synthetic-source-{i}.bin"
        create_bytes(path, b"x" * METADATA_LIMIT, METADATA_LIMIT)
        hashlib.sha256(read_regular(path, METADATA_LIMIT)).hexdigest()
    source_seconds = time.monotonic() - source_started
    save(out / "synthetic-result.json", dict(schema="war1-study-synthetic-probe-v1", purpose="construction_fixture",
        label="synthetic engineering only; no sampled scientific population", schema_proof=proof,
        manifest_bytes=len(raw), manifest_cases=len(parsed.cases), archive_bytes=total,
        archive_seconds_per_byte=archive_seconds / total, source_seconds=source_seconds,
        wall_seconds=time.monotonic() - started))


def _run_synthetic(root, out):
    out.mkdir(mode=0o700)
    command = (sys.executable, "-c", "from pathlib import Path; from survey.war1_study.preparation import _synthetic_probe; import sys; _synthetic_probe(Path(sys.argv[1]), Path(sys.argv[2]))", str(root), str(out))
    child = subprocess.Popen(command, cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    started = time.monotonic()
    while True:
        pid, status, usage = os.wait4(child.pid, os.WNOHANG)
        if pid:
            child.returncode = os.waitstatus_to_exitcode(status)
            break
        if time.monotonic() - started > 120:
            os.kill(child.pid, 9)
            _, status, usage = os.wait4(child.pid, 0)
            child.returncode = os.waitstatus_to_exitcode(status)
            break
        time.sleep(.05)
    stderr = child.stderr.read(65537)
    child.stderr.close()
    if child.returncode != 0 or len(stderr) > 65536:
        raise ValueError(f"synthetic probe failed: exit{child.returncode}: {stderr[:1024]!r}")
    result = load(out / "synthetic-result.json")
    result["peak_rss_bytes"] = peak_bytes(usage.ru_maxrss, sys.platform)
    result["command"] = list(command)
    save(out / "synthetic-monitored.json", result)
    return result


def _native_pure(binary, command, manifest_path, output=None):
    args = (str(binary), command, "--manifest", str(manifest_path))
    if output is not None:
        args += ("--out", str(output))
    with subprocess.Popen(args, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE) as child:
        error = child.stderr.read(65537)
        if len(error) > 65536:
            child.kill()
            raise ValueError(f"native {command} error output exceeds bound")
        if child.wait() != 0:
            raise ValueError(f"native {command} failed: {error[:4096]!r}")


def prepare_evidence(root: Path, fixture_binary: Path, scientific_binary: Path,
                     manifest: Manifest, out: Path) -> tuple[Json, Json]:
    # Reject symlinks before normalizing relative/dot path spelling. The same
    # canonical arguments are launched, recorded, and bound by descriptors.
    root, fixture_binary, scientific_binary, out = (
        _checked_path(path).resolve() for path in (root, fixture_binary, scientific_binary, out))
    fixture = native_metadata(fixture_binary)
    scientific = native_metadata(scientific_binary)
    _same_sources(fixture, scientific)
    if Path(scientific["root"]) != root.absolute() or scientific["binding"]["manifest_sha256"] != manifest.sha256:
        raise ValueError("preparation source/manifest binding")
    manifest_path = root / scientific["selected_path"]
    if read_regular(manifest_path, METADATA_LIMIT) != manifest.raw_bytes:
        raise ValueError("selected manifest bytes mismatch")
    proof = schema_proof()
    out.mkdir(mode=0o700, exist_ok=False)
    _native_pure(scientific_binary, "validate", manifest_path)
    _native_pure(scientific_binary, "numerics", manifest_path, out / "numeric-export.json")
    export = load(out / "numeric-export.json")
    if export["compiled_inputs_sha256"] != scientific["binding"]["compiled_inputs_sha256"]:
        raise ValueError("numeric export source binding")
    save(out / "dose-proof.json", dose_proof(root, manifest.sha256, export["compiled_inputs_sha256"]))
    oracles = prepare_oracles(manifest, export)
    if any(o.ctmc_one_vs_one and o.ctmc_one_vs_one["required_at_this_dt"] and not all(v["passes"] is True for v in o.ctmc_one_vs_one["criteria"].values()) for o in oracles):
        raise ValueError("required CTMC criterion failed")
    save(out / "oracle-packet.json", oracle_packet(manifest, export["compiled_inputs_sha256"], oracles))
    binary = artifact(scientific_binary, 1024**3)
    numeric = dict(schema="war1-study-numeric-v1", manifest_sha256=manifest.sha256,
        compiled_inputs_sha256=export["compiled_inputs_sha256"], binary_sha256=binary["sha256"],
        dose_proof=artifact(out / "dose-proof.json"),
        numeric_export=artifact(out / "numeric-export.json"), oracle_packet=artifact(out / "oracle-packet.json"),
        certificate_valid=True, mass_valid=True, width_valid=True, ulp_valid=True, ctmc_valid=True,
        unresolved_findings=[], oracle_ids=[o.arm_id for o in oracles])
    save(out / "numeric.json", numeric)
    command = (str(fixture_binary), "fixture", "--manifest", str(root / fixture["selected_path"]), "--out", str(out / "fixture-archive"))
    monitored = supervise(command, out / "fixture-archive", 43200, 2147483648)
    if monitored["schema"] != "war1-study-monitor-result-v1":
        raise ValueError("fixed construction probe failed; no resource readiness")
    metrics = load(out / "fixture-archive/fixture-probe.json")
    synthetic = _run_synthetic(root, out / "synthetic")
    synthetic["peak_rss_bytes"] = max(synthetic["peak_rss_bytes"], monitored["peak_rss_bytes"])
    # Fixed closure/startup/final archive validation measured separately from
    # frame costs; double-counting is conservative and intentional.
    synthetic["source_seconds"] = float(synthetic["source_seconds"]) + float(monitored["wall_seconds"])
    measured_forecast = forecast(metrics["cases"], synthetic)
    inputs = dict(schema="war1-study-resource-forecast-inputs-v1", purpose="construction_fixture",
        schema_proof=proof, fixture_metadata=fixture, scientific_metadata=scientific,
        measured_cases=metrics["cases"], synthetic=synthetic, forecast=measured_forecast,
        fixture_binary=artifact(fixture_binary, 1024**3), fixture_monitor=artifact(out / "fixture-archive/monitor.json"),
        fixture_probe=artifact(out / "fixture-archive/fixture-probe.json"), synthetic_probe=artifact(out / "synthetic/synthetic-monitored.json"))
    save(out / "resource-forecast-inputs.json", inputs)
    probes = [dict(artifact=artifact(out / "fixture-archive/fixture-probe.json"), command=list(command),
        compiled_inputs_sha256=fixture["binding"]["compiled_inputs_sha256"], binary_sha256=fixture["binding"]["executable_sha256"],
        wall_seconds=monitored["wall_seconds"], peak_rss_bytes=monitored["peak_rss_bytes"], fixture_purpose="construction_fixture")]
    resources = dict(schema="war1-study-resources-v1", manifest_sha256=manifest.sha256,
        compiled_inputs_sha256=scientific["binding"]["compiled_inputs_sha256"], binary_sha256=binary["sha256"],
        budgets=LIMITS, census=POPULATION, reserved_bytes=proof["reserved_bytes"], failure_reserve_bytes=proof["failure_reserve_bytes"],
        schema_bounds_valid=True, aggregate_proof_valid=True, forecast_valid=True,
        forecast_method="maximum measured kernel/actor, projection/step, start+outcome, constructor/terminal, byte/source costs;4x margin",
        forecast_inputs=artifact(out / "resource-forecast-inputs.json"), forecast_wall_seconds=measured_forecast["wall_seconds"],
        forecast_peak_rss_bytes=measured_forecast["peak_rss_bytes"], fixture_probes=probes, unavailable=[])
    save(out / "resources.json", resources)
    if manifest.purpose == "construction_fixture":
        save(out / "preseed-resource-readiness.json", dict(schema="war1-study-preseed-resources-v1", purpose="construction_fixture",
            registered_histories=0, dose_proof=artifact(out / "dose-proof.json"), resources=artifact(out / "resources.json"), forecast_inputs=artifact(out / "resource-forecast-inputs.json")))
    return numeric, resources


def validate_preseed(root: Path):
    from .manifest import _fields
    from .archive_provenance import validate_artifact_descriptor, validate_resource_receipt
    gate = load(root / PRESEED)
    _fields(gate, {"schema", "purpose", "registered_histories", "resources", "forecast_inputs", "dose_proof"}, "preseed gate")
    if gate["schema"] != "war1-study-preseed-resources-v1" or gate["purpose"] != "construction_fixture" or type(gate["registered_histories"]) is not int or gate["registered_histories"] != 0:
        raise ValueError("construction preseed resource gate required")
    def linked(descriptor):
        validate_artifact_descriptor(descriptor)
        if artifact(Path(descriptor["path"])) != descriptor:
            raise ValueError("preseed artifact changed")
        return load(Path(descriptor["path"]))
    resource = linked(gate["resources"])
    validate_resource_receipt(resource, {key: resource[key] for key in ("manifest_sha256", "compiled_inputs_sha256", "binary_sha256")})
    if gate["forecast_inputs"] != resource["forecast_inputs"]:
        raise ValueError("preseed forecast descriptor differs from resource receipt")
    inputs = linked(gate["forecast_inputs"])
    metadata = native_metadata(Path(inputs["fixture_binary"]["path"]))
    if Path(metadata["root"]) != root.absolute():
        raise ValueError("preseed gate belongs to a different source root")
    validate_dose_proof(linked(gate["dose_proof"]), root, metadata["binding"]["manifest_sha256"],
                        metadata["binding"]["compiled_inputs_sha256"])
    probe = linked(inputs["fixture_probe"])
    validate_resource_evidence(resource, inputs, probe, metadata)
    return resource


def validate_resource_evidence(resource, inputs, probe, metadata):
    """Recompute the operational resource decision, beyond receipt flag syntax.

    Used before declaration and before the supervisor starts a scientific child.
    Saved-only analysis still uses the shared retained-schema validator.
    """
    from .manifest import _fields
    from .archive_provenance import validate_resource_receipt, validate_artifact_descriptor
    ids = dict(manifest_sha256=metadata["binding"]["manifest_sha256"],
        compiled_inputs_sha256=metadata["binding"]["compiled_inputs_sha256"], binary_sha256=metadata["binding"]["executable_sha256"])
    validate_resource_receipt(resource, ids)
    if artifact(Path(resource["forecast_inputs"]["path"])) != resource["forecast_inputs"] or canonical_bytes(load(Path(resource["forecast_inputs"]["path"]))) != canonical_bytes(inputs):
        raise ValueError("resource forecast raw artifact/input binding")
    _fields(inputs, {"schema", "purpose", "schema_proof", "fixture_metadata", "scientific_metadata", "measured_cases", "synthetic", "forecast", "fixture_binary", "fixture_monitor", "fixture_probe", "synthetic_probe"}, "forecast inputs")
    if inputs["schema"] != "war1-study-resource-forecast-inputs-v1" or inputs["purpose"] != "construction_fixture":
        raise ValueError("construction forecast input schema")
    fixture = native_metadata(Path(inputs["fixture_binary"]["path"]))
    if fixture != inputs["fixture_metadata"] or artifact(Path(inputs["fixture_binary"]["path"]), 1024**3) != inputs["fixture_binary"]:
        raise ValueError("resource fixture binary/source changed")
    if metadata != inputs["scientific_metadata"]:
        raise ValueError("resource scientific metadata differs from actual selected executable")
    _same_sources(fixture, metadata)
    if source_inventory(Path(metadata["root"]), metadata["selected_path"]) != metadata["inventory"]:
        raise ValueError("resource source inventory changed")
    proof = schema_proof()
    if inputs["schema_proof"] != proof or resource["reserved_bytes"] != proof["reserved_bytes"] or resource["failure_reserve_bytes"] != proof["failure_reserve_bytes"]:
        raise ValueError("resource algebraic proof/reservation mismatch")
    if probe.get("schema") != "war1-study-fixture-probe-v1" or probe.get("purpose") != "construction_fixture" or probe.get("cases") != inputs["measured_cases"]:
        raise ValueError("resource measured fixed fixture cases mismatch")
    for descriptor in (inputs["fixture_monitor"], inputs["fixture_probe"], inputs["synthetic_probe"]):
        validate_artifact_descriptor(descriptor)
        if artifact(Path(descriptor["path"])) != descriptor:
            raise ValueError("resource probe descriptor changed")
    monitor = load(Path(inputs["fixture_monitor"]["path"]))
    from .archive_provenance import validate_monitor
    validate_monitor(monitor, dict(mode="construction_fixture", binding=fixture["binding"]))
    if len(resource["fixture_probes"]) != 1:
        raise ValueError("resource operational fixed probe census")
    receipt_probe = resource["fixture_probes"][0]
    expected_command = [inputs["fixture_binary"]["path"], "fixture", "--manifest",
        str(Path(fixture["root"]) / fixture["selected_path"]), "--out", str(Path(inputs["fixture_probe"]["path"]).parent)]
    if (receipt_probe["artifact"] != inputs["fixture_probe"]
        or canonical_bytes(load(Path(inputs["fixture_probe"]["path"]))) != canonical_bytes(probe)
        or receipt_probe["compiled_inputs_sha256"] != fixture["binding"]["compiled_inputs_sha256"]
        or receipt_probe["binary_sha256"] != fixture["binding"]["executable_sha256"]
        or receipt_probe["command"] != expected_command
        or canonical_bytes(dict(wall=receipt_probe["wall_seconds"])) != canonical_bytes(dict(wall=monitor["wall_seconds"]))
        or receipt_probe["peak_rss_bytes"] != monitor["peak_rss_bytes"]):
        raise ValueError("resource fixed probe receipt/metadata binding")
    synthetic = load(Path(inputs["synthetic_probe"]["path"]))
    if synthetic.get("schema") != "war1-study-synthetic-probe-v1" or synthetic.get("purpose") != "construction_fixture" or synthetic.get("manifest_cases") != 52224 or synthetic.get("schema_proof") != proof:
        raise ValueError("synthetic full-workload proof mismatch")
    synthetic["source_seconds"] = float(synthetic["source_seconds"]) + float(monitor["wall_seconds"])
    synthetic["peak_rss_bytes"] = max(synthetic["peak_rss_bytes"], monitor["peak_rss_bytes"])
    expected = forecast(probe["cases"], synthetic)
    if canonical_bytes(synthetic) != canonical_bytes(inputs["synthetic"]) or canonical_bytes(expected) != canonical_bytes(inputs["forecast"]) or float(resource["forecast_wall_seconds"]) != expected["wall_seconds"] or resource["forecast_peak_rss_bytes"] != expected["peak_rss_bytes"]:
        raise ValueError("resource forecast differs from measured-work recomputation")
    for key, value in (("manifest_sha256", metadata["binding"]["manifest_sha256"]), ("compiled_inputs_sha256", metadata["binding"]["compiled_inputs_sha256"]), ("binary_sha256", metadata["binding"]["executable_sha256"])):
        if resource[key] != value:
            raise ValueError("resource current source/binary binding")
    if resource["unavailable"] or any(resource[k] is not True for k in ("schema_bounds_valid", "aggregate_proof_valid", "forecast_valid")):
        raise ValueError("resource proof unavailable")
    return expected
