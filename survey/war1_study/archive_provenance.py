"""Saved source/authorization/monitor validation; never reads external paths."""

import hashlib
from pathlib import PurePosixPath
import stat

from .io import METADATA_LIMIT, read_regular, strict_json
from .manifest import _fields, _text, _uint


def _require(condition, message):
    if not condition:
        raise ValueError(message)


def _hash(value):
    _require(
        isinstance(value, str)
        and len(value) == 64
        and all(c in "0123456789abcdef" for c in value),
        "invalid SHA256",
    )


def _json(root, path):
    raw = read_regular(root / path, METADATA_LIMIT)
    return strict_json(raw, METADATA_LIMIT), raw


def validate_artifact_descriptor(descriptor):
    _fields(descriptor, {"path", "sha256", "bytes"}, "artifact")
    _text(descriptor["path"], "original artifact path", 4096)
    _hash(descriptor["sha256"])
    _uint(descriptor["bytes"], "artifact bytes", METADATA_LIMIT)


def _artifact(root, path, descriptor):
    validate_artifact_descriptor(descriptor)
    raw = read_regular(root / path, METADATA_LIMIT)
    _require(
        hashlib.sha256(raw).hexdigest() == descriptor["sha256"]
        and len(raw) == descriptor["bytes"],
        "retained artifact digest/length",
    )
    return strict_json(raw, METADATA_LIMIT)


def validate_receipt(receipt, planned):
    """Validate the exact native archive-prefix schema before value comparison."""
    _fields(receipt, {"sha256", "bytes", "acknowledged_cases"}, "archive prefix")
    _hash(receipt["sha256"])
    _uint(receipt["bytes"], "archive prefix bytes", 4 * 1024**3)
    _uint(receipt["acknowledged_cases"], "acknowledged cases", planned)


def validate_census(census, planned):
    """Require integer census tokens and the fixed planned partition."""
    states = {"complete_valid", "failed", "invalid", "partial", "pending", "unstarted"}
    _fields(census, states | {"planned", "attempted"}, "archive census")
    for key, value in census.items():
        _uint(value, f"census {key}", planned)
    _require(
        census["planned"] == planned and sum(census[k] for k in states) == planned,
        "archive census partition",
    )


def validate_inventory(packet):
    """Validate both inventory serializations before strict semantic matching."""
    _fields(packet, {"schema", "compiled_inputs_sha256", "entries"}, "source inventory")
    _require(
        packet["schema"] == "war1-study-source-inventory-v1", "source inventory schema"
    )
    _hash(packet["compiled_inputs_sha256"])
    entries = packet["entries"]
    _require(
        isinstance(entries, list) and 1 <= len(entries) <= 4096, "source entry bound"
    )
    total = 0
    previous = ""
    for item in entries:
        _fields(item, {"path", "sha256", "bytes"}, "source entry")
        _text(item["path"], "source path", 4096)
        _require(item["path"] > previous, "source path/order")
        previous = item["path"]
        _hash(item["sha256"])
        _uint(item["bytes"], "source file bytes", METADATA_LIMIT)
        total += item["bytes"]
    _require(total <= 64 * 1024**2, "source aggregate bound")


