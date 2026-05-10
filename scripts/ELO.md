# Elo Measurement

Kramer uses [Ordo](https://github.com/michiguel/Ordo) for Elo measurement. Ordo computes ratings from a PGN file using a Bayesian approach, giving tighter and more reliable estimates than raw score percentage alone.

## Prerequisites

- `cutechess-cli` installed and on PATH
- `ordo` built from source and on PATH
- `stockfish` installed and on PATH

### Building Ordo

```bash
git clone https://github.com/michiguel/Ordo
cd Ordo && make
sudo cp ordo /usr/local/bin/
```

## Workflow

### 1. Play games against bracketed Stockfish levels

Pick three Stockfish Elo levels that bracket your expected rating and run a separate match for each. Use `tc=10+0.1` for reliable results(shorter time controls add noise).

```bash
cutechess-cli \
  -engine cmd=./target/release/kramer proto=uci name=Kramer \
  -engine cmd=stockfish proto=uci name=SF1600 \
          option.UCI_LimitStrength=true option.UCI_Elo=1600 \
  -each tc=10+0.1 \
  -games 200 -concurrency $(nproc) \
  -pgnout games_1600.pgn

cutechess-cli \
  -engine cmd=./target/release/kramer proto=uci name=Kramer \
  -engine cmd=stockfish proto=uci name=SF1700 \
          option.UCI_LimitStrength=true option.UCI_Elo=1700 \
  -each tc=10+0.1 \
  -games 200 -concurrency $(nproc) \
  -pgnout games_1700.pgn

cutechess-cli \
  -engine cmd=./target/release/kramer proto=uci name=Kramer \
  -engine cmd=stockfish proto=uci name=SF1800 \
          option.UCI_LimitStrength=true option.UCI_Elo=1800 \
  -each tc=10+0.1 \
  -games 200 -concurrency $(nproc) \
  -pgnout games_1800.pgn
```

### 2. Combine the PGNs

```bash
cat games_1600.pgn games_1700.pgn games_1800.pgn > elo_measurement.pgn
```

### 3. Run Ordo

Anchor on the middle Stockfish level, the one closest to your expected rating:

```bash
ordo -Q -D -a 0 -A "SF1700" -n1 -N 100 -p elo_measurement.pgn
```

### 4. Read the results

Ordo outputs ratings relative to the anchor (SF1700 = 0). Add 1700 to get absolute Elo:

PLAYER RATING POINTS PLAYED (%)
SF1800 +100.0 ...
Kramer +44.0 ...
SF1700 0.0 ...
SF1600 -99.0 ...

`Kramer = 0 + 44 = 1744 Elo`

Note: Stockfish's limited strength mode is not perfectly linear, especially above 1700, so SF1800 may not come in at exactly +100. This is expected!

### 5. Record the result

Put the final number in `metadata.json` when running `scripts/release.py`:

```json
{
  "elo": 1744,
  "tc": "10+0.1"
}
```

## Choosing bracket levels

| Situation                              | Suggested levels       |
| -------------------------------------- | ---------------------- |
| First measurement, no idea             | SF1500, SF1700, SF1900 |
| After a major feature (null move, LMR) | current ± 150          |
| Fine-tuning eval                       | current ± 100          |

If you score above 70% against your upper bracket or below 30% against your lower bracket, widen the bracket and rerun.

## Error margins

| Games per level | Elo error margin (approx) |
| --------------- | ------------------------- |
| 100             | ± 50                      |
| 200             | ± 35                      |
| 400             | ± 25                      |
| 800             | ± 18                      |

200 games per level (600 total) is the minimum for a release-quality number. 400 per level is better if time allows.
