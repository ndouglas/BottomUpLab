"""Create-only prospective input preparation and noncircular build identities.

No function in this module constructs an Engine. Only prepare opens scientific
entropy; callers must satisfy the resource gate before invoking it.
"""
import hashlib
import os
import re
from pathlib import Path
import stat
import subprocess

from .contracts import Json, Manifest
from .io import METADATA_LIMIT, _checked_path, canonical_bytes, create_bytes, read_regular, strict_json
from .manifest import sample_manifest, validate_manifest
from .archive_provenance import SOURCE_REQUIRED, LITERAL_INVENTORY, literal_paths, validate_inventory

DEFAULT_MANIFEST = "survey/war1_study/fixtures/construction-manifest.json"


def load(path: Path) -> Json:
    return strict_json(read_regular(path, METADATA_LIMIT), METADATA_LIMIT)


def save(path: Path, value: Json) -> Json:
    create_bytes(path, canonical_bytes(value), METADATA_LIMIT)
    return value


def artifact(path: Path, limit=METADATA_LIMIT) -> Json:
    path = _checked_path(path)
    if not stat.S_ISREG(path.stat().st_mode) or path.stat().st_size > limit:
        raise ValueError("artifact must be a bounded regular file")
    digest = hashlib.sha256()
    count = 0
    with path.open("rb") as source:
        while block := source.read(min(65536, limit - count + 1)):
            count += len(block)
            if count > limit:
                raise ValueError("artifact grew beyond bound")
            digest.update(block)
    return dict(path=str(path), sha256=digest.hexdigest(), bytes=count)


def _source_files(root: Path, relative: str):
    """Streaming enumeration; queued directories are bounded before retention."""
    pending = [root / relative]
    directories = 0
    files = 0
    while pending:
        directory = _checked_path(pending.pop())
        directories += 1
        with os.scandir(directory) as children:
            for child in children:
                path = _checked_path(Path(child.path))
                mode = child.stat(follow_symlinks=False).st_mode
                if stat.S_ISDIR(mode):
                    if directories + len(pending) >= 4096:
                        raise ValueError("source directory bound")
                    pending.append(path)
                elif stat.S_ISREG(mode):
                    files += 1
                    if files > 4096:
                        raise ValueError("source entry bound")
                    yield path
                else:
                    raise ValueError("nonregular source")


# Concrete closed literal macros used by core's source identity receipts. Match
# the complete bodies (with flexible whitespace), never arbitrary macro Rust.
_SOURCE_MACRO_BODIES = (
    r'''macro_rules! source {
        ($path:literal) => {
            (concat!("crates/sugarscape-core/src/", $path),
             include_bytes!(concat!("../", $path)).as_slice(),)
        };
    }''',
    r'''macro_rules! source {
        ($path:literal) => {
            (concat!("crates/sugarscape-core/src/", $path),
             include_bytes!(concat!("../../", $path)).as_slice(),)
        };
    }''',
    r'''macro_rules! source {
        ($path:literal) => {
            ArtifactReceipt {
                path: concat!("crates/sugarscape-core/src/", $path).into(),
                bytes: include_bytes!(concat!("../../", $path)).len() as u64,
                sha256: hash::sha256(include_bytes!(concat!("../../", $path))),
            }
        };
    }''',
)
# Lean identity wrappers preserve the same exact native include expansions.
_SOURCE_MACRO_BODIES += tuple(body.replace('include_bytes!', 'identity_bytes!').replace('.as_slice(),)', ',)') for body in _SOURCE_MACRO_BODIES[:2])
_SOURCE_MACRO_BODIES += (r'''macro_rules! source {
    ($path:literal) => {{
        let path = concat!("crates/sugarscape-core/src/", $path);
        let raw = crate::identity_bytes::identity_bytes!(concat!("../../", $path));
        let (bytes, sha256) = crate::compact_receipts::raw(path, raw).expect("compiled source receipt");
        ArtifactReceipt { path: path.into(), bytes, sha256, }
    }};
}''',)
_SOURCE_MACRO_BODIES += tuple(body.replace("$path", "$p") for body in _SOURCE_MACRO_BODIES)
_SOURCE_MACROS = tuple(re.compile(r"\s*".join(re.escape(token) for token in
    re.findall(r'"[^"\n]*"|[A-Za-z_][A-Za-z_0-9]*|=>|::|[^\s]', body)))
    for body in _SOURCE_MACRO_BODIES)