SOURCE_REQUIRED = (
    "crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json",
    "crates/sugarscape-core/src/key_delegation/source_inputs.rs",
    "crates/sugarscape-core/src/key_delegation/tests/fixtures/kernel-reference.json",
    "crates/sugarscape-core/src/key_delegation/tests/fixtures/game-reference.json",
    "crates/sugarscape-core/src/key_delegation/tests/fixtures/limits-reference.json",
    "Cargo.toml",
    "Cargo.lock",
    "crates/sugarscape-core/Cargo.toml",
    "crates/sugarscape-core/build.rs",
    "crates/sugarscape-core/build_support/surface_discovery.rs",
    "crates/sugarscape-core/assets/sugar-map.txt",
    "survey/Cargo.toml",
    "survey/Cargo.lock",
    "survey/build.rs",
    "survey/src/bin/war1_study.rs",
    "survey/build_support/war1_study_source_identity.rs",
    "survey/build_support/f5_source_identity.rs",
    "survey/build_support/bt_source_identity.rs",
    "survey/build_support/war1_source_identity.rs",
    "survey/war1_study/fixtures/core-literal-inputs-v1.txt",
    "docs/superpowers/specs/2026-10-10-war-1-measurement-design.md",
    "docs/superpowers/specs/2026-10-10-war-1-measurement-proposal.json",
    "docs/superpowers/plans/2026-10-10-war-1-measurement.md",
    "docs/superpowers/specs/2026-10-09-war-1-engagement-design.md",
    "docs/superpowers/specs/2026-10-10-war-1-engineering-amendment.md",
    "docs/superpowers/plans/2026-10-09-war-1-engagement.md",
    "crates/sugarscape-core/src/lib.rs",
    "survey/src/war1_study/mod.rs",
    "survey/war1_study/contracts.py",
)


LITERAL_INVENTORY = "survey/war1_study/fixtures/core-literal-inputs-v1.txt"


def literal_paths(raw):
    text = raw.decode("utf-8")
    paths = text.splitlines()
    _require(
        text.endswith("\n") and 1 <= len(paths) <= 4096 and paths == sorted(set(paths)),
        "literal inventory ordering/bounds",
    )
    for path in paths:
        _text(path, "literal path", 4096)
        p = PurePosixPath(path)
        _require(
            str(p) == path
            and not p.is_absolute()
            and ".." not in p.parts
            and "\\" not in path,
            "literal canonical path",
        )
        _require(
            path.startswith(
                ("sweeps/", "data/anasazi/", "data/bali/", "docs/examples/burrow/")
            )
            or path == "crates/sugarscape-core/assets/sugar-map.txt",
            "literal input outside approved roots",
        )
    return paths


def validate_source(root, header, manifest):
    """Rehash saved closure in bounded file increments and enforce total budget."""
    packet, _ = _json(root, "source-inventory.json")
    validate_inventory(packet)
    _fields(packet, {"schema", "compiled_inputs_sha256", "entries"}, "source inventory")
    _require(
        packet["schema"] == "war1-study-source-inventory-v1"
        and packet["compiled_inputs_sha256"]
        == header["binding"]["compiled_inputs_sha256"],
        "source inventory binding",
    )
    entries = packet["entries"]
    _require(
        isinstance(entries, list) and 1 <= len(entries) <= 4096, "source entry bound"
    )
    literal_index = next(
        (i for i, entry in enumerate(entries) if entry["path"] == LITERAL_INVENTORY),
        None,
    )
    _require(literal_index is not None, "missing literal source inventory")
    literals = literal_paths(
        read_regular(root / f"source/{literal_index:06}.bin", METADATA_LIMIT)
    )
    paths = {i.get("path") for i in entries if isinstance(i, dict)}
    _require(
        set(SOURCE_REQUIRED) | set(literals) <= paths,
        "missing required frozen source inputs",
    )
    total = 0
    previous = ""
    digest = hashlib.sha256(b"war1-study-compiled-inputs-v1\0")
    manifest_seen = False
    for i, item in enumerate(entries):
        _fields(item, {"path", "sha256", "bytes"}, "source entry")
        path = item["path"]
        _text(path, "source path", 4096)
        parts = PurePosixPath(path)
        _require(
            not parts.is_absolute()
            and str(parts) == path
            and ".." not in parts.parts
            and "\\" not in path
            and path > previous,
            "source path/order",
        )
        selected_manifest = (
            path == "survey/war1_study/fixtures/construction-manifest.json"
            or (
                path.startswith("survey/war1_study/manifests/")
                and path.endswith(".json")
            )
        )
        _require(
            path in SOURCE_REQUIRED
            or path in literals
            or selected_manifest
            or path.startswith(
                ("crates/sugarscape-core/src/", "survey/src/war1_study/")
            )
            or (path.startswith("survey/war1_study/") and path.endswith(".py")),
            "unexpected frozen source input",
        )
        _hash(item["sha256"])
        _uint(item["bytes"], "source file bytes", METADATA_LIMIT)
        total += item["bytes"]
        _require(total <= 64 * 1024**2, "source aggregate bound")
        raw = read_regular(
            root / f"source/{i:06}.bin",
            min(METADATA_LIMIT, 64 * 1024**2 - total + item["bytes"]),
        )
        _require(
            len(raw) == item["bytes"]
            and hashlib.sha256(raw).hexdigest() == item["sha256"],
            "saved source bytes mismatch",
        )
        normative = {
            "docs/superpowers/specs/2026-10-10-war-1-measurement-design.md": manifest.raw[
                "protocol_sha256"
            ],
            "docs/superpowers/specs/2026-10-10-war-1-measurement-proposal.json": manifest.raw[
                "proposal_sha256"
            ],
        }
        if path in normative:
            _require(
                item["sha256"] == normative[path], "saved normative source identity"
            )
        for data in (path.encode(), raw):
            digest.update(len(data).to_bytes(8, "big"))
            digest.update(data)
        if selected_manifest:
            _require(
                raw == manifest.raw_bytes and not manifest_seen,
                "selected manifest source bytes",
            )
            manifest_seen = True
        previous = path
    _require(
        manifest_seen and digest.hexdigest() == packet["compiled_inputs_sha256"],
        "saved closure/manifest mismatch",
    )
    archive_metadata_namespace(root)
    return packet


