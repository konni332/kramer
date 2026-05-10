# Scripts

## `scripts/regression.py`

Runs a benchmark and compares it against the latest release baseline. Optionally runs an SPRT test via cutechess.

### Usage

```bash
# bench + comparison only (fast, no games)
python3 scripts/regression.py --no-sprt

# bench + SPRT against latest release binary
python3 scripts/regression.py --games 200

# bench + SPRT vs stockfish (if no baseline release exists)
python3 scripts/regression.py --games 200 --elo 1600

# custom depth and time control
python3 scripts/regression.py --no-sprt --depth 12
python3 scripts/regression.py --games 400 --tc 10+0.1 --concurrency 8
```

### Arguments

| Argument         | Default         | Description                                                                    |
| ---------------- | --------------- | ------------------------------------------------------------------------------ |
| `--depth`        | 10              | Search depth for benchmark                                                     |
| `--games`        | 200             | Number of SPRT games                                                           |
| `--elo`          | 1600            | Stockfish Elo fallback if no baseline binary found                             |
| `--concurrency`  | 4               | Cutechess `-concurrency` value                                                 |
| `--tc`           | `5+0.1`         | Time control for SPRT games                                                    |
| `--no-sprt`      | off             | Skip SPRT, run bench only                                                      |
| `--save-release` | off             | Save binary + bench result to `releases/` (depracated! Use scripts/release.py) |
| `--version`      | from Cargo.toml | Version tag for `--save-release`                                               |

### Output

- `bench_current.json`: benchmark results for current build
- `regression_report.json`: full report including bench comparison and SPRT results
- `sprt_result.pgn`: game PGN if SPRT was run

### Exit codes

- `0`: passed (no regression)
- `1`: failed (node count increased >5% vs baseline, or build/test failure)

---

## `scripts/release.py`

Full release pipeline. Runs tests, builds binaries, runs the regression benchmark, prompts for metadata, and bundles everything into `releases/vX_Y_Z/`.

### Usage

```bash
# fully interactive
python3 scripts/release.py

# pre-fill known values to reduce prompts
python3 scripts/release.py --version 0.2.0 --elo 1850 --tc "10+0.1"
```

### Arguments

| Argument    | Default         | Description                           |
| ----------- | --------------- | ------------------------------------- |
| `--version` | from Cargo.toml | Release version string                |
| `--depth`   | 10              | Bench depth passed to regression.py   |
| `--elo`     | prompted        | Elo rating to record in metadata      |
| `--tc`      | prompted        | Time control used for Elo measurement |

### Steps

1. `cargo test --release`: aborts if any test fails
2. `cargo build --release`: builds `kramer` and `kramer_bench`
3. Runs `regression.py --no-sprt`: benchmarks and compares to baseline, warns on regression
4. Prompts for metadata: Elo, time control, new features, release notes
5. Bundles `releases/vX_Y_Z/` with:
   - `kramer`: engine binary
   - `bench_result.json`: benchmark results
   - `metadata.json`: version, commit, date, Elo, features

### Output layout

 releases/
│ └──  v0_1_0/
│ ├──  kramer
│ ├──  bench_result.json
│ └──  metadata.json

### Notes

- If `releases/vX_Y_Z/` already exists you will be prompted before overwriting
- A node count regression warning (>5%) during bench will prompt you to confirm before continuing
- Elo should be measured separately using the Ordo workflow described in the [ELO](ELO.md) before running this script
