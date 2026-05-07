use kramer_core::Board;
use std::time::Instant;

fn perft_bench() {
    let positions = [
        (
            "startpos",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            6,
        ),
        (
            "kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            5,
        ),
        ("position3", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 6),
    ];

    println!(
        "{:<12} {:>8} {:>12} {:>12}",
        "position", "depth", "nodes", "mnps"
    );
    println!("{}", "-".repeat(48));

    for (name, fen, depth) in positions {
        let mut board = Board::from_fen(fen).unwrap();
        let start = Instant::now();
        let nodes = board.perft(depth);
        let elapsed = start.elapsed();
        let mnps = nodes as f64 / elapsed.as_secs_f64() / 1_000_000.0;
        println!("{:<12} {:>8} {:>12} {:>12.2}", name, depth, nodes, mnps);
    }
}

fn search_bench() {
    use kramer_core::tt::TranspositionTable;
    use std::sync::{Arc, atomic::AtomicBool};

    let positions = [
        (
            "startpos",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ),
        (
            "kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        ),
        ("endgame", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
    ];

    let depth = 8;
    println!("\n{:<12} {:>8} {:>12}", "position", "depth", "time(ms)");
    println!("{}", "-".repeat(36));

    for (name, fen) in positions {
        let mut board = Board::from_fen(fen).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let mut tt = TranspositionTable::new(64); // 64MB for bench

        let start = Instant::now();
        board.search_root(depth, &stop, &mut tt);
        let elapsed = start.elapsed();

        println!("{:<12} {:>8} {:>12.1}", name, depth, elapsed.as_millis());
    }
}

fn main() {
    println!("=== Perft Benchmark ===");
    perft_bench();

    println!("\n=== Search Benchmark ===");
    search_bench();
}