def archive_metadata_namespace(root):
    # Count actual uncompressed bytes, including unknown leftovers, before use.
    metadata = []
    budget = 0
    files = 0
    directories = 0
    pending = [root]
    while pending:
        directory = pending.pop()
        directories += 1
        _require(directories <= 4096, "archive directory bound")
        for child in directory.iterdir():
            mode = child.lstat().st_mode
            _require(not stat.S_ISLNK(mode), "archive symlink forbidden")
            if stat.S_ISDIR(mode):
                _require(directories + len(pending) < 4096, "archive directory bound")
                pending.append(child)
            else:
                _require(stat.S_ISREG(mode), "archive nonregular file forbidden")
                if (
                    child.suffix == ".json"
                    and "audits" not in child.relative_to(root).parts
                ):
                    metadata.append(child)
                files += 1
                budget += child.stat().st_size
                _require(
                    files <= 10000 and budget <= 4 * 1024**3, "archive file/byte budget"
                )
    return frozenset(metadata)


LIMITS = dict(
    workers=1,
    rss_bytes=2147483648,
    wall_seconds=43200,
    campaign_bytes=4294967296,
    compact_row_bytes=32768,
    start_row_bytes=1024,
    audit_frame_bytes=32768,
    metadata_bytes=16777216,
)
POPULATION = dict(
    arms=66,
    cases=52224,
    settlements=17563648,
    initial_actor_steps=2054160384,
    maximum_population=768,
    maximum_case_work=393216,
)
IDENTITY_FIELDS = {
    "protocol_sha256",
    "proposal_sha256",
    "manifest_sha256",
    "compiled_inputs_sha256",
    "source_inventory_sha256",
    "resolved_roster_sha256",
    "binary_sha256",
}


def _exact_integers(actual, expected, name):
    _fields(actual, set(expected), name)
    for k, v in expected.items():
        _uint(actual[k], name)
        _require(actual[k] == v, f"{name} mismatch")


RESOURCE_FIELDS = {
    "schema",
    "manifest_sha256",
    "compiled_inputs_sha256",
    "binary_sha256",
    "budgets",
    "census",
    "reserved_bytes",
    "failure_reserve_bytes",
    "schema_bounds_valid",
    "aggregate_proof_valid",
    "forecast_valid",
    "forecast_method",
    "forecast_inputs",
    "forecast_wall_seconds",
    "forecast_peak_rss_bytes",
    "fixture_probes",
    "unavailable",
}


