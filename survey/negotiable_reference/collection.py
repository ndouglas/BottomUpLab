"""Bounded, explicit first/repeat collection of the frozen negotiable report.

Certification hashes inputs without invoking the survey. Only run-first-repeat
executes its fixed command. Failed attempts and their ownership lock are retained.
RSS monitoring is sampled, with final wait4 peak evidence; it is not a hard RSS
allocation limit on macOS. No invocation is retried.
"""

import argparse
from dataclasses import dataclass
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import resource
import selectors
import signal
import stat
import subprocess
import sys
import time

from . import reference

SOURCE_ROOT = Path("/Users/nathan/Projects/ndouglas/SugarScape")
EVIDENCE = Path("/Users/nathan/.config/superpowers/worktrees/SugarScape/negotiable-alignment-design/.superpowers/negotiable-assistance")
APPROVED_COMMIT = "9e35bba81bb099810f253c1cbaae52b5d9168548"
SOURCE_SHA256 = "dede454af44b33817fcf14cbd8688633abfcd1c901e744c95c4ad676a6debf70"
BINARY = SOURCE_ROOT / "survey/target/release/survey"
LEGACY = SOURCE_ROOT / "survey/out/results-negotiable.json"
ARCHIVES = EVIDENCE / "collection"
COMMAND = [str(BINARY), "--only", "negotiable", "--seeds", "1"]
CHILD_ENVIRONMENT = {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C", "TZ": "UTC"}
STAMP = SOURCE_ROOT / "survey/target/release/build/survey-6bf75bec43bebd5f/out/f5_source_identity.rs"
BT_STAMP = STAMP.with_name("bt_compiled_inputs.rs")
WIRE_BOUNDS = {"cake": 120933, "pareto": 16247, "priority": 1387111}
REFERENCE_HASHES = {
    "cake": "42e738fe443809b7598e61c3d80879ab975147e435fd32c3c7e1d0703573da76",
    "pareto": "fafca8c4c1c19009aa3ec78b171e39cc3e6a754aba265d387c39e564125e73ae",
    "priority": "0e2e19f4d7847a6277d99394b27d3c142fc1fc82d5367e54160227117c1d4b41",
}
REPORT_FIELDS = {"id", "item", "source", "citation", "text", "verdict", "measured", "detail", "seconds"}
VERDICTS = {"Holds", "Weak", "Inconclusive", "Fails", "Untestable", "Error"}
SCHEMA = "negotiable-collection-certificate-v1"


@dataclass(frozen=True)
class _Limits:
    seconds: float = 120
    rss_bytes: int = 512 * 1024 * 1024
    total_bytes: int = 32 * 1024 * 1024
    sample_seconds: float = 0.05


def _sha(data):
    return hashlib.sha256(data).hexdigest()


def _json(data):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError(f"duplicate JSON key {key!r}")
            result[key] = value
        return result
    def constant(value):
        raise ValueError(f"invalid JSON constant {value}")
    try:
        return json.loads(data, object_pairs_hook=pairs, parse_constant=constant)
    except (UnicodeError, RecursionError, TypeError) as error:
        raise ValueError(f"invalid JSON: {error}") from error


def _regular(path, maximum=None):
    flags = os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK
    fd = os.open(path, flags)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode):
            raise ValueError(f"not a regular file: {path}")
        if maximum is not None and info.st_size > maximum:
            raise ValueError(f"retained bytes limit: {path}")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            data = stream.read() if maximum is None else stream.read(maximum + 1)
        if maximum is not None and len(data) > maximum:
            raise ValueError(f"retained bytes limit: {path}")
        return data
    finally:
        os.close(fd)


def _identity(path):
    data = _regular(path)
    return {"path": str(path), "bytes": len(data), "sha256": _sha(data)}


def _binding(kind, source):
    return {
        "main_pdf_sha256": reference.SOURCE_HASHES["main_pdf"],
        "supplement_pdf_sha256": reference.SOURCE_HASHES["supplement_pdf"],
        "reference_sha256": dict(REFERENCE_HASHES),
        "settings_sha256": _sha(reference.canonical_bytes(reference.settings(kind))),
        "compiled_source_sha256": source,
    }


