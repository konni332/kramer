use std::io::{self, BufRead};

use crossbeam::channel::Sender;
use vampirc_uci::{UciMessage, parse_one};

pub fn run(tx: Sender<(UciMessage, bool)>) -> io::Result<()> {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let ponder = line.split_whitespace().any(|t| t == "ponder") && line.starts_with("go");
        let msg = parse_one(line);

        let quit = matches!(msg, UciMessage::Quit);

        if tx.send((msg, ponder)).is_err() {
            break;
        }

        if quit {
            break;
        }
    }

    Ok(())
}
