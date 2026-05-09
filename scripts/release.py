#!/usr/bin/env python3
"""
Kramer release script.
Usage: python3 scripts/release.py [--version X.Y.Z] [--elo N] [--tc TC]
Runs tests, builds, benchmarks, bundles a release directory.
"""

import argparse
import json
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT   = SCRIPT_DIR.parent
RELEASES_DIR = REPO_ROOT / "releases"


# ── helpers ───────────────────────────────────────────────────────────────────

def run(cmd, **kwargs):
    print(f"  $ {' '.join(str(c) for c in cmd)}")
    result = subprocess.run(cmd, **kwargs)
    if result.returncode != 0:
        print(f"\nERROR: command failed (exit {result.returncode})")
        sys.exit(1)
    return result


def run_capture(cmd):
    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else None


def prompt(question, default=None):
    suffix = f" [{default}]" if default is not None else ""
    while True:
        answer = input(f"  {question}{suffix}: ").strip()
        if answer:
            return answer
        if default is not None:
            return default
        print("  (required, please enter a value)")


def prompt_list(question):
    print(f"  {question}")
    print("  (enter one per line, empty line to finish)")
    items = []
    while True:
        line = input("    > ").strip()
        if not line:
            break
        items.append(line)
    return items


def write_json(path, data):
    with open(path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"  Written: {path}")


def get_cargo_version():
    pkgid = run_capture(["cargo", "pkgid", "--manifest-path",
                         str(REPO_ROOT / "Cargo.toml")])
    if pkgid and "#" in pkgid:
        return pkgid.split("#")[-1].strip()
    return None


def find_latest_release():
    if not RELEASES_DIR.exists():
        return None, None
    versions = sorted([d for d in RELEASES_DIR.iterdir() if d.is_dir()], reverse=True)
    for v in versions:
        meta = v / "metadata.json"
        bench = v / "bench_result.json"
        if meta.exists() and bench.exists():
            return v, json.loads(meta.read_text())
    return None, None


# ── steps ─────────────────────────────────────────────────────────────────────

def step_tests():
    print("\n[1/5] Running tests...")
    run(["cargo", "test", "--release"], cwd=REPO_ROOT)
    print("  All tests passed.")


def step_build():
    print("\n[2/5] Building release binaries...")
    run(["cargo", "build", "--release"], cwd=REPO_ROOT)
    run(["cargo", "build", "--release", "--bin", "bench"], cwd=REPO_ROOT)
    engine = REPO_ROOT / "target" / "release" / "kramer"
    bench  = REPO_ROOT / "target" / "release" / "bench"
    if not engine.exists() or not bench.exists():
        print("ERROR: binaries not found after build")
        sys.exit(1)
    print("  Build successful.")
    return engine, bench


def step_bench(bench_bin, depth):
    print(f"\n[3/5] Running benchmark (depth {depth})...")
    out = REPO_ROOT / "bench_current.json"
    run([str(bench_bin), str(depth), str(out)])
    with open(out) as f:
        data = json.load(f)
    print(f"  Total nodes : {data['total_nodes']:,}")
    print(f"  Total NPS   : {data['total_nps']:,}")

    baseline_dir, baseline_meta = find_latest_release()
    if baseline_dir:
        baseline_bench = baseline_dir / "bench_result.json"
        with open(baseline_bench) as f:
            baseline = json.load(f)
        delta = (data["total_nodes"] - baseline["total_nodes"]) / baseline["total_nodes"] * 100
        print(f"  Node delta vs {baseline_meta['version']}: {delta:+.1f}%")
        if delta > 5:
            print(f"\n  WARNING: node count increased >5% — possible search regression")
            answer = input("  Continue anyway? [y/N]: ").strip().lower()
            if answer != "y":
                print("  Aborting.")
                sys.exit(1)
    else:
        print("  No baseline found, skipping comparison.")

    return data, out


def step_metadata(args, git_hash, version):
    print("\n[4/5] Building metadata...")
    print("  Some values could not be derived — please enter them:\n")

    _, baseline_meta = find_latest_release()
    baseline_features = baseline_meta.get("features", []) if baseline_meta else []

    # values we can derive
    derived = {
        "version": version,
        "commit":  git_hash,
        "date":    date.today().isoformat(),
    }
    for k, v in derived.items():
        print(f"  {k}: {v}  (derived)")

    # values we prompt for
    elo = args.elo or prompt("base_elo (Elo rating for this version)", default=None)
    tc  = args.tc  or prompt("tc (time control used for Elo test)", default="5+0.1")

    print(f"\n  Existing features from last release:")
    for f in baseline_features:
        print(f"    - {f}")
    new_features = prompt_list("\n  New features added in this release (added on top of above)")
    features = list(dict.fromkeys(baseline_features + new_features))

    notes = input("\n  Release notes (optional, one line): ").strip() or None

    metadata = {
        "version":  version,
        "commit":   git_hash,
        "date":     derived["date"],
        "elo":      int(elo),
        "tc":       tc,
        "features": features,
    }
    if notes:
        metadata["notes"] = notes

    print(f"\n  Metadata preview:")
    print(json.dumps(metadata, indent=4))
    confirm = input("\n  Looks good? [Y/n]: ").strip().lower()
    if confirm == "n":
        print("  Aborting — edit and rerun.")
        sys.exit(0)

    return metadata


def step_bundle(version, engine, bench_out, metadata):
    print("\n[5/5] Bundling release...")

    tag = f"v{version.replace('.', '_')}"
    release_dir = RELEASES_DIR / tag
    if release_dir.exists():
        answer = input(f"  {release_dir} already exists. Overwrite? [y/N]: ").strip().lower()
        if answer != "y":
            print("  Aborting.")
            sys.exit(1)
        shutil.rmtree(release_dir)

    release_dir.mkdir(parents=True)

    shutil.copy(engine,    release_dir / "kramer")
    shutil.copy(bench_out, release_dir / "bench_result.json")
    write_json(release_dir / "metadata.json", metadata)

    print(f"\n  Release bundled at: {release_dir}")
    print(f"  Contents:")
    for f in sorted(release_dir.iterdir()):
        size = f.stat().st_size
        print(f"    {f.name:<25} {size:>10,} bytes")

    return release_dir


# ── main ──────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Kramer release bundler")
    parser.add_argument("--version", type=str, default=None, help="override version (default: from Cargo.toml)")
    parser.add_argument("--depth",   type=int, default=10,   help="bench depth")
    parser.add_argument("--elo",     type=int, default=None, help="base elo (skip prompt)")
    parser.add_argument("--tc",      type=str, default=None, help="time control (skip prompt)")
    args = parser.parse_args()

    git_hash = run_capture(["git", "rev-parse", "--short", "HEAD"]) or "unknown"
    version  = args.version or get_cargo_version()
    if not version:
        print("ERROR: could not determine version, pass --version X.Y.Z")
        sys.exit(1)

    print(f"Kramer release script — v{version} @ {git_hash}")
    print("=" * 50)

    step_tests()
    engine, bench_bin = step_build()
    bench_data, bench_out = step_bench(bench_bin, args.depth)
    metadata = step_metadata(args, git_hash, version)
    release_dir = step_bundle(version, engine, bench_out, metadata)

    print(f"\n{'=' * 50}")
    print(f"Release v{version} ready at {release_dir}")
    print(f"Upload to GitHub releases as tag v{version}")


if __name__ == "__main__":
    main()