def validate_monitor(monitor, header):
    import math
    from decimal import Decimal

    _fields(
        monitor,
        {
            "schema",
            "mode",
            "binding",
            "child_pid",
            "exit_code",
            "platform",
            "peak_method",
            "wall_seconds",
            "peak_rss_bytes",
            "wall_limit_seconds",
            "rss_limit_bytes",
            "campaign_limit_bytes",
            "source_valid",
            "archive_valid",
        },
        "monitor",
    )
    _require(
        monitor["schema"] == "war1-study-monitor-result-v1"
        and monitor["mode"] == header["mode"]
        and monitor["binding"] == header["binding"],
        "monitor identity",
    )
    _uint(monitor["child_pid"], "child PID")
    _require(monitor["child_pid"] > 0, "positive child PID required")
    _uint(monitor["exit_code"], "child exit code")
    _require(monitor["exit_code"] == 0, "child did not exit successfully")
    _require(
        monitor["platform"] in ("darwin", "linux")
        and monitor["peak_method"] == "os.wait4.ru_maxrss",
        "individual child peak method",
    )
    wall = monitor["wall_seconds"]
    _require(
        type(wall) in (int, float, Decimal)
        and math.isfinite(wall)
        and 0 <= wall <= 43200,
        "wall measurement",
    )
    _uint(monitor["peak_rss_bytes"], "child peak", 2147483648)
    _require(monitor["peak_rss_bytes"] > 0, "positive child peak required")
    for k, v in [
        ("wall_limit_seconds", 43200),
        ("rss_limit_bytes", 2147483648),
        ("campaign_limit_bytes", 4294967296),
    ]:
        _uint(monitor[k], k)
        _require(monitor[k] == v, "monitor limit")
    _require(
        monitor["source_valid"] is True and monitor["archive_valid"] is True,
        "monitor validation decisions",
    )


def oracle_rows(oracles):
    from .analysis import rational

    return [
        dict(
            arm_id=o.arm_id,
            scalars={e: i.wire() for e, i in o.scalars.items()},
            cdfs={
                e: [dict(point=rational(p), interval=i.wire()) for p, i in points]
                for e, points in o.cdfs.items()
            },
            numerical_certificate=o.numerical_certificate,
            ctmc_one_vs_one=o.ctmc_one_vs_one,
        )
        for o in oracles
    ]


def oracle_packet(manifest, compiled_inputs_sha256, oracles):
    _hash(compiled_inputs_sha256)
    return dict(
        schema="war1-study-oracle-packet-v1",
        manifest_sha256=manifest.sha256,
        compiled_inputs_sha256=compiled_inputs_sha256,
        oracles=oracle_rows(oracles),
    )


def validate_oracle_packet(packet, manifest, numeric_export):
    """Recompute the entire packet once from retained native numeric inputs."""
    from .oracle import prepare_oracles
    from .io import canonical_bytes

    _fields(
        packet,
        {"schema", "manifest_sha256", "compiled_inputs_sha256", "oracles"},
        "oracle packet",
    )
    _require(
        numeric_export["binding_state"] == "source_bound",
        "source-bound numeric export required",
    )
    oracles = prepare_oracles(manifest, numeric_export)
    expected = oracle_packet(
        manifest, numeric_export["compiled_inputs_sha256"], oracles
    )
    # Recursive canonical equality refuses extras, missing nullable keys, changed
    # exact rationals, ordering, supports, scalar names and numeric certificates.
    _require(
        canonical_bytes(packet) == canonical_bytes(expected),
        "saved oracle packet differs from recomputed exact law",
    )
    return oracles


def _command(command):
    _require(
        isinstance(command, list) and 1 <= len(command) <= 32, "bounded command argv"
    )
    for arg in command:
        _text(arg, "command argument", 1024)


