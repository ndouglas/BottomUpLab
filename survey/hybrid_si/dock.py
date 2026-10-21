#!/usr/bin/env python3
"""The docking reference for milestone 42 (spec S1, amendment 2): copy Si et al.'s published C++
into a scratch directory, patch it minimally, compile it, run every S1 point under sandbox-exec
with the network denied, and keep every run's value.

    python3 -I survey/hybrid_si/dock.py --source papers/alignment/code/si-et-al-2024-jrsi-cooperative-bots-zenodo-13957161 \
        --scratch $SCRATCH/m42/si-dock --out survey/out/hybrid/si-reference [--runs 100] [--jobs 10] [--smoke]

The patches, each required exactly once where it applies: strip stray characters before the first
#include (three files ship with them), make the run count a constant (clang rejects the original
variable-length array initializer) settable as SI_LOOP, set the seed to SI_SEED, and pin the r and
ρ loops to SI_R and SI_RHO. Nothing else changes.

Each compiled program runs under /usr/bin/sandbox-exec. The profile denies the network, file
writes everywhere except the chunk's own box directory (the programs write their summary CSV into
their working directory), forks, Mach lookups, and exec of anything but the box's ./run. The
deposit is never compiled or run in place. Each chunk's result is kept on disk as it finishes, and
rerunning the same command resumes. --smoke runs one run of each file's first point.
"""
import argparse, concurrent.futures, hashlib, json, os, pathlib, re, subprocess, time

FILES = {  # slug: (source file, structure, framework)
    "fig1a": ("For data_Figure1A_Discrete.cpp", "well_mixed", "discrete"),
    "fig1b": ("For data_Figure1B_Continuous.cpp", "well_mixed", "continuous"),
    "fig1c": ("For data_Figure1C_Mixed.cpp", "well_mixed", "mixed"),
    "fig3a": ("For data_Figure3A_Discrete strategy.cpp", "lattice", "discrete"),
    "fig3b": ("For data_Figure3B_Continuous strategy.cpp", "lattice", "continuous"),
    "fig3c": ("For data_Figure3C_Mixed strategy.cpp", "lattice", "mixed"),
}
R = [0.05, 0.1, 0.15]
RHO = {"well_mixed": [0.3], "lattice": [0.0, 0.3, 0.5]}
CHUNKS = 4
# BOX is the run's own directory, passed with `sandbox-exec -D BOX=...`. Beyond the network and
# writes outside BOX, the program may not fork, look up Mach services, or exec anything but BOX/run.
SANDBOX_EXEC = "/usr/bin/sandbox-exec"
SANDBOX = (
    '(version 1)(allow default)(deny network*)(deny file-write* (require-not (subpath (param "BOX"))))'
    '(deny process-fork)(deny mach-lookup)'
    '(deny process-exec)(allow process-exec (literal (string-append (param "BOX") "/run")))'
)
FLAGS = ["-std=c++17", "-O2"]

# (name, pattern, replacement, optional): an optional patch may match zero times; every patch
# that matches must match exactly once.
PATCHES = [
    ("stray characters before #include", r"\A[^#\n]+(#include)", r"\1", True),
    ("constant run count", r"^int loop = 100;", "const int loop = SI_LOOP;", True),
    ("constant run count", r"^#define loop 100\b", "#define loop SI_LOOP", True),
    ("seed", r"sgenrand\(time\(0\)\);", "sgenrand(SI_SEED);", False),
    ("r loop", r"for\s*\(\s*r\s*=\s*0(?:\.0+)?\s*;[^)]*\)", "for (r = SI_R; r <= SI_R; r += 1)", False),
    ("rho loop", r"for\s*\(\s*rho\s*=\s*0(?:\.0+)?\s*;[^)]*\)", "for (rho = SI_RHO; rho <= SI_RHO; rho += 1)", False),
]


def patch(source):
    """The patched source and the list of patches applied; ValueError if a required one is missing."""
    applied = []
    for name, pattern, replacement, optional in PATCHES:
        out, n = re.subn(pattern, replacement, source, flags=re.MULTILINE)
        if n > 1 or (n == 0 and not optional):
            raise ValueError(f"{name}: {n} matches")
        if n == 1:
            applied.append({"name": name, "pattern": pattern, "replacement": replacement})
            source = out
    if not any(a["name"] == "constant run count" for a in applied):
        raise ValueError("constant run count: no match")
    return source, applied


def parse(structure, framework, stdout):
    """(r, ρ, cooperation) for every run line the program printed."""
    rows = []
    for line in stdout.splitlines():
        if structure == "well_mixed":
            f = line.split(",")
            if len(f) == 6 and f[0][:1].isdigit():
                rows.append((float(f[0]), float(f[1]), float(f[5])))
        else:
            f = line.split("\t")
            if len(f) in (4, 5) and f[0][:1].isdigit():
                rows.append((float(f[0]), float(f[1]), float(f[3] if framework == "continuous" else f[2])))
    return rows


def points():
    return [(slug, r, rho) for slug, (_, structure, _) in FILES.items() for r in R for rho in RHO[structure]]


def seeds(p):
    return [CHUNKS * p + k + 1 for k in range(CHUNKS)]


def csv(slug, results):
    """One file's CSV, one row per run, from run_chunk's results; None if the file has no runs."""
    lines = ["r,rho,chunk,seed,run,cooperation"]
    for _, r, rho, chunk, seed, values, _ in sorted(x for x in results if x[0] == slug):
        lines += [f"{r},{rho},{chunk},{seed},{i},{v:.6f}" for i, v in enumerate(values)]
    return "\n".join(lines) + "\n" if len(lines) > 1 else None


def sha(data):
    return hashlib.sha256(data).hexdigest()


