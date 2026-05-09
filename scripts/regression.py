#!/usr/bin/env python3


"""
Kramer regression script.
Usage: python3 scripts/regression.py [--depth N] [--games N] [--elo N]
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_ROOT = SCRIPT_DIR.parent
RELEASES_DIR = REPO_ROOT / "releases"
CUTECHESS = "cutechess-cli"
STOCKFISH = "stockfish"

def run(cmd, **kwargs):
    print(f"  $ {' '.join(cmd)}")
    result = subprocess.run(cmd, **kwargs)
    if result.returncode != 0:
        print(f"ERROR: command failed with exit code {result.returncode}")
        sys.exit(1)
    return result

def run_capture(cmd):
    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else None


def build_current():
    print("\n[1/4] Building current version...")
    run(["cargo", "build", "--release", "--bin", "kramer"], cwd=REPO_ROOT)
    run(["cargo", "build", "--release", "--bin", "kramer_bench"], cwd=REPO_ROOT)
    engine = REPO_ROOT / "target" / "release" / "kramer"
    bench = REPO_ROOT / "target" / "release" / "kramer_bench"
    if not engine.exists() or not bench.exists():
        print("ERROR: binaries not found after buidl")
        sys.exit(1)
    return engine, bench

def find_latest_release():
    """Return (version_string, engine_path, bench_result_path) for latest release."""
    if not RELEASES_DIR.exists():
        return None, None, None
    versions = sorted(
            [d for d in RELEASES_DIR.iterdir() if d.is_dir()],
            reverse=True
    )
    for v in versions:
        engine = v / "kramer"
        bench = v / "bench_result.json"
        if engine.exists() and bench.exists():
            return v.name, engine, bench
    return None, None, None


def run_bench(bench_bin, depth, output_path):
    run([str(bench_bin), str(depth), str(output_path)])
    with open(output_path) as f:
        return json.load(f)


def compare_bench(baseline, current):
    print("\n[2/4] Benchmark comparison")

    def pct(old, new):
        return (new - old) / old * 100 if old else 0

    print(f"\n  {'position':<12} {'base nodes':>12} {'curr nodes':>12} {'Δnodes':>8}  {'base nps':>10} {'curr nps':>10} {'Δnps':>8}")
    print("  " + "-" * 78)

    base_by = {p["name"]: p for p in baseline["positions"]}
    curr_by = {p["name"]: p for p in current["positions"]}
    regression = False

    for name, b in base_by.items():
        c = curr_by.get(name)
        if not c:
            print(f"  {name:<12} (missing in current)")
            continue
        nd = pct(b["nodes"], c["nodes"])
        npsd = pct(b["nps"], c["nps"])
        flag = "  <-- REGRESSION" if nd > 5 else ""
        if nd > 5:
            regression = True
        print(f"  {name:<12} {b['nodes']:>12} {c['nodes']:>12} {nd:>+7.1f}%  {b['nps']:>10} {c['nps']:>10} {npsd:>+7.1f}%{flag}")

    print("  " + "-" * 78)
    nd = pct(baseline["total_nodes"], current["total_nodes"])
    npsd = pct(baseline["total_nps"], current["total_nps"])
    print(f"  {'TOTAL':<12} {baseline['total_nodes']:>12} {current['total_nodes']:>12} {nd:>+7.1f}%  {baseline['total_nps']:>10} {current['total_nps']:>10} {npsd:>+7.1f}%")

    return regression, nd



def run_sprt(current_engine, baseline_engine, games, elo_target, concurrency):
    print(f"\n[3/4] SPRT ({games} games vs baseline, concurrency {concurrency})")

    pgn_out = REPO_ROOT / "sprt_result.pgn"

    cmd = [
        CUTECHESS,
        "-engine", f"cmd={current_engine}",  "proto=uci", "name=Current",
        "-engine", f"cmd={baseline_engine}", "proto=uci", "name=Baseline",
        "-each", "tc=5+0.1",
        "-games", str(games),
        "-concurrency", str(concurrency),
        "-sprt", "elo0=0", "elo1=10", "alpha=0.05", "beta=0.05",
        "-pgnout", str(pgn_out),
        "-resign", "movecount=5", "score=900",
        "-draw", "movenumber=40", "movecount=8", "score=10",
    ]

    print(f"  $ {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    output = result.stdout + result.stderr

    # parse score line
    score_line = next((l for l in output.splitlines() if "Score of" in l), None)
    elo_line   = next((l for l in output.splitlines() if "Elo difference" in l), None)
    sprt_line  = next((l for l in output.splitlines() if "SPRT" in l and "llr" in l), None)

    return {
        "score_line": score_line,
        "elo_line":   elo_line,
        "sprt_line":  sprt_line,
        "full_output": output,
        "pgn": str(pgn_out),
    }



def write_report(output_path, data):
    with open(output_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"\n  Report written to {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Kramer regression pipeline")
    parser.add_argument("--depth",       type=int, default=10,  help="bench depth")
    parser.add_argument("--games",       type=int, default=200, help="SPRT game count")
    parser.add_argument("--elo",         type=int, default=1600,help="stockfish elo (fallback if no baseline)")
    parser.add_argument("--concurrency", type=int, default=4,   help="cutechess concurrency")
    parser.add_argument("--no-sprt",     action="store_true",   help="skip SPRT, bench only")
    parser.add_argument("--save-release",action="store_true",   help="save current binary to releases/")
    parser.add_argument("--version",     type=str, default=None,help="version tag for --save-release")
    args = parser.parse_args()

    git_hash = run_capture(["git", "rev-parse", "--short", "HEAD"]) or "unknown"
    print(f"Kramer regression — commit {git_hash}")

    # 1. build
    current_engine, current_bench = build_current()

    # 2. bench current
    current_bench_out = REPO_ROOT / "bench_current.json"
    print("\n  Running current bench...")
    current_data = run_bench(current_bench, args.depth, current_bench_out)

    # 3. find baseline
    baseline_version, baseline_engine, baseline_bench_path = find_latest_release()
    bench_regression = False
    bench_delta = 0.0
    baseline_data = None

    if baseline_engine and baseline_bench_path:
        print(f"\n  Baseline found: {baseline_version}")
        with open(baseline_bench_path) as f:
            baseline_data = json.load(f)
        bench_regression, bench_delta = compare_bench(baseline_data, current_data)
    else:
        print("\n  No baseline release found — skipping bench comparison")
        print(f"  (run with --save-release --version v0_1_0 to save this as baseline)")

    # 4. sprt
    sprt_data = None
    if not args.no_sprt:
        if baseline_engine:
            sprt_data = run_sprt(
                current_engine, baseline_engine,
                args.games, args.elo, args.concurrency
            )
        else:
            print(f"\n[3/4] No baseline binary — running SPRT vs SF{args.elo} instead")
            sprt_data = run_sprt(
                current_engine,
                f"{STOCKFISH} option.UCI_LimitStrength=true option.UCI_Elo={args.elo}",
                args.games, args.elo, args.concurrency
            )

        if sprt_data:
            print(f"\n  {sprt_data['score_line'] or 'no score line found'}")
            print(f"  {sprt_data['elo_line']   or 'no elo line found'}")
            print(f"  {sprt_data['sprt_line']  or 'no sprt line found'}")
    else:
        print("\n[3/4] SPRT skipped (--no-sprt)")

    # 5. optionally save release
    if args.save_release:
        if args.version:
            version = args.version
        else:
            pkgid = run_capture(["cargo", "pkgid"])
            if pkgid is None:
                print("ERROR: could not determine version from cargo pkgid")
                sys.exit(1)
            version = f"v{pkgid.split('#')[-1].replace('.', '_')}"
        release_dir = RELEASES_DIR / version
        release_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy(current_engine, release_dir / "kramer")
        shutil.copy(current_bench_out, release_dir / "bench_result.json")
        print(f"\n[4/4] Saved release to {release_dir}")
    else:
        print("\n[4/4] Not saving release (pass --save-release to save)")

    # 6. write report
    report = {
        "git_hash": git_hash,
        "depth": args.depth,
        "bench_current": current_data,
        "bench_baseline": baseline_data,
        "bench_regression": bench_regression,
        "bench_node_delta_pct": bench_delta,
        "sprt": sprt_data,
    }
    write_report(REPO_ROOT / "regression_report.json", report)

    # 7. exit code
    if bench_regression:
        print("\nFAILED: search regression detected (nodes >5% higher)")
        sys.exit(1)
    else:
        print("\nPASSED")


if __name__ == "__main__":
    main()