def validate_retained_receipts(root, header, manifest):
    """Validate frozen links using fixed retained paths, never original paths.

    Returns the recomputed numerical oracles for reuse by the caller. This is
    evidence validation, not authority to launch a process or publish success.
    """
    from .io import canonical_bytes
    from .manifest import proposal, PROTOCOL_SHA256, PROPOSAL_SHA256
    from pathlib import Path
    import math

    d, db = _json(root, "provenance/declaration.json")
    a, ab = _json(root, "provenance/acceptance.json")
    e, eb = _json(root, "provenance/execution.json")
    _fields(
        d,
        {
            "schema",
            "purpose",
            "identities",
            "build",
            "numeric",
            "resources",
            "source_inventory",
            "manifest",
            "binary",
            "normative_frozen_revision",
            "declaration_revision",
            "registered_histories",
            "census",
            "budgets",
        },
        "declaration",
    )
    _fields(
        a,
        {
            "schema",
            "declaration_sha256",
            "reviewer_identity",
            "reviewer_role",
            "decision",
            "registered_histories",
            "open_required_findings",
            "build_sha256",
            "numeric_sha256",
            "resources_sha256",
            "binary_sha256",
            "manifest_sha256",
        },
        "acceptance",
    )
    _fields(
        e,
        {
            "schema",
            "declaration_sha256",
            "acceptance_sha256",
            "owner_identity",
            "action",
            "census",
            "budgets",
            "authorization_quote",
            "authorized_at",
            "compiled_inputs_sha256",
            "source_inventory_sha256",
            "binary_sha256",
            "manifest_sha256",
        },
        "execution",
    )
    _require(
        d["schema"] == "war1-study-declaration-v1"
        and a["schema"] == "war1-study-readiness-v1"
        and e["schema"] == "war1-study-execution-v1",
        "receipt schemas",
    )
    _require(
        d["purpose"] == "prospective_frozen_not_blind"
        and a["decision"] == "accepted"
        and e["action"] == "execute_exact_population",
        "receipt decisions",
    )
    for receipt in (d, a):
        _uint(receipt["registered_histories"], "registered histories")
        _require(receipt["registered_histories"] == 0, "prospective histories required")
    _require(a["open_required_findings"] == [], "open readiness findings")
    for r, key in (
        (a, "reviewer_identity"),
        (e, "owner_identity"),
        (e, "authorized_at"),
    ):
        _text(r[key], key)
    _text(e["authorization_quote"], "authorization quote", 4096)
    _require(bool(e["authorization_quote"].strip()), "empty authorization quote")
    _require(
        a["reviewer_role"] == "independent_reviewer"
        and a["reviewer_identity"] != e["owner_identity"],
        "independent review",
    )
    for r in (d, e):
        _exact_integers(r["census"], POPULATION, "population")
        _exact_integers(r["budgets"], LIMITS, "budgets")
    for key in ("normative_frozen_revision", "declaration_revision"):
        v = d[key]
        _require(
            isinstance(v, str)
            and len(v) == 40
            and all(c in "0123456789abcdefABCDEF" for c in v),
            "frozen revision",
        )
    _require(
        d["normative_frozen_revision"] != d["declaration_revision"],
        "distinct frozen revisions",
    )
    binding = header["binding"]
    ids = d["identities"]
    _fields(ids, IDENTITY_FIELDS, "identities")
    for value in ids.values():
        _hash(value)
    for key, value in [
        ("protocol_sha256", PROTOCOL_SHA256),
        ("proposal_sha256", PROPOSAL_SHA256),
        ("manifest_sha256", manifest.sha256),
        ("resolved_roster_sha256", manifest.raw["roster_sha256"]),
        ("compiled_inputs_sha256", binding["compiled_inputs_sha256"]),
        ("binary_sha256", binding["executable_sha256"]),
    ]:
        _require(ids[key] == value, "frozen identity mismatch")
    for key, raw in [
        ("declaration_sha256", db),
        ("acceptance_sha256", ab),
        ("execution_approval_sha256", eb),
    ]:
        _require(
            binding[key] == hashlib.sha256(raw).hexdigest(), "header receipt binding"
        )
    _require(
        a["declaration_sha256"]
        == binding["declaration_sha256"]
        == e["declaration_sha256"]
        and e["acceptance_sha256"] == binding["acceptance_sha256"],
        "receipt chain",
    )
    for name in ("build", "numeric", "resources"):
        _require(
            a[f"{name}_sha256"] == d[name]["sha256"], "acceptance artifact binding"
        )
    for receipt in (a, e):
        for key in ("binary_sha256", "manifest_sha256"):
            _require(receipt[key] == ids[key], "authorization identity")
    for key in ("compiled_inputs_sha256", "source_inventory_sha256"):
        _require(e[key] == ids[key], "execution source identity")
    return validate_preparation(root, manifest, d)