def smoke_indices():
    """The index in points() of each file's first point: the smoke runs one run of each."""
    all_points = points()
    return [next(p for p, point in enumerate(all_points) if point[0] == slug) for slug in FILES]


def checked(what, where, argv, **kw):
    """subprocess.run with check=True, but a failure raises with the process's stderr in the message."""
    try:
        return subprocess.run(argv, cwd=where, check=True, capture_output=True, text=True, **kw)
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"{what} failed in {where} (exit {e.returncode}):\n{e.stderr}") from e


def run_chunk(job):
    """Compile and run one chunk in its own directory and keep its result there.

    The chunk directory holds result.json and stdout.txt; the program itself sees only the
    sandbox-writable box/ inside it (its source, binary and the summary CSV it writes), so it cannot
    forge the result a rerun resumes from. A chunk whose result.json matches the job is not rerun."""
    slug, r, rho, seed, per, patched, scratch, chunk = job
    _, structure, framework = FILES[slug]
    home = (scratch / f"{slug}-r{r}-rho{rho}-seed{seed}").resolve()
    box, result = home / "box", home / "result.json"
    key = {"slug": slug, "r": r, "rho": rho, "chunk": chunk, "seed": seed, "runs": per, "patched_sha256": sha(patched.encode("utf-8"))}
    if result.exists():
        kept = json.loads(result.read_text(encoding="utf-8"))
        if {k: kept.get(k) for k in key} == key:
            return slug, r, rho, chunk, seed, kept["values"], kept["seconds"]
    box.mkdir(parents=True, exist_ok=True)
    (box / "x.cpp").write_text(patched, encoding="utf-8")
    defines = [f"-DSI_R={r}", f"-DSI_RHO={rho}", f"-DSI_SEED={seed}", f"-DSI_LOOP={per}"]
    checked("compile", box, ["c++", *FLAGS, *defines, "-o", "run", "x.cpp"])
    started = time.time()
    done = checked("sandboxed run", box, [SANDBOX_EXEC, "-D", f"BOX={box}", "-p", SANDBOX, "./run"])
    seconds = time.time() - started
    (home / "stdout.txt").write_text(done.stdout, encoding="utf-8")
    rows = parse(structure, framework, done.stdout)
    if len(rows) != per or any(abs(a - r) > 1e-6 or abs(b - rho) > 1e-6 for a, b, _ in rows):
        raise RuntimeError(f"{home}: expected {per} runs at r {r}, rho {rho}, got {rows[:3]}…")
    values = [c for _, _, c in rows]
    partial = home / "result.json.partial"
    partial.write_text(json.dumps({**key, "values": values, "seconds": seconds}) + "\n", encoding="utf-8")
    partial.replace(result)
    return slug, r, rho, chunk, seed, values, seconds


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--source", required=True, type=pathlib.Path)
    ap.add_argument("--scratch", required=True, type=pathlib.Path)
    ap.add_argument("--out", required=True, type=pathlib.Path)
    ap.add_argument("--runs", type=int, default=100)
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--smoke", action="store_true", help="one run (chunk 0's seed) of each file's first point")
    a = ap.parse_args()
    assert a.runs % CHUNKS == 0, "runs split into four chunks"
    a.scratch.mkdir(parents=True, exist_ok=True)
    a.out.mkdir(parents=True, exist_ok=True)
    sources, jobs = {}, []
    for slug, (name, _, _) in FILES.items():
        raw = (a.source / name).read_bytes()
        patched, applied = patch(raw.decode("utf-8"))
        sources[slug] = {"file": name, "sha256": sha(raw), "patched_sha256": sha(patched.encode("utf-8")), "patches": applied, "patched": patched}
    all_points = points()
    for p in smoke_indices() if a.smoke else range(len(all_points)):
        slug, r, rho = all_points[p]
        for chunk, seed in enumerate(seeds(p)[:1] if a.smoke else seeds(p)):
            per = 1 if a.smoke else a.runs // CHUNKS
            jobs.append((slug, r, rho, seed, per, sources[slug]["patched"], a.scratch, chunk))
    results, failures, started = [], [], time.time()
    with concurrent.futures.ThreadPoolExecutor(max_workers=a.jobs) as pool:
        for future in concurrent.futures.as_completed([pool.submit(run_chunk, job) for job in jobs]):
            try:
                out = future.result()
            except Exception as e:  # keep the other chunks going; each finished one is on disk
                failures.append(e)
                print("FAILED:", e, flush=True)
                continue
            results.append(out)
            print(out[0], out[1], out[2], "chunk", out[3], f"{out[6]:.1f} s", flush=True)
    if failures:
        raise SystemExit(f"{len(failures)} of {len(jobs)} chunks failed; rerun the same command to resume")
    for slug in FILES:
        text = csv(slug, results)
        if text is not None:
            (a.out / f"{slug}.csv").write_text(text, encoding="utf-8")
    compiler = subprocess.run(["c++", "--version"], capture_output=True, text=True).stdout.splitlines()[0]
    receipts = {
        "sources": {k: {x: v[x] for x in ("file", "sha256", "patched_sha256", "patches")} for k, v in sources.items()},
        "compiler": compiler, "flags": FLAGS, "sandbox_exec": SANDBOX_EXEC, "sandbox": SANDBOX,
        "smoke": "one run of each file's first point" if a.smoke else None, "runs_per_point": 1 if a.smoke else a.runs,
        "wall_seconds": round(time.time() - started, 1),
        "chunks": [{"slug": s, "r": r, "rho": rho, "chunk": c, "seed": seed, "seconds": round(t, 1)} for s, r, rho, c, seed, _, t in sorted(results)],
    }
    (a.out / "receipts.json").write_text(json.dumps(receipts, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
