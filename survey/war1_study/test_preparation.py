"""Synthetic monitor and schema proof boundaries; no scientific engines."""
import unittest
import sys
from contextlib import ExitStack
from types import SimpleNamespace
from pathlib import Path
import tempfile
from unittest.mock import MagicMock, patch
from survey.war1_study import supervisor, resource_proof


def synthetic_resources(root, metadata=None, fixture=None):
    """Synthetic positive resource packet; metadata is injected only by tests."""
    from copy import deepcopy
    from survey.war1_study import registration, preparation
    from survey.war1_study.archive_provenance import LIMITS, POPULATION
    root.mkdir(parents=True, exist_ok=True)
    binary = root / "construction_test_binary"
    binary.write_bytes(b"construction_test fake executable, never launched")
    bound = registration.artifact(binary)
    if fixture is None:
        fixture = dict(root=str(root), selected_path="construction_test_fixture.json",
            selected_purpose="construction_fixture", flags=[["PROFILE", "release"]], reference_calls_checked=264,
            inventory=dict(entries=[]), binding=dict(manifest_sha256="b"*64, compiled_inputs_sha256="c"*64,
                executable_sha256=bound["sha256"], declaration_sha256=None, acceptance_sha256=None, execution_approval_sha256=None))
    metadata = metadata or fixture
    proof = resource_proof.schema_proof()
    rows = [dict(actor_steps=1, kernel_seconds_per_actor_step=1e-8, projection_seconds_per_step=1e-8,
        start_outcome_seconds=1e-8, case_overhead_seconds=1e-8, audit_seconds_per_frame=1e-8) for _ in range(16)]
    def saved(name, value):
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        registration.save(path, value)
        return registration.artifact(path)
    probe = dict(schema="war1-study-fixture-probe-v1", purpose="construction_fixture", cases=rows)
    probe_ref = saved("fixture-archive/fixture-probe.json", probe)
    monitor = dict(schema="war1-study-monitor-result-v1", mode="construction_fixture", binding=fixture["binding"],
        child_pid=1234, exit_code=0, platform=sys.platform, peak_method="os.wait4.ru_maxrss", wall_seconds=0.1,
        peak_rss_bytes=1000000, wall_limit_seconds=43200, rss_limit_bytes=2147483648, campaign_limit_bytes=4294967296,
        source_valid=True, archive_valid=True)
    monitor_ref = saved("fixture-archive/monitor.json", monitor)
    command = [str(binary), "fixture", "--manifest", str(Path(fixture["root"])/fixture["selected_path"]), "--out", str(root/"fixture-archive")]
    synthetic = dict(schema="war1-study-synthetic-probe-v1", purpose="construction_fixture", manifest_cases=52224,
        schema_proof=proof, source_seconds=1, archive_seconds_per_byte=1e-9, peak_rss_bytes=1000000)
    synthetic_ref = saved("synthetic-monitored.json", synthetic)
    adjusted = dict(synthetic, source_seconds=1.1)
    predicted = resource_proof.forecast(rows, adjusted)
    inputs = dict(schema="war1-study-resource-forecast-inputs-v1", purpose="construction_fixture", schema_proof=proof,
        fixture_metadata=fixture, scientific_metadata=metadata, measured_cases=rows, synthetic=adjusted, forecast=predicted,
        fixture_binary=bound, fixture_monitor=monitor_ref, fixture_probe=probe_ref, synthetic_probe=synthetic_ref)
    inputs_ref = saved("resource-forecast-inputs.json", inputs)
    resources = dict(schema="war1-study-resources-v1", manifest_sha256=metadata["binding"]["manifest_sha256"],
        compiled_inputs_sha256=metadata["binding"]["compiled_inputs_sha256"], binary_sha256=metadata["binding"]["executable_sha256"],
        budgets=LIMITS, census=POPULATION, reserved_bytes=proof["reserved_bytes"], failure_reserve_bytes=proof["failure_reserve_bytes"],
        schema_bounds_valid=True, aggregate_proof_valid=True, forecast_valid=True, forecast_method="construction_test synthetic costs",
        forecast_inputs=inputs_ref, forecast_wall_seconds=predicted["wall_seconds"], forecast_peak_rss_bytes=predicted["peak_rss_bytes"],
        fixture_probes=[dict(artifact=probe_ref, command=command, compiled_inputs_sha256=fixture["binding"]["compiled_inputs_sha256"],
            binary_sha256=bound["sha256"], wall_seconds=0.1, peak_rss_bytes=1000000, fixture_purpose="construction_fixture")], unavailable=[])
    return fixture, resources, inputs, probe


