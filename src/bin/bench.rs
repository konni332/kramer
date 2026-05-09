use crossbeam::channel;
use kramer_core::{Board, tt::TranspositionTable};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
use vampirc_uci::{UciInfoAttribute, UciMessage};

const BENCH_POSITIONS: &[(&str, &str)] = &[
    (
        "startpos",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    (
        "kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ),
    ("endgame", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
    (
        "tactical1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    ),
    (
        "tactical2",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    ),
    (
        "complex",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    ),
    ("pawn_eg", "8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8 b - - 99 50"),
    (
        "promo",
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
    ),
];

const BENCH_DEPTH: u8 = 10;

#[derive(Serialize, Deserialize)]
struct PositionResult {
    name: String,
    fen: String,
    depth: u8,
    nodes: u64,
    time_ms: u64,
    nps: u64,
}

#[derive(Serialize, Deserialize)]
struct BenchResult {
    version: String,
    git_hash: String,
    depth: u8,
    positions: Vec<PositionResult>,
    total_nodes: u64,
    total_time_ms: u64,
    total_nps: u64,
}

fn main() {
    let depth = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(BENCH_DEPTH);

    let output_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "bench_result.json".to_string());

    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string();

    let version = env!("CARGO_PKG_VERSION").to_string();

    let mut positions = Vec::new();
    let mut total_nodes = 0u64;
    let mut total_time_ms = 0u64;

    for (name, fen) in BENCH_POSITIONS {
        let mut board = Board::from_fen(fen).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let mut tt = TranspositionTable::new(64);
        let (tx, rx) = channel::unbounded();

        let start = Instant::now();
        board.iterative_deepening(depth, &stop, tx, &mut tt);
        let time_ms = start.elapsed().as_millis() as u64;

        // extract final node count and nps from the last info message
        let mut node_count = 0u64;
        let mut nps = 0u64;
        while let Ok(msg) = rx.try_recv() {
            if let UciMessage::Info(attrs) = msg {
                for attr in attrs {
                    match attr {
                        UciInfoAttribute::Nodes(n) => node_count = n,
                        UciInfoAttribute::Nps(n) => nps = n,
                        _ => {}
                    }
                }
            }
        }

        total_nodes += node_count;
        total_time_ms += time_ms;

        eprintln!(
            "{}: {} nodes in {}ms ({} nps)",
            name, node_count, time_ms, nps
        );

        positions.push(PositionResult {
            name: name.to_string(),
            fen: fen.to_string(),
            depth,
            nodes: node_count,
            time_ms,
            nps,
        });
    }

    let total_nps = if total_time_ms > 0 {
        total_nodes * 1000 / total_time_ms
    } else {
        0
    };

    let result = BenchResult {
        version,
        git_hash,
        depth,
        positions,
        total_nodes,
        total_time_ms,
        total_nps,
    };

    let json = serde_json::to_string_pretty(&result).unwrap();
    std::fs::write(&output_path, &json).unwrap();
    eprintln!("\nTotal: {} nodes", total_nodes);
    eprintln!("Written to {}", output_path);
}