def validate_resource_receipt(r, ids):
    """Complete strict resource schema shared by preseed and saved packets."""
    import math

    _fields(r, RESOURCE_FIELDS, "resources")
    _require(r["schema"] == "war1-study-resources-v1", "resource schema")
    for key in ("manifest_sha256", "compiled_inputs_sha256", "binary_sha256"):
        _hash(r[key])
        _require(r[key] == ids[key], "resource binding")
    _exact_integers(r["budgets"], LIMITS, "budgets")
    _exact_integers(r["census"], POPULATION, "census")
    for key in ("schema_bounds_valid", "aggregate_proof_valid", "forecast_valid"):
        _require(r[key] is True, "resource decision")
    _uint(r["reserved_bytes"], "reserved bytes", 4294967296)
    _uint(r["failure_reserve_bytes"], "failure reserve", 4294967296)
    _require(
        1048576 <= r["failure_reserve_bytes"] <= r["reserved_bytes"]
        and r["unavailable"] == [],
        "resource reserve/availability",
    )
    _text(r["forecast_method"], "forecast method")
    validate_artifact_descriptor(r["forecast_inputs"])
    from decimal import Decimal

    wall = r["forecast_wall_seconds"]
    _require(
        type(wall) in (int, float, Decimal)
        and math.isfinite(wall)
        and 0 < wall <= 43200,
        "forecast wall",
    )
    _uint(r["forecast_peak_rss_bytes"], "forecast RSS", 2147483648)
    _require(r["forecast_peak_rss_bytes"] > 0, "forecast RSS")
    probes = r["fixture_probes"]
    _require(isinstance(probes, list) and 1 <= len(probes) <= 32, "probe count")
    for index, p in enumerate(probes):
        _fields(
            p,
            {
                "artifact",
                "command",
                "compiled_inputs_sha256",
                "binary_sha256",
                "wall_seconds",
                "peak_rss_bytes",
                "fixture_purpose",
            },
            "probe",
        )
        _command(p["command"])
        _hash(p["compiled_inputs_sha256"])
        _hash(p["binary_sha256"])
        _require(p["fixture_purpose"] == "construction_fixture", "probe purpose")
        _require(
            type(p["wall_seconds"]) in (int, float, Decimal)
            and math.isfinite(p["wall_seconds"])
            and p["wall_seconds"] > 0,
            "probe wall",
        )
        _uint(p["peak_rss_bytes"], "probe RSS", 2147483648)
        _require(p["peak_rss_bytes"] > 0, "probe RSS")
        validate_artifact_descriptor(p["artifact"])