def validate_report(data, source):
    """Validate all exact records; preserve typed outcomes and explicit gaps."""
    rows = _json(data)
    ids = {f"negotiable.{kind}" for kind in reference.ROW_COUNTS}
    if type(rows) is not list or len(rows) != 3:
        raise ValueError("claim census must contain exactly three claims")
    seen, records, verdicts, gaps, counts = set(), {}, {}, [], {}
    for row in rows:
        if type(row) is not dict or set(row) != REPORT_FIELDS:
            raise ValueError("report fields differ from fixed claim envelope")
        claim = row["id"]
        if type(claim) is not str or claim not in ids or claim in seen:
            raise ValueError("unexpected or duplicate claim ID")
        seen.add(claim)
        kind = claim.split(".")[1]
        if (row["item"] != "negotiable" or type(row["source"]) is not str or
                row["source"] not in {"Book", "App", "Comment"}):
            raise ValueError(f"{claim}: invalid metadata")
        if any(type(row[name]) is not str or len(row[name].encode()) > 16384
               for name in ("citation", "text", "detail")):
            raise ValueError(f"{claim}: invalid metadata text")
        seconds = row["seconds"]
        if type(seconds) not in (float, int) or not 0 <= seconds <= 120 or not math.isfinite(seconds):
            raise ValueError(f"{claim}: invalid seconds")
        verdict = row["verdict"]
        if type(verdict) is not str or verdict not in VERDICTS:
            raise ValueError(f"{claim}: invalid verdict")
        verdicts[claim] = verdict
        measured = row["measured"]
        if type(measured) is not str:
            raise ValueError(f"{claim}: measured must be JSON text")
        if not measured:
            if verdict == "Holds":
                raise ValueError(f"{claim}: empty Holds record")
            gaps.append(claim)
            continue
        if len(measured.encode()) > WIRE_BOUNDS[kind]:
            raise ValueError(f"{claim}: record wire bound exceeded")
        record = _json(measured)
        if type(record) is not dict or set(record) != {"schema", "scope", "binding", "settings", "rows"}:
            raise ValueError(f"{claim}: invalid record envelope")
        if record["scope"] != "finite_exact_foundation" or record["binding"] != _binding(kind, source):
            raise ValueError(f"{claim}: stale or malformed binding")
        mathematical = {name: record[name] for name in ("schema", "settings", "rows")}
        reference.validate_reference(mathematical, kind)
        records[kind] = record
        counts[kind] = len(record["rows"])
    return {"records": records, "verdicts": verdicts, "gaps": sorted(gaps), "row_counts": counts,
            "complete": not gaps and "Error" not in verdicts.values()}


def packing_certificate(provenance):
    """Conservative complete packing, including both raw copies and escaping.

    JSON escaping uses at most six ASCII bytes per input byte. Fixed outer
    metadata allows three fields of 16 KiB per claim plus identifiers/seconds.
    stdout contains the unescaped records and metadata; stderr is bounded.
    Before/after provenance allowances include the actual certificate envelope.
    Runtime enforces the total independently of this predicted packing.
    """
    record_bytes = sum(WIRE_BOUNDS.values())
    metadata = 3 * (3 * 16384 + 1024)
    outer = 6 * record_bytes + 6 * metadata + 4096
    snapshot = len(reference.canonical_bytes(provenance)) + 65536
    components = {
        "stdout.log": record_bytes + metadata + 4096,
        "report.json": outer,
        "canonical.json": record_bytes + 65536,
        "stderr.log": 65536,
        "provenance-before.json": snapshot,
        "provenance-after.json": snapshot,
        "status.json": 65536,
    }
    total = sum(components.values())
    if total > _Limits().total_bytes:
        raise ValueError("worst-case retained packing exceeds 32 MiB")
    return {"wire_bounds": dict(WIRE_BOUNDS), "escape_factor": 6,
            "outer_report_bytes": outer, "components": components, "total_bytes": total,
            "headroom_bytes": _Limits().total_bytes - total}