def _audit_source_macro(text, path, root):
    if re.search(r"\b(?:source|input|identity_bytes|include(?:_str|_bytes)?)\s*/[/*]", text):
        raise ValueError("comment-interposed include/macro syntax is unsupported")
    definitions = list(re.finditer(r"\bmacro_rules\s*!\s*source\b", text))
    calls = list(re.finditer(r"\bsource\s*!(?!=)", text))
    if not definitions and not calls:
        return text
    matches = [m for pattern in _SOURCE_MACROS for m in pattern.finditer(text)]
    if len(definitions) != 1 or len(matches) != 1 or definitions[0].start() != matches[0].start():
        raise ValueError("unsupported source literal macro definition")
    match = matches[0]
    text = text[:match.start()] + text[match.end():]
    literals = list(re.finditer(r'(?<![A-Za-z_0-9:])source\s*!\s*\(\s*"([^"\\\n]+)"\s*\)', text))
    if not calls or len(calls) > 4096 or len(literals) != len(calls):
        raise ValueError("unsupported or unresolved source literal macro call")
    core = root / "crates/sugarscape-core/src"
    for literal in literals:
        value = literal.group(1)
        relative = Path(value)
        if (len(value.encode()) > 4096 or not re.fullmatch(r"[A-Za-z_0-9./-]+", value)
            or relative.is_absolute() or any(part in ("", ".", "..") for part in value.split("/"))):
            raise ValueError("noncanonical source literal macro path")
        # Validate the expansion relative to the actual including file, then
        # require the path label and expanded target to agree in the bound tree.
        prefix = "../" if '"../"' in match.group() else "../../"
        target = _checked_path(path.parent / prefix / relative).resolve()
        if target != core / relative:
            raise ValueError("source literal macro outside bound core tree")
        read_regular(target, METADATA_LIMIT)
    return text


_IDENTITY_MACRO_BODY = r'''macro_rules! identity_bytes {
    ($p:expr) => {{
        #[cfg(not(target_arch = "wasm32"))] { include_bytes!($p).as_slice() }
        #[cfg(target_arch = "wasm32")] { b"".as_slice() }
    }};
}'''
def _token_pattern(body):
    return re.compile(r"\s*".join(re.escape(token) for token in
        re.findall(r'"[^"\n]*"|[A-Za-z_][A-Za-z_0-9]*|=>|::|[^\s]', body)))


def _audit_identity_macro(text, path, root):
    notes_input = r'''macro_rules! input {
        ($p:literal) => {
            (concat!("crates/sugarscape-core/src/persistent_notes/", $p), identity_bytes!($p),)
        };
    }'''
    if path == root / "crates/sugarscape-core/src/persistent_notes/source_inputs.rs":
        matches = list(_token_pattern(notes_input).finditer(text))
        if len(matches) != 1:
            raise ValueError("unsupported Notes input macro definition")
        match = matches[0]
        text = text[:match.start()] + text[match.end():]
        calls = list(re.finditer(r'\binput\s*!\s*\(\s*"([^"\\\n]+)"\s*\)', text))
        if len(calls) != len(re.findall(r"\binput\s*!", text)):
            raise ValueError("unsupported Notes input macro call")
        for call in reversed(calls):
            value = call.group(1)
            if not re.fullmatch(r"[A-Za-z_0-9/-]+\.rs|tests/fixtures/[A-Za-z_0-9-]+\.json", value):
                raise ValueError("noncanonical Notes input path")
            text = text[:call.start()] + 'identity_bytes!("' + value + '")' + text[call.end():]
    if path == root / "crates/sugarscape-core/src/identity_bytes.rs":
        matches = list(_token_pattern(_IDENTITY_MACRO_BODY).finditer(text))
        if len(matches) != 1 or len(re.findall(r"macro_rules\s*!\s*identity_bytes", text)) != 1:
            raise ValueError("unsupported identity literal macro definition")
        match = matches[0]
        text = text[:match.start()] + text[match.end():]
    if re.search(r"macro_rules\s*!\s*identity_bytes", text):
        raise ValueError("identity literal macro outside bound definition")
    literal = r'"([^"\\\n]+)"'
    expression = rf'(?:{literal}|concat!\(\s*{literal}(?:\s*,\s*{literal})*\s*,?\s*\))'
    pattern = re.compile(rf'\bidentity_bytes\s*!\s*\(\s*({expression})\s*\)')
    matches = list(pattern.finditer(text))
    if len(matches) != len(re.findall(r"\bidentity_bytes\s*!", text)):
        raise ValueError("unsupported identity literal macro call")
    for match in reversed(matches):
        value = ''.join(re.findall(literal, match.group(1)))
        if (len(value.encode()) > 4096 or not re.fullmatch(r"[A-Za-z_0-9./-]+", value)
                or Path(value).is_absolute() or any(part in ("", ".") for part in value.split('/'))):
            raise ValueError("noncanonical identity literal path")
        target = _checked_path(path.parent / value).resolve()
        if not target.is_relative_to(root):
            raise ValueError("identity literal outside bound repository")
        # This precise unrelated reference was already omitted by the native W1 selector.
        if str(target.relative_to(root)) == "crates/sugarscape-core/src/persistent_notes/tests/fixtures/evaluation-reference.json":
            if not stat.S_ISREG(target.stat().st_mode):
                raise ValueError("Notes reference is not regular")
        else:
            read_regular(target, METADATA_LIMIT)
        text = text[:match.start()] + 'include_bytes!("' + value + '")' + text[match.end():]
    return text


