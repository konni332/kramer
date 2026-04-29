mod engine;

use std::io::{self, BufRead};

pub use engine::boot;
use vampirc_uci::parse_one;

pub fn run() {
    let mut engine = boot();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => continue,
        };

        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let msg = parse_one(line);
    }
}