def _source_inputs(root):
    """Independent implementation of the frozen Rust F5 selected-input hash."""
    selector = _regular(root / "survey/build_support/f5_source_identity.rs").decode()
    def paths(name):
        match = re.search(rf"pub const {name}:.*?= &\[(.*?)\];", selector, re.S)
        if not match:
            raise ValueError(f"source selector lacks {name}")
        # Ignore Rust line comments before extracting literals.
        body = re.sub(r"//[^\n]*", "", match[1])
        return re.findall(r'"([^"\n]+)"', body)
    directories, files = paths("DIRECTORIES"), paths("FILES")
    selected = set()
    for directory in directories:
        base = root / directory
        if base.is_symlink():
            raise ValueError("symlink source directory")
        for parent, dirs, names in os.walk(base, followlinks=False):
            if any((Path(parent) / name).is_symlink() for name in dirs):
                raise ValueError("symlink source directory")
            for name in names:
                path = Path(parent) / name
                if path.is_symlink():
                    raise ValueError("symlink source input")
                relative = path.relative_to(root).as_posix()
                if relative.endswith(".rs") or relative in files:
                    selected.add(relative)
    for relative in files:
        if os.path.lexists(root / relative):
            selected.add(relative)
    digest = hashlib.sha256(b"foraging-shortcut-source-v1\0")
    inventory = []
    for relative in sorted(selected):
        data = _regular(root / relative)
        encoded = relative.encode()
        digest.update(len(encoded).to_bytes(8, "big"))
        digest.update(encoded)
        digest.update(len(data).to_bytes(8, "big"))
        digest.update(data)
        inventory.append({"path": relative, "bytes": len(data), "sha256": _sha(data)})
    return digest.hexdigest(), inventory


def certify(*, _allow_drift=False):
    """Hash and certify fixed settings only; never execute the scientific child."""
    source, inputs = _source_inputs(SOURCE_ROOT)
    if not _allow_drift and source != SOURCE_SHA256:
        raise ValueError("approved selected source inputs changed")
    head = subprocess.check_output(["git", "-C", str(SOURCE_ROOT), "rev-parse", "HEAD"], text=True).strip()
    if not _allow_drift and head != APPROVED_COMMIT:
        raise ValueError("source root is not the approved merged commit")
    stamp = _regular(STAMP)
    if not _allow_drift and stamp != f'const COMPILED_SOURCE_IDENTITY: &str = "{source}";\n'.encode():
        raise ValueError("compiled source stamp changed")
    binary = _identity(BINARY)
    if not _allow_drift and SOURCE_SHA256.encode() not in _regular(BINARY):
        raise ValueError("compiled source identity absent from release binary")
    fixtures = {}
    for kind, expected in REFERENCE_HASHES.items():
        path = SOURCE_ROOT / f"crates/sugarscape-core/src/negotiable/tests/fixtures/{kind}-reference.json"
        identity = _identity(path)
        if not _allow_drift and identity["sha256"] != expected:
            raise ValueError("sealed reference changed")
        if _allow_drift:
            try:
                record = _json(_regular(path))
            except ValueError:
                record = {"settings": None}
        else:
            record = reference.decode_reference(_regular(path), kind)
        fixtures[kind] = {**identity, "settings_sha256": _sha(reference.canonical_bytes(record.get("settings") if type(record) is dict else None))}
    pdfs = {}
    for name in ("main", "supplement"):
        identity = _identity(EVIDENCE / f"sources/{name}.pdf")
        if not _allow_drift and identity["sha256"] != reference.SOURCE_HASHES[f"{name}_pdf"]:
            raise ValueError("original PDF changed")
        pdfs[name] = identity
    package = Path(__file__).resolve().parent
    collector = [_identity(package / name) for name in ("collection.py", "test_collection.py", "reference.py", "test_reference.py", "__init__.py")]
    compiler = subprocess.check_output(["rustc", "-vV"], text=True)
    cargo = subprocess.check_output(["cargo", "-V"], text=True)
    provenance = {
        "schema": SCHEMA, "source_root": str(SOURCE_ROOT), "approved_commit": APPROVED_COMMIT,
        "source_head": head,
        "source_sha256": source, "selected_inputs": inputs, "binary": binary,
        "compiled_stamps": [_identity(STAMP), _identity(BT_STAMP)],
        "release_receipt": _identity(EVIDENCE / "merge-integration/release-build.status.json"),
        "wire_certificate_source": _identity(EVIDENCE / "execution-record/sdd/task-4-report.md"),
        "reference_seal": _identity(EVIDENCE / "reference-seal-v2.json"),
        "fixtures": fixtures, "original_pdfs": pdfs, "collector": collector,
        "command": list(COMMAND), "child_environment": dict(CHILD_ENVIRONMENT),
        "legacy_path": str(LEGACY), "archives": str(ARCHIVES),
        "limits": _Limits().__dict__,
        "toolchain": {"python": sys.version, "python_executable": _identity(Path(sys.executable).resolve()),
                      "rustc": compiler, "cargo": cargo, "platform": platform.platform(),
                      "system": platform.uname()._asdict(),
                      "environment": {key: os.environ.get(key, "") for key in
                                      ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "DYLD_LIBRARY_PATH", "DYLD_INSERT_LIBRARIES")}},
        "rss_measurement": {"method": "process-group ps RSS sampling plus final wait4 direct-child peak",
                            "sampling_seconds": 0.05, "hard_allocation_limit": False,
                            "limitations": "Transient group RSS can exceed the cap between samples; wait4 rejects direct-child peak overshoot after exit. Killing a process group cannot contain a descendant that deliberately starts another session."},
    }
    provenance["packing"] = packing_certificate(provenance)
    return provenance


