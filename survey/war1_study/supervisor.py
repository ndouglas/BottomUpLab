"""One-child Unix supervisor, individual wait4 peak, bounded control/log output.

Scientific permission is validated from the retained receipt packet before the
child starts; the child independently validates it before its channel handshake.
"""
import hashlib
import math
import os
from pathlib import Path
import signal
import stat
import socket
import subprocess
import sys
import tempfile
import time

from .archive import read_campaign
from .archive_provenance import LIMITS, validate_monitor, validate_retained_receipts
from .io import METADATA_LIMIT, _checked_path, canonical_bytes, create_bytes, read_regular, strict_json
from .manifest import validate_manifest
from .registration import artifact, load, native_metadata, save, source_inventory

MESSAGE_LIMIT = 16384
LOG_LIMIT = 65536
CAMPAIGN_LIMIT = 4294967296
PARENT_RESERVE = 524288  # logs128KiB + outer failure256KiB + monitor/success64KiB each


def publication_budget(root):
    total = 0
    entries = 0
    pending = [root]
    while pending:
        directory = pending.pop()
        with os.scandir(directory) as children:
            for child in children:
                entries += 1
                if entries > 8192:
                    raise ValueError("parent publication entry bound")
                info = child.stat(follow_symlinks=False)
                if stat.S_ISDIR(info.st_mode):
                    pending.append(Path(child.path))
                elif stat.S_ISREG(info.st_mode):
                    total += info.st_size
                    if total + PARENT_RESERVE > CAMPAIGN_LIMIT:
                        raise ValueError("campaign exceeds parent publication reserve")
                else:
                    raise ValueError("nonregular parent publication input")
    return total



def peak_bytes(value, platform):
    if platform not in ("darwin", "linux"):
        raise ValueError("unsupported monitor platform")
    if type(value) not in (int, float) or not math.isfinite(value) or value <= 0:
        raise ValueError("missing/invalid individual wait4 peak")
    return math.ceil(value * (1024 if platform == "linux" else 1))


def check_limits(elapsed, peak, wall_seconds, rss_bytes):
    if type(elapsed) not in (int, float) or not math.isfinite(elapsed) or elapsed < 0:
        raise ValueError("invalid monotonic wall sample")
    if type(peak) is not int or peak <= 0:
        raise ValueError("missing/invalid RSS sampler")
    if elapsed > wall_seconds:
        raise ValueError("collection wall limit exceeded")
    if peak > rss_bytes:
        raise ValueError("collection RSS limit exceeded")


def sample_rss(pid):
    if sys.platform == "linux":
        with Path(f"/proc/{pid}/status").open("rb") as status:
            raw = status.read(65537)
        if len(raw) > 65536:
            raise ValueError("process status sample exceeds bound")
        values = {}
        for line in raw.splitlines():
            words = line.split()
            if words and words[0] in (b"VmRSS:", b"VmHWM:"):
                if len(words) != 3 or words[2] != b"kB":
                    raise ValueError("invalid Linux RSS units")
                values[words[0]] = int(words[1]) * 1024
        if set(values) != {b"VmRSS:", b"VmHWM:"}:
            raise ValueError("missing Linux RSS sample")
        return max(values.values())
    if sys.platform == "darwin":
        result = subprocess.run(("/bin/ps", "-o", "rss=", "-p", str(pid)), capture_output=True, timeout=0.075, check=True)
        raw = result.stdout
        if len(raw) > 128 or not raw.strip().isdigit():
            raise ValueError("missing/invalid macOS RSS sample")
        return int(raw) * 1024
    raise ValueError("unsupported monitor platform")


def _copy_descriptor(descriptor, target):
    raw = read_regular(Path(descriptor["path"]), METADATA_LIMIT)
    if len(raw) != descriptor["bytes"] or hashlib.sha256(raw).hexdigest() != descriptor["sha256"]:
        raise ValueError("provenance changed or invalid raw artifact binding")
    create_bytes(target, raw, METADATA_LIMIT)
    return strict_json(raw, METADATA_LIMIT)


