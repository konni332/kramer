use std::{
    io::{self, BufRead},
    sync::mpsc::Sender,
};

use vampirc_uci::{UciMessage, parse_one};

pub fn run(tx: Sender<UciMessage>) -> io::Result<()> {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let cmd = parse_one(line);

        let quit = matches!(cmd, UciMessage::Quit);

        if tx.send(cmd).is_err() {
            break;
        }

        if quit {
            break;
        }
    }

    Ok(())
}
