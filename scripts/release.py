#!/usr/bin/env python3
"""
Kramer release script.
Usage: python3 scripts/release.py [--version X.Y.Z] [--elo N] [--tc TC]
Runs tests, builds, benchmarks via regression.py, bundles a release directory.
"""

import argparse
import json
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path

SCRIPT_DIR   = Path(__file__).parent
REPO_ROOT    = SCRIPT_DIR.parent
RELEASES_DIR = REPO_ROOT / "releases"
REGRESSION   = SCRIPT_DIR / "regression.py"


# ── helpers ───────────────────────────────────────────────────────────────────

def run(cmd: list[str], **kwargs) -> subprocess.CompletedProcess:
    print(f"  $ {' '.join(str(c) for c in cmd)}")
    result = subprocess.run(cmd, **kwargs)
    if result.returncode != 0:
        print(f"\nERROR: command failed (exit {result.returncode})")
        sys.exit(1)
    return result


def run_capture(cmd: list[str]) -> str | None:
    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else None


def prompt(question: str, default: str | None = None) -> str:
    suffix = f" [{default}]" if default is not None else ""
    while True:
        answer = input(f"  {question}{suffix}: ").strip()
        if answer:
            return answer
        if default is not None:
            return default
        print("  (required, please enter a value)")


def prompt_list(question: str) -> list[str]:
    print(f"  {question}")
    print("  (enter one per line, empty line to finish)")
    items = []
    while True:
        line = input("    > ").strip()
        if not line:
            break
        items.append(line)
    return items


def write_json(path: Path, data: dict) -> None:
    with open(path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"  Written: {path}")


def get_cargo_version() -> str | None:
    pkgid = run_capture([
        "cargo", "pkgid",
        "--manifest-path", str(REPO_ROOT / "Cargo.toml"),
    ])
    if pkgid is not None and "#" in pkgid:
        return pkgid.split("#")[-1].strip()
    return None


def find_latest_release() -> tuple[Path, dict] | tuple[None, None]:
    if not RELEASES_DIR.exists():
        return None, None
    versions = sorted(
        [d for d in RELEASES_DIR.iterdir() if d.is_dir()],
        reverse=True,
    )
    for v in versions:
        meta_path  = v / "metadata.json"
        bench_path = v / "bench_result.json"
        if meta_path.exists() and bench_path.exists():
            return v, json.loads(meta_path.read_text())
    return None, None


# ── steps ─────────────────────────────────────────────────────────────────────

def step_tests() -> None:
    print("\n[1/5] Running tests...")
    run(["cargo", "test", "--release"], cwd=REPO_ROOT)
    print("  All tests passed.")


def step_build() -> tuple[Path, Path]:
    print("\n[2/5] Building release binaries...")
    run(["cargo", "build", "--release"], cwd=REPO_ROOT)
    run(["cargo", "build", "--release", "--bin", "kramer_bench"], cwd=REPO_ROOT)

    engine = REPO_ROOT / "target" / "release" / "kramer"
    bench  = REPO_ROOT / "target" / "release" / "kramer_bench"

    if not engine.exists():
        print("ERROR: kramer binary not found after build")
        sys.exit(1)
    if not bench.exists():
        print("ERROR: kramer_bench binary not found after build")
        sys.exit(1)

    print("  Build successful.")
    return engine, bench


def step_regression(depth: int) -> Path:
    """
    Delegate bench + comparison to regression.py.
    Returns path to the bench_current.json it produced.
    """
    print("\n[3/5] Running regression (bench + comparison)...")

    bench_out = REPO_ROOT / "bench_current.json"

    run([
        sys.executable, str(REGRESSION),
        "--depth",   str(depth),
        "--no-sprt",
    ])

    if not bench_out.exists():
        print("ERROR: bench_current.json not produced by regression.py")
        sys.exit(1)

    return bench_out


