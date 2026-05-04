use crate::engine::Engine;
use crossbeam::channel::unbounded;
use std::io::{self, Write};
use std::thread;
use vampirc_uci::UciMessage;

mod board;
mod engine;
pub mod error;
mod fen;
mod logging;
mod moves;
mod perft;
mod uci;
mod zobrist;

pub use logging::log_location;

pub fn run_engine() {
    let _guard = logging::init();
    logging::install_panic_hook();

    tracing::info!("kramer boot");

    // command channel (uci -> engine)
    let (cmd_tx, cmd_rx) = unbounded::<UciMessage>();
    // output channel (uci -> stdout)
    let (out_tx, out_rx) = unbounded::<UciMessage>();

    let engine_thread = thread::spawn(move || {
        let mut engine = Engine::new(out_tx);

        while let Ok(cmd) = cmd_rx.recv() {
            if matches!(cmd, UciMessage::Quit) {
                break;
            }

            engine.command(cmd);
        }
    });

    let uci = thread::spawn(move || {
        uci::run(cmd_tx).unwrap();
    });

    let stdout = io::stdout();
    let mut out = stdout.lock();

    while let Ok(msg) = out_rx.recv() {
        writeln!(out, "{msg}").expect("failed to write stdout");
        out.flush().expect("failed to flush stdout");
    }

    uci.join().expect("uci thread panicked");
    engine_thread.join().expect("engine thread panicked");
}