_GENERATED_INCLUDES = {
    "crates/sugarscape-core/src/gate_assistance/diagnostics.rs": (
        'include!(concat!(env!("OUT_DIR"), "/gate_source_receipts.rs"))',),
    "crates/sugarscape-core/src/compact_receipts.rs": (
        'include!(concat!(env!("OUT_DIR"), "/compact_raw_receipts.rs"))',),
    "crates/sugarscape-core/src/browser_experiments/recorded.rs": tuple(
        'include_str!(concat!(env!("OUT_DIR"), "/' + name + '-selected.json"))'
        for name in ("strategic", "strategy", "audit")),
}


def _audit_generated_includes(text, path, root):
    for expression in _GENERATED_INCLUDES.get(str(path.relative_to(root)), ()):
        if text.count(expression) != 1:
            raise ValueError("unsupported bound generated receipt include")
        text = text.replace(expression, "", 1)
        read_regular(root / "crates/sugarscape-core/build.rs", METADATA_LIMIT)
        generator = "surface_discovery.rs" if "gate_source_receipts" in expression else "compact_receipts.rs"
        read_regular(root / "crates/sugarscape-core/build_support" / generator, METADATA_LIMIT)
    return text


def audit_literal_inputs(root: Path):
    expected = set(literal_paths(read_regular(root / LITERAL_INVENTORY, METADATA_LIMIT)))
    found = set()
    pattern = re.compile(r'include_(?:str|bytes)\s*!\s*\(\s*"([^"\\\n]+)"\s*,?\s*\)')
    retained = 0
    for index, path in enumerate(p for p in _source_files(root, "crates/sugarscape-core/src") if p.suffix == ".rs"):
        if index >= 4096:
            raise ValueError("literal include audit source entry bound")
        raw = read_regular(path, min(METADATA_LIMIT, 64*1024**2 - retained))
        retained += len(raw)
        text = _audit_source_macro(raw.decode("utf-8"), path, root)
        text = _audit_identity_macro(text, path, root)
        text = _audit_generated_includes(text, path, root)
        matches = list(pattern.finditer(text))
        if len(matches) != len(re.findall(r"include_(?:str|bytes)\s*!", text)) or re.search(r"\binclude\s*!", text):
            raise ValueError("unsupported or unresolved core include syntax")
        for match in matches:
            target = (path.parent / match.group(1)).resolve()
            if not target.is_relative_to(root / "crates/sugarscape-core/src"):
                relative = str(target.relative_to(root))
                if relative in (
                    "Cargo.toml",
                    "Cargo.lock",
                    "crates/sugarscape-core/Cargo.toml",
                    "crates/sugarscape-core/build.rs",
                    "crates/sugarscape-core/build_support/surface_discovery.rs",
                    "crates/sugarscape-core/build_support/compact_receipts.rs",
                    "docs/superpowers/protocols/2026-10-14-paid-persistent-notes.md",
                ):
                    # These exact mandatory manifests/host sources are separate from the fixed
                    # external asset inventory and must actually exist.
                    read_regular(target, METADATA_LIMIT)
                else:
                    found.add(relative)
    if expected != found:
        raise ValueError("core literal inventory differs from actual include expressions")
    return found