def _exclusive(path, data):
    with Path(path).open("xb") as output:
        output.write(data)
        output.flush()
        os.fsync(output.fileno())
    if _regular(path, len(data)) != data:
        raise ValueError("archive readback differs")


def _group_rss(group):
    output = subprocess.check_output(["ps", "-axo", "pgid=,rss="], text=True)
    total, members = 0, []
    for line in output.splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[0] == str(group):
            total += int(fields[1]) * 1024
            members.append(fields)
    return total, len(members)


def _kill_group(group):
    try:
        os.killpg(group, signal.SIGKILL)
    except ProcessLookupError:
        pass


def _run_child(command, legacy, directory, limits):
    """Private fake-child boundary; the public CLI never accepts a command."""
    # Reserve one third each for pipe bytes and the legacy report. Metadata and
    # canonical output use the remainder, with a separate aggregate final check.
    pipe_cap = limits.total_bytes // 3
    report_cap = limits.total_bytes // 3
    def child_limits():
        resource.setrlimit(resource.RLIMIT_FSIZE, (report_cap, report_cap))
        # RSS enforcement deliberately does not use ineffective macOS RLIMIT_RSS.
    started = time.monotonic()
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               start_new_session=True, preexec_fn=child_limits,
                               cwd=str(legacy.parent), env=CHILD_ENVIRONMENT.copy())
    selector = selectors.DefaultSelector()
    streams = {}
    for name, pipe in (("stdout.log", process.stdout), ("stderr.log", process.stderr)):
        os.set_blocking(pipe.fileno(), False)
        selector.register(pipe, selectors.EVENT_READ, name)
        streams[name] = (directory / name).open("xb")
    captured = 0
    peak_sample = 0
    usage = None
    returncode = None
    error = ""
    killed = False
    try:
        while returncode is None or selector.get_map():
            elapsed = time.monotonic() - started
            if elapsed > limits.seconds and not error:
                error = "elapsed seconds limit exceeded"
            rss, _ = _group_rss(process.pid)
            peak_sample = max(peak_sample, rss)
            if rss > limits.rss_bytes and not error:
                error = "observed process-group RSS limit exceeded"
            try:
                if legacy.lstat().st_size > report_cap and not error:
                    error = "legacy report bytes limit exceeded"
            except FileNotFoundError:
                if not error:
                    error = "owned staging path disappeared"
            if error and not killed:
                _kill_group(process.pid)
                killed = True
            if returncode is None:
                pid, status, usage_now = os.wait4(process.pid, os.WNOHANG)
                if pid:
                    usage = usage_now
                    returncode = os.waitstatus_to_exitcode(status)
                    process.returncode = returncode
                    _, children = _group_rss(process.pid)
                    if children and not error:
                        error = "lingering process-group descendant"
                        _kill_group(process.pid)
                        killed = True
            for key, _ in selector.select(limits.sample_seconds):
                data = os.read(key.fileobj.fileno(), 65536)
                if not data:
                    selector.unregister(key.fileobj)
                    key.fileobj.close()
                    continue
                available = max(0, pipe_cap - captured)
                retained = data[:available]
                streams[key.data].write(retained)
                captured += len(retained)
                if len(data) > available and not error:
                    error = "captured pipe bytes limit exceeded; truncated retained output"
            if returncode is not None and not selector.get_map():
                break
    finally:
        if not killed:
            _kill_group(process.pid)
        if returncode is None:
            _, status, usage = os.wait4(process.pid, 0)
            returncode = os.waitstatus_to_exitcode(status)
            process.returncode = returncode
        selector.close()
        for output in streams.values():
            output.flush()
            os.fsync(output.fileno())
            output.close()
        for pipe in (process.stdout, process.stderr):
            pipe.close()
    peak = usage.ru_maxrss * (1 if sys.platform == "darwin" else 1024)
    if peak > limits.rss_bytes and not error:
        error = "final wait4 direct-child peak RSS limit exceeded"
    if time.monotonic() - started > limits.seconds and not error:
        error = "final elapsed seconds limit exceeded"
    if returncode and not error:
        error = f"child exited {returncode}"
    return {"returncode": returncode, "elapsed_seconds": time.monotonic() - started,
            "direct_child_peak_rss_bytes": peak, "sampled_group_peak_rss_bytes": peak_sample,
            "captured_pipe_bytes": captured, "error": error,
            "rss_method": "ps group RSS every <= nominal 50ms plus wait4 peak (ps latency adds sampling delay)"}