def validate_preparation(root, manifest, d):
    """Shared strict frozen build/numeric/resource packet validation.

    Deliberately independent of future reviewer/owner receipts, so preparation
    never needs to invent an activation chain to validate its own evidence.
    """
    from pathlib import Path
    import math
    from .io import canonical_bytes
    from .manifest import proposal

    ids = d["identities"]
    _fields(ids, IDENTITY_FIELDS, "identities")
    for value in ids.values():
        _hash(value)
    _require(ids["manifest_sha256"] == manifest.sha256, "preparation manifest")
    _require(
        ids["resolved_roster_sha256"] == manifest.raw["roster_sha256"],
        "preparation roster",
    )
    for key in ("protocol_sha256", "proposal_sha256"):
        _require(ids[key] == manifest.raw[key], "preparation normative identity")
    _exact_integers(d["budgets"], LIMITS, "budgets")
    _exact_integers(d["census"], POPULATION, "population")
    _artifact(root, "manifest.json", d["manifest"])
    _require(d["manifest"]["sha256"] == manifest.sha256, "declared manifest hash")
    _fields(d["binary"], {"path", "sha256", "bytes"}, "binary descriptor")
    _text(d["binary"]["path"], "binary path", 4096)
    _hash(d["binary"]["sha256"])
    _uint(d["binary"]["bytes"], "binary bytes")
    _require(
        d["binary"]["sha256"] == ids["binary_sha256"] and d["binary"]["bytes"] > 0,
        "declared binary identity",
    )
    inventory = _artifact(
        root, "provenance/source-inventory.json", d["source_inventory"]
    )
    _require(
        d["source_inventory"]["sha256"] == ids["source_inventory_sha256"],
        "frozen source inventory hash",
    )
    native_inventory, _ = _json(root, "source-inventory.json")
    validate_inventory(inventory)
    validate_inventory(native_inventory)
    _require(inventory == native_inventory, "frozen/native source inventory semantics")
    b = _artifact(root, "provenance/build.json", d["build"])
    _fields(
        b,
        {
            "schema",
            "identities",
            "selected_path",
            "selected_purpose",
            "binary",
            "rustc",
            "cargo",
            "platform",
            "features",
            "flags",
            "build_command",
            "inventory_before",
            "inventory_after",
        },
        "build",
    )
    _require(
        b["schema"] == "war1-study-build-v1"
        and b["identities"] == ids
        and canonical_bytes(b["binary"]) == canonical_bytes(d["binary"])
        and b["selected_purpose"] == manifest.purpose,
        "build binding",
    )
    _require(
        canonical_bytes(b["inventory_before"])
        == canonical_bytes(inventory["entries"])
        == canonical_bytes(b["inventory_after"]),
        "build inventory",
    )
    _require(
        any(
            i["path"] == b["selected_path"] and i["sha256"] == manifest.sha256
            for i in inventory["entries"]
        ),
        "selected manifest input",
    )
    _command(b["build_command"])
    for name in ("rustc", "cargo", "platform"):
        _text(b[name], name, 4096)
    _require(
        isinstance(b["features"], list)
        and len(b["features"]) <= 128
        and b["features"] == sorted(set(b["features"])),
        "build features",
    )
    for feature in b["features"]:
        _text(feature, "build feature")
        _require(feature.startswith("CARGO_FEATURE_"), "feature name")
    _require(isinstance(b["flags"], list) and len(b["flags"]) == 7, "build flags")
    for pair, name in zip(
        b["flags"],
        (
            "RUSTFLAGS",
            "CARGO_ENCODED_RUSTFLAGS",
            "TARGET",
            "HOST",
            "PROFILE",
            "OPT_LEVEL",
            "DEBUG",
        ),
    ):
        _require(
            isinstance(pair, list)
            and len(pair) == 2
            and pair[0] == name
            and isinstance(pair[1], str)
            and len(pair[1].encode()) <= 4096,
            "ordered build flags",
        )
    n = _artifact(root, "provenance/numeric.json", d["numeric"])
    _fields(
        n,
        {
            "schema",
            "manifest_sha256",
            "compiled_inputs_sha256",
            "binary_sha256",
            "numeric_export",
            "dose_proof",
            "oracle_packet",
            "certificate_valid",
            "mass_valid",
            "width_valid",
            "ulp_valid",
            "ctmc_valid",
            "unresolved_findings",
            "oracle_ids",
        },
        "numeric receipt",
    )
    _require(n["schema"] == "war1-study-numeric-v1", "numeric schema")
    for key in ("manifest_sha256", "compiled_inputs_sha256", "binary_sha256"):
        _require(n[key] == ids[key], "numeric identity")
    for key in (
        "certificate_valid",
        "mass_valid",
        "width_valid",
        "ulp_valid",
        "ctmc_valid",
    ):
        _require(n[key] is True, "numeric validity")
    approved = proposal(Path(__file__).resolve().parents[2])
    _require(
        n["unresolved_findings"] == []
        and n["oracle_ids"]
        == [a["arm_id"] for a in approved["arms"] if a["group"] == "oracle"],
        "numeric oracle census",
    )
    from .dose_proof import validate_dose_proof

    proof = _artifact(root, "provenance/dose-proof.json", n["dose_proof"])
    validate_dose_proof(
        proof,
        Path(__file__).resolve().parents[2],
        manifest.sha256,
        ids["compiled_inputs_sha256"],
    )
    export = _artifact(root, "provenance/numeric-export.json", n["numeric_export"])
    packet = _artifact(root, "provenance/oracle-packet.json", n["oracle_packet"])
    _require(
        export["compiled_inputs_sha256"] == ids["compiled_inputs_sha256"],
        "numeric source digest",
    )
    r = _artifact(root, "provenance/resources.json", d["resources"])
    validate_resource_receipt(r, ids)
    minimum_reserve = (
        sum(entry["bytes"] for entry in inventory["entries"])
        + len(manifest.cases) * (32768 + 1024)
        + sum((arm["config"]["max_steps"] + 3) * 32768 for arm in manifest.arms)
        + r["failure_reserve_bytes"]
    )
    _require(
        r["reserved_bytes"] >= minimum_reserve,
        "reservation below independent source/row/audit bound",
    )
    _artifact(root, "provenance/resource-forecast-inputs.json", r["forecast_inputs"])
    for index, probe in enumerate(r["fixture_probes"]):
        _artifact(root, f"provenance/fixture-probes/{index:06}.json", probe["artifact"])
    oracles = validate_oracle_packet(packet, manifest, export)
    for oracle in oracles:
        if oracle.ctmc_one_vs_one and oracle.ctmc_one_vs_one["required_at_this_dt"]:
            _require(
                all(
                    c["passes"] is True
                    for c in oracle.ctmc_one_vs_one["criteria"].values()
                ),
                "required CTMC criteria failed",
            )
    return oracles