def _inputs(command, out, staging):
    if type(command) is not tuple or len(command) not in (6, 10):
        raise ValueError("supervisor requires exact fixture/collect argv")
    if any(not isinstance(a, str) or not a or len(a.encode()) > 1024 for a in command):
        raise ValueError("invalid command argv")
    binary = _checked_path(Path(command[0]))
    metadata = native_metadata(binary)
    if source_inventory(Path(metadata["root"]), metadata["selected_path"]) != metadata["inventory"]:
        raise ValueError("parent/native source inventory mismatch")
    if command[1] == "fixture" and len(command) == 6 and command[2] == "--manifest" and command[4] == "--out":
        manifest_path = Path(command[3])
        if metadata["selected_purpose"] != "construction_fixture":
            raise ValueError("fixture binary required")
        binding = metadata["binding"]
    elif command[1] == "collect" and len(command) == 10 and command[2::2] == ("--declaration", "--acceptance", "--execution", "--out"):
        # Actual receipts only. Neither this module nor the CLI creates review/owner decisions.
        provenance = staging / "provenance"
        provenance.mkdir()
        (provenance / "fixture-probes").mkdir()
        receipts = {}
        hashes = {}
        for name, path in zip(("declaration", "acceptance", "execution"), command[3:8:2]):
            raw = read_regular(Path(path), METADATA_LIMIT)
            receipts[name] = strict_json(raw, METADATA_LIMIT)
            hashes[name] = hashlib.sha256(raw).hexdigest()
            create_bytes(provenance / f"{name}.json", raw, METADATA_LIMIT)
        declaration = receipts["declaration"]
        manifest_path = Path(declaration["manifest"]["path"])
        for name in ("build", "numeric", "resources", "source_inventory"):
            value = _copy_descriptor(declaration[name], provenance / (name.replace("_", "-") + ".json"))
            receipts[name] = value
        n, r = receipts["numeric"], receipts["resources"]
        _copy_descriptor(n["numeric_export"], provenance / "numeric-export.json")
        _copy_descriptor(n["oracle_packet"], provenance / "oracle-packet.json")
        _copy_descriptor(r["forecast_inputs"], provenance / "resource-forecast-inputs.json")
        if not isinstance(r["fixture_probes"], list) or not 1 <= len(r["fixture_probes"]) <= 32:
            raise ValueError("fixture probe count")
        for i, p in enumerate(r["fixture_probes"]):
            _copy_descriptor(p["artifact"], provenance / f"fixture-probes/{i:06}.json")
        binding = dict(metadata["binding"], declaration_sha256=hashes["declaration"], acceptance_sha256=hashes["acceptance"], execution_approval_sha256=hashes["execution"])
        save(staging / "source-inventory.json", metadata["inventory"])
    else:
        raise ValueError("unsupported supervisor command or argument order")
    if _checked_path(Path(command[-1])) != out or out.exists():
        raise ValueError("supervisor output must be exact new campaign directory")
    raw = read_regular(manifest_path, METADATA_LIMIT)
    manifest = validate_manifest(raw, Path(metadata["root"]))
    if manifest.sha256 != binding["manifest_sha256"] or manifest.purpose != metadata["selected_purpose"]:
        raise ValueError("compiled selected manifest mismatch")
    header = dict(schema="war1-study-archive-v1", mode=manifest.purpose, binding=binding, planned=len(manifest.cases))
    if command[1] == "collect":
        create_bytes(staging / "manifest.json", raw, METADATA_LIMIT)
        validate_retained_receipts(staging, header, manifest)
        from .preparation import validate_resource_evidence
        validate_resource_evidence(receipts["resources"], load(staging / "provenance/resource-forecast-inputs.json"),
                                   load(staging / "provenance/fixture-probes/000000.json"), metadata)
    return metadata, manifest, header


