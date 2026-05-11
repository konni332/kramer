# Kramer

A UCI chess engine written in Rust, currently rated approximately 2223 Elo (CCRL scale, tc=10+0.1, tested against elo limited stockfish 18).

## Building

Requires Rust and Cargo.

```bash
cargo build --release
```

The engine binary will be at `target/release/kramer`.

## Usage

Kramer communicates via the UCI protocol. Connect it to any UCI-compatible chess GUI such as Arena, CuteChess, or Banksia.

```bash
# run directly for manual UCI input
./target/release/kramer
```

### UCI Options

| Option  | Default | Range  | Description                                                 |
| ------- | ------- | ------ | ----------------------------------------------------------- |
| Hash    | 16      | 1-4096 | Transposition table size in MB                              |
| Threads | 1       | 1-N    | Number of search threads (currently single-threaded search) |

## Features

### Board Representation

- Bitboard-based board representation
- Magic bitboards for sliding piece attack generation
- Zobrist hashing for position identification

### Move Generation

- Legal move generation via pseudo-legal generation + legality filtering
- Full support for castling, en passant, and promotions
- Perft-verified correct move generation

### Search

- Negamax with alpha-beta pruning
- Iterative deepening
- Quiescence search
- Transposition table with four flag types (exact, lower bound, upper bound, move-only)
- Null move pruning with zugzwang guard (material threshold + static eval)
- Killer move heuristic (2 killers per depth)
- Repetition detection via Zobrist hash history
- Fifty-move rule

### Move Ordering

- TT move first
- MVV-LVA for captures
- Killer moves

### Evaluation

- Tapered evaluation interpolating between middlegame and endgame scores
- PeSTO piece-square tables (material baked in)
- Passed pawn detection with rank-scaled bonus
- Bishop pair bonus
- Phase-based game stage detection

### Time Management

- Soft time limit based on remaining time and increment
- Hard time check inside iterative deepening loop
- Generation counter to prevent stale timer threads from firing into subsequent searches

## Development

See [SCRIPTS.md](./scripts/SCRIPTS.md) for documentation on the regression and release scripts.

See [ELO.md](./scripts/ELO.md) for the Elo measurement workflow using Ordo and reference engines.

### Running Tests

```bash
cargo test
```

### Running the Benchmark

```bash
cargo run --release --bin kramer_bench -- 10 bench_result.json
```

### Release Process

```bash
python3 scripts/release.py --version X.Y.Z --elo NNNN --tc "10+0.1"
```

### Regression Testing

```bash
# bench only
python3 scripts/regression.py --no-sprt

# bench + SPRT against latest release
python3 scripts/regression.py --games 200
```

## Release History

| Version | Elo  | Features added                                                           |
| ------- | ---- | ------------------------------------------------------------------------ |
| v0.1.0  | 1744 | Baseline: alpha-beta, iterative deepening, quiescence, TT, MVV-LVA       |
| v0.2.0  | 1786 | Null move pruning                                                        |
| v0.3.0  | 1856 | PeSTO tapered evaluation                                                 |
| v0.4.0  | 2223 | Passed pawns, bishop pair, killer moves, repetition detection, bug fixes |

## Acknowledgements

- PeSTO piece-square tables by Ronald Friederich, values from Rofchade
- [vampirc-uci](https://github.com/vampirc/vampirc-uci) for UCI message parsing