def source_inventory(root: Path, selected: str) -> Json:
    """Match the native selected closure, with bounds before each allocation."""
    root = _checked_path(root)
    audit_literal_inputs(root)
    parts = Path(selected).parts
    if (not parts or Path(selected).is_absolute() or ".." in parts or "\\" in selected
        or str(Path(selected)) != selected
        or (selected != DEFAULT_MANIFEST and not (selected.startswith("survey/war1_study/manifests/") and selected.endswith(".json")))):
        raise ValueError("invalid selected manifest path")
    paths = set(SOURCE_REQUIRED) | {selected} | set(literal_paths(read_regular(root / LITERAL_INVENTORY, METADATA_LIMIT)))
    for base in ("crates/sugarscape-core/src", "survey/src/war1_study"):
        for path in _source_files(root, base):
            relative = str(path.relative_to(root))
            # Match the pre-existing native W1 closure; Notes verifies this separate large reference.
            if relative == "crates/sugarscape-core/src/persistent_notes/tests/fixtures/evaluation-reference.json":
                continue
            paths.add(relative)
            if len(paths) > 4096:
                raise ValueError("source entry bound")
    with os.scandir(root / "survey/war1_study") as children:
        for item in children:
            if item.name.endswith(".py"):
                paths.add(str(Path(item.path).relative_to(root)))
                if len(paths) > 4096:
                    raise ValueError("source entry bound")
    digest = hashlib.sha256(b"war1-study-compiled-inputs-v1\0")
    entries = []
    total = 0
    for name in sorted(paths):
        if len(name.encode()) > 4096:
            raise ValueError("source path bound")
        raw = read_regular(root / name, min(METADATA_LIMIT, 64 * 1024**2 - total))
        total += len(raw)
        for data in (name.encode(), raw):
            digest.update(len(data).to_bytes(8, "big"))
            digest.update(data)
        entries.append(dict(path=name, sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw)))
    value = dict(schema="war1-study-source-inventory-v1", compiled_inputs_sha256=digest.hexdigest(), entries=entries)
    validate_inventory(value)
    return value


def prepare(root: Path, out_dir: Path) -> Path:
    root, out_dir = _checked_path(root), _checked_path(out_dir)
    if out_dir.exists():
        raise ValueError("preparation destination already exists; never redraw")
    from .preparation import validate_preseed
    validate_preseed(root)
    # Reserve destination first: failures preserve it and cannot silently redraw.
    out_dir.mkdir(mode=0o700)
    (out_dir / "manifests").mkdir(mode=0o700)
    command = ("python3", "-m", "survey.war1_study", "prepare", "--root", str(root), "--out-dir", str(out_dir))
    raw = sample_manifest(root, os.urandom, command=command)
    manifest = validate_manifest(raw, root)
    path = out_dir / "manifests" / f"frozen-{manifest.sha256}.json"
    create_bytes(path, raw, METADATA_LIMIT)
    return path