def supervise(command: tuple[str, ...], out: Path, wall_seconds: int, rss_bytes: int):
    if type(wall_seconds) is not int or not 1 <= wall_seconds <= 43200 or type(rss_bytes) is not int or not 1 <= rss_bytes <= 2147483648:
        raise ValueError("monitor limits outside registered caps")
    if sys.platform not in ("darwin", "linux") or not hasattr(os, "wait4"):
        raise ValueError("individual wait4 monitoring unavailable")
    out = _checked_path(out)
    with tempfile.TemporaryDirectory(prefix="w1-", dir=Path("/tmp").resolve()) as temporary:
        temporary = Path(temporary)
        metadata, manifest, header = _inputs(command, out, temporary)
        server = socket.socket(socket.AF_UNIX)
        socket_path = temporary / "s"
        if len(str(socket_path).encode()) > 99:
            raise ValueError("Unix socket path exceeds 99 bytes")
        server.bind(str(socket_path))
        server.listen(1)
        server.setblocking(False)
        child = None
        channel = None
        errors = []
        logs = {"stdout": bytearray(), "stderr": bytearray()}
        logfiles = {name: (temporary / name).open("xb") for name in logs}
        start = time.monotonic()
        peak = 0
        final_peak = None
        last_progress = None
        buffer = bytearray()
        handshake = False
        stopped_at = None
        reaped = False
        try:
            child = subprocess.Popen((*command, "--control-socket", str(socket_path)), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            for pipe in (child.stdout, child.stderr):
                os.set_blocking(pipe.fileno(), False)
            while not reaped:
                tick = time.monotonic()
                waited, status, usage = os.wait4(child.pid, os.WNOHANG)
                if waited:
                    child.returncode = os.waitstatus_to_exitcode(status)
                    final_peak = peak_bytes(usage.ru_maxrss, sys.platform)
                    peak = max(peak, final_peak)
                    reaped = True
                if channel is None:
                    try:
                        channel, _ = server.accept()
                        channel.setblocking(False)
                    except BlockingIOError:
                        pass
                if channel is not None:
                    drained = 0
                    while drained < MESSAGE_LIMIT:
                        try:
                            data = channel.recv(min(4096, MESSAGE_LIMIT - drained))
                        except BlockingIOError:
                            break
                        if not data:
                            break
                        drained += len(data)
                        buffer.extend(data)
                        if len(buffer) > MESSAGE_LIMIT:
                            raise ValueError("monitor message bound")
                        while b"\n" in buffer:
                            end = buffer.index(b"\n")
                            message = strict_json(bytes(buffer[:end]), MESSAGE_LIMIT)
                            del buffer[:end + 1]
                            if not handshake:
                                expected = dict(schema="war1-study-monitor-v1", binding=header["binding"], child_pid=child.pid, limits=LIMITS)
                                if canonical_bytes(message) != canonical_bytes(expected):
                                    raise ValueError("monitor handshake identity/limits mismatch")
                                channel.sendall(canonical_bytes(dict(schema="war1-study-monitor-v1", action="ready")))
                                handshake = True
                            else:
                                from .manifest import _fields, _uint
                                _fields(message, {"schema", "key", "semantic_steps", "observed_steps", "durable_audit_steps"}, "progress")
                                if message["schema"] != "war1-study-progress-v1":
                                    raise ValueError("progress schema")
                                for k in ("semantic_steps", "observed_steps", "durable_audit_steps"):
                                    _uint(message[k], k, 512)
                                _fields(message["key"], {"ordinal", "arm_id", "replicate_index"}, "progress key")
                                _uint(message["key"]["ordinal"], "progress ordinal", len(manifest.cases)-1)
                                expected_key = manifest.cases[message["key"]["ordinal"]].key
                                if message["key"] != dict(ordinal=expected_key.ordinal, arm_id=expected_key.arm_id, replicate_index=expected_key.replicate_index):
                                    raise ValueError("progress key outside declared population")
                                last_progress = dict(sample=message, elapsed_seconds=time.monotonic() - start)
                    if handshake and not reaped and stopped_at is None:
                        channel.sendall(canonical_bytes(dict(schema="war1-study-monitor-v1", action="heartbeat")))
                for name in logs:
                    pipe = getattr(child, name)
                    try:
                        data = os.read(pipe.fileno(), LOG_LIMIT + 1)
                    except BlockingIOError:
                        data = b""
                    remaining = LOG_LIMIT - len(logs[name])
                    logs[name].extend(data[:remaining])
                    logfiles[name].write(data[:remaining])
                    logfiles[name].flush()
                    if len(data) > remaining:
                        raise ValueError(f"{name} output bound exceeded")
                if not reaped:
                    try:
                        peak = max(peak, sample_rss(child.pid))
                    except (OSError, ValueError, subprocess.SubprocessError):
                        # A process may exit between wait4 and the active sampler.
                        waited, status, usage = os.wait4(child.pid, os.WNOHANG)
                        if not waited:
                            raise ValueError("active process RSS sampler unavailable") from None
                        child.returncode = os.waitstatus_to_exitcode(status)
                        final_peak = peak_bytes(usage.ru_maxrss, sys.platform)
                        peak = max(peak, final_peak)
                        reaped = True
                try:
                    check_limits(time.monotonic() - start, peak, wall_seconds, rss_bytes)
                except ValueError as error:
                    if stopped_at is None:
                        errors.append(str(error))
                        stopped_at = time.monotonic()
                        if channel is not None:
                            try:
                                channel.sendall(canonical_bytes(dict(schema="war1-study-monitor-v1", action="stop")))
                            except OSError as send_error:
                                errors.append(f"stop channel: {send_error}")
                        if not reaped:
                            os.kill(child.pid, signal.SIGTERM)
                if stopped_at is not None and not reaped and time.monotonic() - stopped_at >= 5:
                    os.kill(child.pid, signal.SIGKILL)
                if not reaped:
                    time.sleep(max(0, 0.075 - (time.monotonic() - tick)))
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            errors.append(str(error))
        finally:
            if child is not None and not reaped:
                try:
                    os.kill(child.pid, signal.SIGTERM)
                except OSError as error:
                    errors.append(f"SIGTERM: {error}")
                deadline = time.monotonic() + 5
                while not reaped:
                    waited, status, usage = os.wait4(child.pid, os.WNOHANG)
                    if waited:
                        child.returncode = os.waitstatus_to_exitcode(status)
                        try:
                            final_peak = peak_bytes(usage.ru_maxrss, sys.platform)
                            peak = max(peak, final_peak)
                        except ValueError as error:
                            errors.append(str(error))
                        reaped = True
                    else:
                        if time.monotonic() >= deadline:
                            try:
                                os.kill(child.pid, signal.SIGKILL)
                            except OSError as error:
                                errors.append(f"SIGKILL: {error}")
                        time.sleep(0.025)
            if child is not None:
                for name in logs:
                    getattr(child, name).close()
            if channel is not None:
                channel.close()
            server.close()
            for logfile in logfiles.values():
                logfile.flush()
                os.fsync(logfile.fileno())
                logfile.close()
        elapsed = time.monotonic() - start
        if child is None or child.returncode != 0:
            errors.append(f"child exit: {None if child is None else child.returncode}")
        if not handshake:
            errors.append("no verified monitor handshake")
        try:
            if final_peak is None:
                raise ValueError("individual final wait4 peak unavailable")
            check_limits(elapsed, peak, wall_seconds, rss_bytes)
        except ValueError as error:
            errors.append(str(error))
        archive = None
        try:
            if source_inventory(Path(metadata["root"]), metadata["selected_path"]) != metadata["inventory"] or artifact(Path(command[0]), 1024**3)["sha256"] != header["binding"]["executable_sha256"]:
                raise ValueError("source/binary changed after child")
        except (ValueError, OSError, KeyError) as error:
            errors.append(f"source: {error}")
        try:
            archive = read_campaign(out, manifest)
            if archive.header["binding"] != header["binding"] or archive.header["mode"] != header["mode"]:
                raise ValueError("saved candidate differs from monitored source/binary binding")
            if not archive.complete:
                raise ValueError("candidate/archive incomplete")
            if manifest.purpose != "construction_fixture":
                validate_retained_receipts(out, header, manifest)
        except (ValueError, OSError, KeyError) as error:
            errors.append(f"archive/source: {error}")
        elapsed = time.monotonic() - start
        try:
            check_limits(elapsed, peak, wall_seconds, rss_bytes)
        except ValueError as error:
            errors.append(str(error))
        # Logs are bounded and counted in the reserved campaign metadata allowance.
        destination = out if out.is_dir() else out.with_name(out.name + ".supervisor-failure")
        if destination == out:
            try:
                publication_budget(out)
            except (ValueError, OSError) as error:
                errors.append(f"publication: {error}")
                destination = out.with_name(out.name + ".supervisor-failure")
        if not destination.exists():
            destination.mkdir(mode=0o700)
        for name, raw in logs.items():
            create_bytes(destination / f"supervisor-{name}.txt", bytes(raw), LOG_LIMIT)
        if errors:
            native_failure = None
            if (out / "failure.json").is_file():
                try:
                    if archive is None or any(e.startswith("failure:") for e in archive.header["validation"]["errors"]):
                        raise ValueError("native failure receipt/prefix could not be independently validated")
                    native_failure = load(out / "failure.json")
                except ValueError as error:
                    errors.append(f"native receipt: {error}")
            case = native_failure.get("case_failure") if native_failure else None
            result = dict(schema="war1-study-supervisor-failure-v1", errors=errors,
                child_pid=None if child is None else child.pid, exit_code=None if child is None else child.returncode,
                platform=sys.platform, peak_method="os.wait4.ru_maxrss", peak_rss_bytes=peak, wall_seconds=elapsed,
                semantic_steps=None if case is None else case["semantic_steps"], observed_steps=None if case is None else case["observed_steps"],
                final_checkpoint=None if case is None else case["final_checkpoint"], reason="unknown_after_termination" if case is None else "native_final_receipt",
                last_progress=last_progress, native_failure=native_failure,
                census=None if archive is None else archive.census,
                partial_keys=[] if archive is None else archive.header["validation"]["partial_keys"],
                acknowledged_archive_prefix=None if archive is None else archive.header["validation"]["acknowledged_archive_prefix"])
            create_bytes(destination / "supervisor-failure.json", canonical_bytes(result), 262144)
            return result
        candidate = load(out / "candidate-index.json")
        result = dict(schema="war1-study-monitor-result-v1", mode=header["mode"], binding=header["binding"], child_pid=child.pid,
            exit_code=0, platform=sys.platform, peak_method="os.wait4.ru_maxrss", wall_seconds=elapsed, peak_rss_bytes=peak,
            wall_limit_seconds=43200, rss_limit_bytes=2147483648, campaign_limit_bytes=4294967296, source_valid=True, archive_valid=True)
        validate_monitor(result, header)
        create_bytes(out / "monitor.json", canonical_bytes(result), 65536)
        descriptor = lambda name: dict(artifact(out / name), path=name)
        success = dict(schema="war1-study-success-v1", mode=header["mode"], binding=header["binding"],
            candidate=descriptor("candidate-index.json"), receipt=candidate["receipt"], census=candidate["census"], monitor=descriptor("monitor.json"))
        try:
            create_bytes(out / "success.json", canonical_bytes(success), 65536)
        except (ValueError, OSError):
            # A failed parent-directory sync cannot leave a misleading marker.
            (out / "success.json").unlink(missing_ok=True)
            fd = os.open(out, os.O_RDONLY | os.O_DIRECTORY)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
            raise
        return result