def step_metadata(
    args: argparse.Namespace,
    git_hash: str,
    version: str,
) -> dict:
    print("\n[4/5] Building metadata...")
    print("  Some values could not be derived — please enter them:\n")

    _, baseline_meta = find_latest_release()
    baseline_features: list[str] = (
        baseline_meta.get("features", []) if baseline_meta is not None else []
    )

    # derived values
    derived = {
        "version": version,
        "commit":  git_hash,
        "date":    date.today().isoformat(),
    }
    for k, v in derived.items():
        print(f"  {k}: {v}  (derived)")

    # prompted values
    elo_input: str = (
        str(args.elo)
        if args.elo is not None
        else prompt("elo (Elo rating for this version)")
    )
    tc: str = (
        args.tc
        if args.tc is not None
        else prompt("tc (time control used for Elo measurement)", default="10+0.1")
    )

    print(f"\n  Features carried from last release:")
    if baseline_features:
        for f in baseline_features:
            print(f"    - {f}")
    else:
        print("    (none — first release)")

    new_features = prompt_list("\n  New features added in this release")
    features = list(dict.fromkeys(baseline_features + new_features))

    notes_input = input("\n  Release notes (optional, one line): ").strip()
    notes: str | None = notes_input if notes_input else None

    metadata: dict = {
        "version":  version,
        "commit":   git_hash,
        "date":     derived["date"],
        "elo":      int(elo_input),
        "tc":       tc,
        "features": features,
    }
    if notes is not None:
        metadata["notes"] = notes

    print(f"\n  Metadata preview:")
    print(json.dumps(metadata, indent=4))
    confirm = input("\n  Looks good? [Y/n]: ").strip().lower()
    if confirm == "n":
        print("  Aborting — edit and rerun.")
        sys.exit(0)

    return metadata


def step_bundle(
    version: str,
    engine: Path,
    bench_out: Path,
    metadata: dict,
) -> Path:
    print("\n[5/5] Bundling release...")

    tag = f"v{version.replace('.', '_')}"
    release_dir = RELEASES_DIR / tag

    if release_dir.exists():
        answer = input(
            f"  {release_dir} already exists. Overwrite? [y/N]: "
        ).strip().lower()
        if answer != "y":
            print("  Aborting.")
            sys.exit(1)
        shutil.rmtree(release_dir)

    release_dir.mkdir(parents=True)

    shutil.copy(engine,    release_dir / "kramer")
    shutil.copy(bench_out, release_dir / "bench_result.json")
    write_json(release_dir / "metadata.json", metadata)

    print(f"\n  Release bundled at: {release_dir}")
    print("  Contents:")
    for f in sorted(release_dir.iterdir()):
        size = f.stat().st_size
        print(f"    {f.name:<25} {size:>10,} bytes")

    return release_dir


# ── main ──────────────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(description="Kramer release bundler")
    parser.add_argument("--version", type=str, default=None,
                        help="override version (default: from Cargo.toml)")
    parser.add_argument("--depth",   type=int, default=10,
                        help="bench depth")
    parser.add_argument("--elo",     type=int, default=None,
                        help="elo rating (skip prompt)")
    parser.add_argument("--tc",      type=str, default=None,
                        help="time control used for elo measurement (skip prompt)")
    args = parser.parse_args()

    git_hash = run_capture(["git", "rev-parse", "--short", "HEAD"]) or "unknown"

    raw_version = args.version or get_cargo_version()
    if raw_version is None:
        print("ERROR: could not determine version, pass --version X.Y.Z")
        sys.exit(1)
    version: str = raw_version

    print(f"Kramer release script — v{version} @ {git_hash}")
    print("=" * 50)

    step_tests()
    engine, _bench_bin = step_build()
    bench_out = step_regression(args.depth)
    metadata  = step_metadata(args, git_hash, version)
    release_dir = step_bundle(version, engine, bench_out, metadata)

    print(f"\n{'=' * 50}")
    print(f"Release v{version} ready at {release_dir}")
    print(f"Upload to GitHub releases as tag v{version}")


if __name__ == "__main__":
    main()