def _git(root, *args):
    child = subprocess.Popen(("git", "-C", str(root), *args), stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    try:
        raw = child.stdout.read(METADATA_LIMIT + 1)
        if len(raw) > METADATA_LIMIT:
            child.kill()
            raise ValueError("Git metadata exceeds bound")
        if child.wait() != 0:
            raise ValueError("Git input/revision validation failed")
        return raw.decode().strip()
    finally:
        child.stdout.close()
        if child.returncode is None:
            child.kill()
            child.wait()


def freeze(root: Path, manifest_path: Path, out: Path) -> Json:
    root = _checked_path(root)
    manifest_path = _checked_path(manifest_path)
    selected = str(manifest_path.relative_to(root))
    validate_manifest(read_regular(manifest_path, METADATA_LIMIT), root)
    inventory = source_inventory(root, selected)
    for entry in inventory["entries"]:
        if _git(root, "status", "--porcelain", "--untracked-files=all", "--", entry["path"]):
            raise ValueError("freeze requires committed unchanged selected inputs")
        _git(root, "ls-files", "--error-unmatch", "--", entry["path"])
    save(out, inventory)
    save(out.parent / "freeze-context.json", dict(schema="war1-study-freeze-context-v1", root=str(root), selected_path=selected,
         normative_frozen_revision=_git(root, "rev-parse", "HEAD"), source_inventory=artifact(out)))
    return inventory


def native_metadata(binary: Path) -> Json:
    # Pure native command. Bound the pipe while reading; do not capture unbounded output.
    with subprocess.Popen((str(binary), "metadata"), stdout=subprocess.PIPE, stderr=subprocess.DEVNULL) as child:
        raw = child.stdout.read(METADATA_LIMIT + 1)
        if len(raw) > METADATA_LIMIT:
            child.kill()
            raise ValueError("native metadata exceeds bound")
        if child.wait() != 0:
            raise ValueError("native metadata/source validation failed")
    value = strict_json(raw, METADATA_LIMIT)
    if value.get("schema") != "war1-study-metadata-v1":
        raise ValueError("native metadata schema")
    return value


def build_receipt(root: Path, manifest: Manifest, inventory: Json,
                  binary: Path, build_command: tuple[str, ...], out: Path) -> Json:
    from .manifest import _text
    validate_inventory(inventory)
    if not isinstance(build_command, tuple) or not 1 <= len(build_command) <= 32:
        raise ValueError("build command must be exact bounded argv")
    for arg in build_command:
        _text(arg, "build argument", 1024)
    metadata = native_metadata(binary)
    if Path(metadata["root"]) != root.absolute() or metadata["inventory"] != inventory:
        raise ValueError("build source/root inventory changed")
    selected = metadata["selected_path"]
    current = source_inventory(root, selected)
    if current != inventory or metadata["binding"]["manifest_sha256"] != manifest.sha256:
        raise ValueError("postbuild source/manifest changed")
    descriptor = artifact(binary, 1024**3)
    if descriptor["sha256"] != metadata["binding"]["executable_sha256"]:
        raise ValueError("postbuild binary changed")
    frozen_path = out.parent / "source-inventory.json"
    if load(frozen_path) != inventory:
        raise ValueError("fixed source-inventory packet mismatch")
    ids = {key: manifest.raw[key] for key in ("protocol_sha256", "proposal_sha256")}
    ids.update(manifest_sha256=manifest.sha256, compiled_inputs_sha256=current["compiled_inputs_sha256"],
               source_inventory_sha256=artifact(frozen_path)["sha256"], resolved_roster_sha256=manifest.raw["roster_sha256"], binary_sha256=descriptor["sha256"])
    return save(out, dict(schema="war1-study-build-v1", identities=ids, selected_path=selected,
        selected_purpose=manifest.purpose, binary=descriptor, rustc=metadata["rustc"], cargo=metadata["cargo"],
        platform=metadata["platform"], features=metadata["features"], flags=metadata["flags"],
        build_command=list(build_command), inventory_before=inventory["entries"], inventory_after=current["entries"]))


def _validated_declaration(build: Json, numeric: Json, resources: Json, out: Path) -> Json:
    """Declare an existing fixed packet without inventing review or execution.

    declaration_revision is the actual excluded-receipt HEAD at creation (the
    declaration's base), not the SHA of a commit containing its own JSON bytes.
    The containing commit is recorded externally in the subsequent handoff.
    """
    import tempfile
    from .archive_provenance import LIMITS, POPULATION, validate_preparation
    from .supervisor import _copy_descriptor
    parent = _checked_path(out.parent)
    for name, value in (("build", build), ("numeric", numeric), ("resources", resources)):
        if canonical_bytes(load(parent / f"{name}.json")) != canonical_bytes(value):
            raise ValueError("declaration inputs differ from fixed raw packet")
    context = load(parent / "freeze-context.json")
    from .manifest import _fields
    _fields(context, {"schema", "root", "selected_path", "normative_frozen_revision", "source_inventory"}, "freeze context")
    if context["schema"] != "war1-study-freeze-context-v1" or context["selected_path"] != build["selected_path"] or not re.fullmatch("[0-9a-f]{40}", context["normative_frozen_revision"]):
        raise ValueError("freeze context identity/revision")
    root = _checked_path(Path(context["root"]))
    inventory = load(parent / "source-inventory.json")
    if any(artifact(parent / "source-inventory.json")[k] != context["source_inventory"][k] for k in ("sha256", "bytes")):
        raise ValueError("freeze context inventory mismatch")
    if source_inventory(root, build["selected_path"]) != inventory:
        raise ValueError("source changed after freeze")
    metadata = native_metadata(Path(build["binary"]["path"]))
    if Path(metadata["root"]) != root or metadata["inventory"] != inventory or artifact(Path(build["binary"]["path"]), 1024**3) != build["binary"]:
        raise ValueError("binary/source changed after build")
    for name in ("rustc", "cargo", "platform", "flags", "features"):
        if metadata[name] != build[name]:
            raise ValueError("build metadata differs from actual binary")
    if metadata["selected_purpose"] != "prospective_seed_population":
        raise ValueError("declaration requires a prospective selected binary")
    current_revision = _git(root, "rev-parse", "HEAD")
    frozen_revision = context["normative_frozen_revision"]
    if current_revision == frozen_revision or _git(root, "merge-base", frozen_revision, current_revision) != frozen_revision:
        raise ValueError("distinct descendant excluded-receipt declaration revision required")
    for entry in inventory["entries"]:
        if _git(root, "status", "--porcelain", "--untracked-files=all", "--", entry["path"]) or _git(root, "diff", frozen_revision, current_revision, "--", entry["path"]):
            raise ValueError("normative source changed across declaration revisions")
    manifest_path = root / build["selected_path"]
    manifest = validate_manifest(read_regular(manifest_path, METADATA_LIMIT), root)
    declaration = dict(schema="war1-study-declaration-v1", purpose="prospective_frozen_not_blind", identities=build["identities"],
        build=artifact(parent / "build.json"), numeric=artifact(parent / "numeric.json"), resources=artifact(parent / "resources.json"),
        source_inventory=artifact(parent / "source-inventory.json"), manifest=artifact(manifest_path), binary=build["binary"],
        normative_frozen_revision=frozen_revision, declaration_revision=current_revision,
        registered_histories=0, census=POPULATION, budgets=LIMITS)
    with tempfile.TemporaryDirectory(prefix="w1-declare-", dir=Path("/tmp").resolve()) as temp:
        stage = Path(temp)
        provenance = stage / "provenance"
        provenance.mkdir()
        (provenance / "fixture-probes").mkdir()
        for name in ("build", "numeric", "resources", "source_inventory"):
            _copy_descriptor(declaration[name], provenance / (name.replace("_", "-") + ".json"))
        _copy_descriptor(declaration["manifest"], stage / "manifest.json")
        save(stage / "source-inventory.json", inventory)
        _copy_descriptor(numeric["dose_proof"], provenance / "dose-proof.json")
        _copy_descriptor(numeric["numeric_export"], provenance / "numeric-export.json")
        _copy_descriptor(numeric["oracle_packet"], provenance / "oracle-packet.json")
        _copy_descriptor(resources["forecast_inputs"], provenance / "resource-forecast-inputs.json")
        if len(resources["fixture_probes"]) > 32:
            raise ValueError("fixture probe bound")
        for i, probe in enumerate(resources["fixture_probes"]):
            _copy_descriptor(probe["artifact"], provenance / f"fixture-probes/{i:06}.json")
        validate_preparation(stage, manifest, declaration)
        from .preparation import validate_resource_evidence
        validate_resource_evidence(resources, load(provenance / "resource-forecast-inputs.json"),
                                   load(provenance / "fixture-probes/000000.json"), metadata)
    return declaration


def declare(build: Json, numeric: Json, resources: Json, out: Path) -> Json:
    """Validate the exact fixed raw packet, then create its external declaration."""
    return save(out, _validated_declaration(build, numeric, resources, out))