class PreseedTests(unittest.TestCase):
    def test_fully_linked_synthetic_preseed_packet_is_validated(self):
        from survey.war1_study import preparation, registration
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            root = Path(temp).resolve()
            metadata, resources, inputs, probe = synthetic_resources(root)
            registration.save(root/"resources.json", resources)
            gate = root/preparation.PRESEED
            gate.parent.mkdir(parents=True)
            registration.save(gate, dict(schema="war1-study-preseed-resources-v1", purpose="construction_fixture", registered_histories=0,
                resources=registration.artifact(root/"resources.json"), forecast_inputs=resources["forecast_inputs"]))
            with patch.object(preparation, "native_metadata", return_value=metadata), patch.object(preparation, "source_inventory", return_value=metadata["inventory"]):
                self.assertEqual(supervisor.canonical_bytes(preparation.validate_preseed(root)), supervisor.canonical_bytes(resources))

    def test_every_invalid_resource_link_is_rejected_before_entropy(self):
        from copy import deepcopy
        from survey.war1_study import preparation, registration
        cases = ("forecast_descriptor", "empty_probes", "empty_method", "forecast_link", "probe_link", "probe_source", "probe_binary", "probe_wall", "probe_rss", "probe_command", "unknown_probe", "typed_reserve")
        for mutation in cases:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
                root = Path(temp).resolve()
                from survey.war1_study import manifest
                project = Path(__file__).resolve().parents[2]
                for relative in (manifest.PROTOCOL, manifest.PROPOSAL):
                    (root/relative).parent.mkdir(parents=True, exist_ok=True)
                    (root/relative).write_bytes((project/relative).read_bytes())
                metadata, resource, inputs, probe = synthetic_resources(root / "evidence")
                metadata["root"] = str(root)
                # Rewrite dependent raw metadata/command identities before mutating.
                inputs["fixture_metadata"]["root"] = str(root)
                resource["fixture_probes"][0]["command"][3] = str(root/metadata["selected_path"])
                (root/"evidence/resource-forecast-inputs.json").unlink()
                registration.save(root/"evidence/resource-forecast-inputs.json", inputs)
                resource["forecast_inputs"] = registration.artifact(root/"evidence/resource-forecast-inputs.json")
                good_input_ref = deepcopy(resource["forecast_inputs"])
                p = resource["fixture_probes"][0]
                if mutation == "forecast_descriptor": resource["forecast_inputs"] = {}
                elif mutation == "empty_probes": resource["fixture_probes"] = []
                elif mutation == "empty_method": resource["forecast_method"] = ""
                elif mutation == "forecast_link": resource["forecast_inputs"] = dict(good_input_ref, path=str(root/"different.json"))
                elif mutation == "probe_link": p["artifact"] = dict(p["artifact"], path=str(root/"different.json"))
                elif mutation == "probe_source": p["compiled_inputs_sha256"] = "0"*64
                elif mutation == "probe_binary": p["binary_sha256"] = "0"*64
                elif mutation == "probe_wall": p["wall_seconds"] = 2
                elif mutation == "probe_rss": p["peak_rss_bytes"] = 2000000
                elif mutation == "probe_command": p["command"] = ["construction_test unrelated command"]
                elif mutation == "unknown_probe": p["extra"] = True
                elif mutation == "typed_reserve": resource["reserved_bytes"] = float(resource["reserved_bytes"])
                resource_path = root/"evidence/resources.json"
                registration.save(resource_path, resource)
                gate_path = root/preparation.PRESEED
                gate_path.parent.mkdir(parents=True)
                registration.save(gate_path, dict(schema="war1-study-preseed-resources-v1", purpose="construction_fixture", registered_histories=0,
                    resources=registration.artifact(resource_path), forecast_inputs=good_input_ref))
                with patch.object(preparation, "native_metadata", return_value=metadata), patch.object(preparation, "source_inventory", return_value=metadata["inventory"]), patch.object(registration.os, "urandom", side_effect=AssertionError("scientific entropy forbidden")) as entropy:
                    with self.assertRaises(ValueError):
                        registration.prepare(root, root/"must-not-exist")
                    entropy.assert_not_called()
                    self.assertFalse((root/"must-not-exist").exists())