def _archive_raw_report(legacy, owned, directory, cap):
    """Retain only the owned regular inode, even when later checks fail."""
    fd = os.open(legacy, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        info = os.fstat(fd)
        if (not stat.S_ISREG(info.st_mode) or
                (info.st_dev, info.st_ino) != (owned.st_dev, owned.st_ino)):
            raise ValueError("owned staging inode was replaced")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            data = stream.read(cap + 1)
        raw = data[:cap]
        receipt = {"bytes": len(raw), "staging_bytes": info.st_size,
                   "truncated": len(data) > cap or info.st_size > cap,
                   "at_byte_limit": len(raw) == cap}
        _exclusive(directory / "report.json", raw)
        return raw, receipt
    finally:
        os.close(fd)


def _collect_pair(command, legacy, archives, snapshot, limits, *, reviewed):
    """Create-only collector core. A retained failed attempt stops both runs."""
    legacy, archives = Path(legacy), Path(archives)
    lock = legacy.with_name(legacy.name + ".collection-lock")
    if os.path.lexists(legacy) or os.path.lexists(lock):
        raise FileExistsError("legacy report or collection ownership already exists")
    for parent in (legacy.parent, archives, *archives.parents):
        if parent.is_symlink():
            raise ValueError(f"symlink destination directory: {parent}")
    if any(os.path.lexists(archives / name) for name in ("first", "repeat")):
        raise FileExistsError("attempt exists; retry prohibited")
    encoded_before = reference.canonical_bytes(reviewed)
    before = _json(encoded_before)
    if reference.canonical_bytes(snapshot()) != encoded_before:
        raise ValueError("reviewed identity changed before campaign")
    if len(encoded_before) > limits.total_bytes // 16:
        raise ValueError("provenance bytes exceed reserved allowance")
    legacy.parent.mkdir(parents=True, exist_ok=True)
    lock.mkdir()
    # A symlink-free create-only staging inode is owned throughout each run.
    archived = []
    successful = False
    try:
        if os.path.lexists(legacy):
            raise FileExistsError("legacy appeared during ownership acquisition")
        for name in ("first", "repeat"):
            current = snapshot()
            if reference.canonical_bytes(current) != encoded_before:
                raise ValueError("certified identity changed before attempt")
            directory = archives / name
            directory.mkdir(parents=True)
            status = {"state": "failed", "attempt": name, "command": list(command),
                      "limits": limits.__dict__, "error": "attempt interrupted"}
            staging_info = None
            try:
                _exclusive(directory / "provenance-before.json", encoded_before)
                fd = os.open(legacy, os.O_CREAT | os.O_EXCL | os.O_WRONLY | os.O_NOFOLLOW, 0o600)
                staging_info = os.fstat(fd)
                os.close(fd)
                status.update(_run_child(command, legacy, directory, limits))
                raw, status["raw_report"] = _archive_raw_report(
                    legacy, staging_info, directory, limits.total_bytes // 3)
                if status["raw_report"]["truncated"]:
                    status["error"] = "legacy report bytes limit exceeded; retained prefix truncated"
                after = snapshot()
                encoded_after = reference.canonical_bytes(after)
                if len(encoded_after) > limits.total_bytes // 16:
                    raise ValueError("after provenance bytes exceed reserved allowance")
                _exclusive(directory / "provenance-after.json", encoded_after)
                if reference.canonical_bytes(after) != encoded_before:
                    raise ValueError("source, binary or collector identity changed during child")
                if status["error"]:
                    raise ValueError(status["error"])
                validation = validate_report(raw, before["source_sha256"])
                # Canonical mathematical records retain typed outcomes but omit timings.
                canonical = reference.canonical_bytes(validation)
                _exclusive(directory / "canonical.json", canonical)
                status["validation"] = {key: value for key, value in validation.items() if key != "records"}
                if not validation["complete"]:
                    raise ValueError("incomplete record census or operational Error; typed outcomes retained")
                if len(canonical) > limits.total_bytes // 8:
                    raise ValueError("canonical bytes exceed reserved allowance")
                status["state"] = "accepted"
                status["error"] = ""
            except (ValueError, OSError, subprocess.SubprocessError) as error:
                status["state"] = "failed"
                status["error"] = str(error)[:2048]
            manifest = {path.name: _identity(path) for path in directory.iterdir() if path.is_file()}
            status["files"] = manifest
            body = reference.canonical_bytes(status)
            total = sum(value["bytes"] for value in manifest.values()) + len(body)
            status["retained_bytes"] = total
            body = reference.canonical_bytes(status)
            status["retained_bytes"] = sum(value["bytes"] for value in manifest.values()) + len(body)
            body = reference.canonical_bytes(status)
            if len(body) > 65536 or status["retained_bytes"] > limits.total_bytes:
                raise ValueError("aggregate retained bytes exceeded; attempt remains failed and owned")
            _exclusive(directory / "status.json", body)
            # Verify every archive byte before releasing the owned staging inode.
            for filename, identity in manifest.items():
                if _identity(directory / filename) != identity:
                    raise ValueError("archive identity verification failed")
            if status["state"] != "accepted":
                return {"state": "failed", "attempt": name, "error": status["error"]}
            if _sha(_regular(legacy, limits.total_bytes // 3)) != manifest["report.json"]["sha256"]:
                raise ValueError("owned staging bytes changed before release")
            final_info = legacy.lstat()
            if (final_info.st_dev, final_info.st_ino) != (staging_info.st_dev, staging_info.st_ino):
                raise ValueError("owned staging inode changed before release")
            legacy.unlink()
            archived.append(_regular(directory / "canonical.json", limits.total_bytes // 8))
        successful = True
        return {"state": "accepted", "canonical_equal": archived[0] == archived[1],
                "first_sha256": _sha(archived[0]), "repeat_sha256": _sha(archived[1])}
    finally:
        if successful:
            lock.rmdir()


def run_first_repeat(certificate_path):
    reviewed = _json(_regular(certificate_path, 2 * 1024 * 1024))
    current = certify()
    if reference.canonical_bytes(reviewed) != reference.canonical_bytes(current):
        raise ValueError("reviewed certificate changed; scoped review required")
    def snapshot():
        # Preserve independently hashed drift evidence before the core refuses it.
        return certify(_allow_drift=True)
    return _collect_pair(COMMAND, LEGACY, ARCHIVES, snapshot, _Limits(), reviewed=reviewed)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest="action", required=True)
    preflight = subs.add_parser("certify", help="hash inputs and packing only; no scientific execution")
    preflight.add_argument("--certificate", type=Path, required=True)
    run = subs.add_parser("run-first-repeat", help="explicit fixed command, one attempt each")
    run.add_argument("--certificate", type=Path, required=True)
    args = parser.parse_args()
    if args.action == "certify":
        certificate = certify()
        _exclusive(args.certificate, reference.canonical_bytes(certificate))
        print(json.dumps({"certificate": str(args.certificate), "sha256": _identity(args.certificate)["sha256"],
                          "packing": certificate["packing"]}, indent=2))
    else:
        result = run_first_repeat(args.certificate)
        print(json.dumps(result, indent=2))
        if result["state"] != "accepted" or not result["canonical_equal"]:
            raise SystemExit(1)


if __name__ == "__main__":
    main()
