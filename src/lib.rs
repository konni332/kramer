use crate::engine::Engine;
use std::io::{self, Write};
use std::{sync::mpsc, thread};
use vampirc_uci::UciMessage;

mod board;
mod engine;
pub mod error;
mod fen;
mod logging;
mod moves;
mod uci;

pub fn run() {
    let _guard = logging::init();
    logging::install_panic_hook();

    tracing::info!("kramer boot");

    let (tx, rx) = mpsc::channel::<UciMessage>();

    let engine_thread = thread::spawn(move || {
        let mut engine = Engine::new();

        while let Ok(cmd) = rx.recv() {
            if matches!(cmd, UciMessage::Quit) {
                break;
            }

            let messages = engine.command(cmd);

            let stdout = io::stdout();
            let mut out = stdout.lock();

            for msg in messages {
                writeln!(out, "{msg}").unwrap();
            }
            out.flush().unwrap();
        }
    });

    let uci = thread::spawn(move || {
        uci::run(tx).unwrap();
    });

    uci.join().expect("uci thread panicked");
    engine_thread.join().expect("engine thread panicked");
}