class MonitorTests(unittest.TestCase):
    def synthetic_child(self, root, *, race=False, invalid_peak=False, fail_write=None, fail_sync=False, complete=False, fail_revoke_sync=False, fail_flush=False, fail_cleanup=False, fail_destination=False):
        """Real trivial child; synthetic archive/resource boundary, no Engine."""
        out = root / "archive"
        out.mkdir()
        binding = dict(executable_sha256="a"*64)
        header = dict(mode="construction_fixture", binding=binding)
        metadata = dict(root=str(root), selected_path="synthetic", inventory={})
        manifest = SimpleNamespace(purpose="construction_fixture")
        partial = [dict(ordinal=0, arm_id="construction_test", replicate_index=0)]
        prefix = dict(sha256="b"*64, bytes=17, acknowledged_cases=0)
        archive = SimpleNamespace(complete=complete, census=dict(partial=1), header=dict(header,
            validation=dict(errors=[], partial_keys=partial, acknowledged_archive_prefix=prefix)))
        candidate = out / "candidate-index.json"
        candidate.write_bytes(b'{"receipt":{},"census":{}}\n')
        code = "pass"
        real_wait = supervisor.os.wait4
        calls = []
        def waited(pid, flags):
            calls.append(pid)
            if race and len(calls) == 1:
                return 0, 0, None
            if len(calls) > (2 if race else 1):
                raise AssertionError("waited again after reaping")
            got, status, usage = real_wait(pid, 0)
            return got, status, SimpleNamespace(ru_maxrss=0) if invalid_peak else usage
        real_create = supervisor.create_bytes
        published_failure = False
        real_sync = supervisor.os.fsync
        def sync(fd):
            if fail_sync or (fail_revoke_sync and published_failure):
                raise OSError("construction_test sync failure")
            return real_sync(fd)
        def created(path, raw, limit):
            nonlocal published_failure
            if fail_write and path.name == fail_write:
                if complete:
                    real_create(path, raw, limit)
                published_failure = True
                raise OSError("construction_test publication failure")
            return real_create(path, raw, limit)
        cleanup_paths = []
        with ExitStack() as stack:
            if complete:
                # Publication tests inject a synthetic completed handshake. The
                # real child is still reaped by wait4. Native integration tests
                # separately exercise the actual Unix channel; a test child
                # closing its channel before process exit can race heartbeats.
                channel = MagicMock()
                delivered = False
                def received(_limit):
                    nonlocal delivered
                    if delivered:
                        return b""
                    delivered = True
                    return supervisor.canonical_bytes(dict(schema="war1-study-monitor-v1",
                        binding=binding, child_pid=calls[-1], limits=supervisor.LIMITS))
                channel.recv.side_effect = received
                server = MagicMock()
                server.accept.return_value = (channel, None)
                stack.enter_context(patch.object(supervisor.socket, "socket", return_value=server))
            for obj, name, kwargs in (
                (supervisor, "_inputs", dict(return_value=(metadata, manifest, header))),
                (supervisor, "source_inventory", dict(return_value={})),
                (supervisor, "artifact", dict(return_value=dict(sha256="a"*64))),
                (supervisor, "read_campaign", dict(return_value=archive)),
                (supervisor.os, "wait4", dict(side_effect=waited)),
                (supervisor.os, "kill", dict(side_effect=AssertionError("signal after reap"))),
                (supervisor, "sample_rss", dict(return_value=1000000) if complete else dict(side_effect=ValueError("construction_test sampler race"))),
                (supervisor, "create_bytes", dict(side_effect=created)),
            ):
                stack.enter_context(patch.object(obj, name, **kwargs))
            stack.enter_context(patch.object(supervisor.os, "fsync", side_effect=sync))
            if fail_flush:
                real_open = Path.open
                def opened(path, *args, **kwargs):
                    file = real_open(path, *args, **kwargs)
                    if path.name == "stdout" and args == ("xb",):
                        return SimpleNamespace(write=file.write, fileno=file.fileno, close=file.close,
                            flush=lambda: (_ for _ in ()).throw(OSError("construction_test flush failure")))
                    return file
                stack.enter_context(patch.object(Path, "open", opened))
            if fail_cleanup:
                def cleanup(path, *args, **kwargs):
                    cleanup_paths.append(path)
                    raise OSError("construction_test cleanup failure")
                stack.enter_context(patch.object(supervisor.shutil, "rmtree", side_effect=cleanup))
            if fail_destination:
                candidate.unlink()
                out.rmdir()
                real_mkdir = Path.mkdir
                def mkdir(path, *args, **kwargs):
                    if path.name.endswith(".supervisor-failure"):
                        raise OSError("construction_test destination failure")
                    return real_mkdir(path, *args, **kwargs)
                stack.enter_context(patch.object(Path, "mkdir", mkdir))
            result = supervisor.supervise((sys.executable, "-c", code), out, 43200, 2147483648)
        for path in set(cleanup_paths):
            supervisor.shutil.rmtree(path)
        self.assertEqual(result["schema"], "war1-study-supervisor-failure-v1")
        self.assertEqual(result["partial_keys"], partial)
        self.assertEqual(result["acknowledged_archive_prefix"], prefix)
        self.assertEqual(result["census"], dict(partial=1))
        self.assertFalse((out / "success.json").exists())
        return result

    def test_invalid_final_peak_never_signals_or_rewaits_reaped_child(self):
        for race in (False, True):
            with self.subTest(sampler_race=race), tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
                result = self.synthetic_child(Path(temp).resolve(), race=race, invalid_peak=True)
                self.assertTrue(any("invalid individual wait4 peak" in e for e in result["errors"]))

    def test_publication_and_log_sync_failures_preserve_original_partial_evidence(self):
        for fail_write, fail_sync in (("supervisor-failure.json", False), ("supervisor-stdout.txt", False), (None, True)):
            with self.subTest(write=fail_write, sync=fail_sync), tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
                result = self.synthetic_child(Path(temp).resolve(), fail_write=fail_write, fail_sync=fail_sync)
                self.assertIn("no verified monitor handshake", result["errors"])
                self.assertTrue(any("construction_test" in e for e in result["errors"]))

    def test_final_publication_failures_revoke_success_and_return_evidence(self):
        for name, sync in (("monitor.json", False), ("success.json", False), ("success.json", True)):
            with self.subTest(file=name, revocation_sync=sync), tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
                result = self.synthetic_child(Path(temp).resolve(), complete=True, fail_write=name, fail_revoke_sync=sync)
                self.assertTrue(any("final publication" in error for error in result["errors"]))
                if sync:
                    self.assertTrue(any("revocation sync" in error for error in result["errors"]))

    def test_log_flush_failure_returns_bounded_original_evidence(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            result = self.synthetic_child(Path(temp).resolve(), fail_flush=True)
            self.assertTrue(any("flush failure" in error for error in result["errors"]))
            self.assertLessEqual(len(result["errors"]), 32)

    def test_cleanup_and_destination_failures_preserve_returned_partial_evidence(self):
        for cleanup in (False, True):
            with self.subTest(cleanup=cleanup), tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
                result = self.synthetic_child(Path(temp).resolve(), fail_cleanup=cleanup, fail_destination=not cleanup)
                self.assertTrue(any("cleanup failure" in error or "destination failure" in error for error in result["errors"]))

    def test_wait4_units_are_individual_and_platform_specific(self):
        self.assertEqual(supervisor.peak_bytes(4096, "darwin"), 4096)
        self.assertEqual(supervisor.peak_bytes(4096, "linux"), 4194304)
        with self.assertRaises(ValueError):
            supervisor.peak_bytes(1, "unsupported")
        with self.assertRaises(ValueError):
            supervisor.peak_bytes(0, "darwin")

    def test_unsampled_final_peak_invalidates_candidate(self):
        with self.assertRaisesRegex(ValueError, "RSS"):
            supervisor.check_limits(1.0, 2147483649, 43200, 2147483648)

    def test_unknown_or_negative_samples_fail_closed(self):
        for sample in (None, -1, True):
            with self.assertRaises(ValueError):
                supervisor.check_limits(1, sample, 43200, 2147483648)

    def test_parent_publication_requires_reserved_space_including_logs(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            root = Path(temp).resolve()
            (root / "tail").write_bytes(b"x" * 100)
            with patch.object(supervisor, "CAMPAIGN_LIMIT", 100 + supervisor.PARENT_RESERVE - 1):
                with self.assertRaisesRegex(ValueError, "publication reserve"):
                    supervisor.publication_budget(root)

    def test_proof_accounts_full_population_and_all_metadata(self):
        proof = resource_proof.schema_proof()
        self.assertEqual(proof["starts_outcomes_bytes"], 1764753408)
        self.assertEqual(proof["audit_bytes"], 778240000)
        self.assertEqual(proof["reserved_bytes"], 3616735232)
        self.assertLessEqual(proof["audit_frame_bytes"], 28672)
        self.assertLessEqual(proof["outcome_bytes"], 32768)
        self.assertLessEqual(proof["start_bytes"], 1024)
        self.assertLessEqual(proof["metadata_reserved_bytes"], 1073741824)

    def test_forecast_requires_measured_per_case_constructor_and_terminal_cost(self):
        rows = [dict(actor_steps=1, kernel_seconds_per_actor_step=1e-8,
                     projection_seconds_per_step=1e-8, start_outcome_seconds=1e-8,
                     audit_seconds_per_frame=1e-8) for _ in range(16)]
        synthetic = dict(source_seconds=1, archive_seconds_per_byte=1e-9, peak_rss_bytes=1000000)
        with self.assertRaisesRegex(ValueError, "missing measured construction cost"):
            resource_proof.forecast(rows, synthetic)

    def test_unavailable_or_excess_forecast_blocks_readiness(self):
        for rows in ([], [{"actor_steps": 0}], [{"kernel_seconds_per_actor_step": 1.0}]):
            with self.assertRaises(ValueError):
                resource_proof.forecast(rows, {})


if __name__ == "__main__":
    unittest.main()
