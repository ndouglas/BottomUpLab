"""Exact standalone preparation and saved-only analysis command surface."""
import argparse
from pathlib import Path
import sys
import tempfile

from . import registration
from .io import METADATA_LIMIT, _checked_path, create_bytes, read_regular
from .manifest import validate_manifest


def main(argv=None):
    parser = argparse.ArgumentParser(prog="python3 -m survey.war1_study", allow_abbrev=False)
    commands = parser.add_subparsers(dest="command", required=True)
    arguments = {
        "prepare": ("root", "out-dir"),
        "freeze": ("root", "manifest", "out"),
        "build-receipt": ("root", "manifest", "inventory", "binary", "command-file", "out"),
        "declare": ("build", "numeric", "resources", "out"),
        "analyze": ("campaign", "manifest", "oracles", "out"),
    }
    for command, names in arguments.items():
        sub = commands.add_parser(command, allow_abbrev=False)
        for name in names:
            sub.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.command == "prepare":
            print(registration.prepare(args.root, args.out_dir))
        elif args.command == "freeze":
            registration.freeze(args.root, args.manifest, args.out)
        elif args.command == "build-receipt":
            manifest = validate_manifest(read_regular(args.manifest, METADATA_LIMIT), args.root)
            command = registration.load(args.command_file)
            if set(command) != {"command"} or not isinstance(command["command"], list):
                raise ValueError("command file must be exactly {command:[exact argv]}")
            registration.build_receipt(args.root, manifest, registration.load(args.inventory), args.binary, tuple(command["command"]), args.out)
        elif args.command == "declare":
            # Fixed sibling packet preserves raw receipt bytes and descriptors.
            # Existing packets are accepted only when all inputs already match.
            inputs = {name: getattr(args, name) for name in ("build", "numeric", "resources")}
            values = {name: registration.load(path) for name, path in inputs.items()}
            parent = _checked_path(args.out.parent)
            if not parent.exists():
                source_parent = args.build.parent
                copies = dict(inputs)
                copies.update({name: source_parent / name for name in ("source-inventory.json", "freeze-context.json")})
                bounded = {(name if name.endswith(".json") else name + ".json"): read_regular(path, METADATA_LIMIT) for name, path in copies.items()}
                with tempfile.TemporaryDirectory(prefix="w1-declare-cli-", dir=Path("/tmp").resolve()) as temp:
                    stage = Path(temp)
                    for name, raw in bounded.items():
                        create_bytes(stage / name, raw, METADATA_LIMIT)
                    registration._validated_declaration(values["build"], values["numeric"], values["resources"], stage / "declaration.json")
                parent.mkdir(mode=0o700)
                for name, raw in bounded.items():
                    create_bytes(parent / name, raw, METADATA_LIMIT)
            registration.declare(values["build"], values["numeric"], values["resources"], args.out)
        else:
            from .archive import read_campaign
            from .archive_provenance import validate_oracle_packet
            from .analysis import analyze, save_analysis
            root = Path(__file__).resolve().parents[2]
            manifest = validate_manifest(read_regular(args.manifest, METADATA_LIMIT), root)
            archive = read_campaign(args.campaign, manifest)
            export_path = args.campaign / "provenance/numeric-export.json"
            if manifest.purpose == "construction_fixture":
                export_path = args.oracles.parent / "numeric-export.json"
            oracles = validate_oracle_packet(registration.load(args.oracles), manifest, registration.load(export_path))
            save_analysis(args.out, analyze(manifest, archive, oracles), archive.header["binding"])
    except (ValueError, KeyError, TypeError) as error:
        # Shared strict IO preserves its OSError cause while presenting ValueError
        # to bounded readers. Only synchronized destination failures are output.
        if isinstance(error.__cause__, OSError) and str(error).startswith("cannot create synchronized destination "):
            print(f"war1-study output/execution: {error}", file=sys.stderr)
            return 1
        print(f"war1-study input: {error}", file=sys.stderr)
        return 2
    except OSError as error:
        print(f"war1-study output/execution: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