def inference_provenance(root, header, manifest, candidate):
    if manifest.purpose == "construction_fixture":
        return False, "unregistered_construction_panel"
    try:
        success, _ = _json(root, "success.json")
        _fields(
            success,
            {"schema", "mode", "binding", "candidate", "receipt", "census", "monitor"},
            "success",
        )
        _require(
            success["schema"] == "war1-study-success-v1"
            and success["mode"] == header["mode"]
            and success["binding"] == header["binding"],
            "success binding",
        )
        _require(
            success["candidate"]["path"] == "candidate-index.json"
            and success["monitor"]["path"] == "monitor.json",
            "success retained paths",
        )
        saved = _artifact(root, "candidate-index.json", success["candidate"])
        for value in (success, saved, candidate):
            validate_receipt(value["receipt"], len(manifest.cases))
            validate_census(value["census"], len(manifest.cases))
        _require(saved == candidate, "success candidate binding")
        _require(
            success["receipt"] == candidate["receipt"]
            and success["census"] == candidate["census"],
            "success census/receipt",
        )
        monitor = _artifact(root, "monitor.json", success["monitor"])
        validate_monitor(monitor, header)
        oracles = validate_retained_receipts(root, header, manifest)
        from .io import canonical_bytes

        header["validated_oracle_rows_sha256"] = hashlib.sha256(
            canonical_bytes(oracle_rows(oracles))
        ).hexdigest()
        return True, None
    except (ValueError, OSError, KeyError, TypeError) as error:
        return False, f"monitored_success_or_authorization_unavailable: {error}"
