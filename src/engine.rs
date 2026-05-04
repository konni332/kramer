use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use crossbeam::channel::Sender;
use vampirc_uci::{MessageList, UciMessage, UciMove, UciOptionConfig, UciSquare};

use crate::{board::Board, moves::MoveList};

#[derive(Debug)]
pub struct Engine {
    debug: bool,
    options: EngineOptions,

    board: Board,

    stop_flag: Arc<AtomicBool>,
    search_thread: Option<thread::JoinHandle<()>>,
    out_tx: Sender<UciMessage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineOptions {
    threads_option: i64,
    hash_option: i64,
}

impl Engine {
    pub fn new(out_tx: Sender<UciMessage>) -> Self {
        Self {
            debug: false,
            options: EngineOptions::default(),
            board: Board::empty(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            search_thread: None,
            out_tx,
        }
    }
    fn stop_search(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.search_thread.take() {
            handle.join().ok();
        }
    }

    fn send_msg(&self, msg: UciMessage) {
        if let Err(err) = self.out_tx.send(msg) {
            tracing::error!(?err, "channel error sending result");
        }
    }

    pub fn command(&mut self, cmd: UciMessage) {
        match cmd {
            UciMessage::Uci => {
                self.send_msg(self.id());
                self.send_msg(self.author());
                for opt in self.options() {
                    self.send_msg(opt);
                }
                self.send_msg(UciMessage::UciOk);
                self.setup();
            }
            UciMessage::IsReady => {
                self.send_msg(UciMessage::ReadyOk);
            }
            UciMessage::Debug(debug) => {
                self.debug = debug;
            }
            UciMessage::SetOption { name, value } => {
                self.set_option(&name, &value);
            }
            UciMessage::Register { .. } => {
                tracing::info!("registration is not necessary for Kramer");
            }
            UciMessage::UciNewGame => {
                tracing::info!("uci new game");
            }
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
                if startpos {
                    self.board = Board::startpos();
                    tracing::info!("Board set to starting position");
                } else if let Some(fen) = fen {
                    self.board = match Board::from_fen(&fen.0) {
                        Ok(board) => board,
                        Err(err) => {
                            tracing::error!(?err, "Error parsing FEN position");
                            return;
                        }
                    }
                }

                for mv in moves {
                    let mv_str = mv.to_string();
                    let mut list = MoveList::new();
                    self.board.generate_legal_moves(&mut list);
                    let found = list
                        .as_slice()
                        .iter()
                        .find(|m| m.to_string() == mv_str)
                        .copied();
                    match found {
                        Some(m) => {
                            self.board.make_move(m);
                        }
                        None => {
                            tracing::error!(?mv_str, "illegal or unknown move in position command");
                            return;
                        }
                    }
                }
            }
            UciMessage::Go { search_control, .. } => {
                self.stop_search();

                let depth = search_control.as_ref().and_then(|sc| sc.depth).unwrap_or(6);

                self.stop_flag.store(false, Ordering::Relaxed);
                let stop = Arc::clone(&self.stop_flag);
                let tx = self.out_tx.clone();

                // clone board for search thread (Board is Copy)
                let mut board = self.board;

                let handle = thread::spawn(move || {
                    let mv = board.best_move(depth, &stop);
                    let msg = match mv {
                        Some(mv) => UciMessage::best_move(mv.into()),
                        // invalid bestmove output (bestmove 0000)
                        None => UciMessage::BestMove {
                            best_move: invalid_uci_move(),
                            ponder: None,
                        },
                    };

                    if let Err(err) = tx.send(msg) {
                        tracing::error!(?err, "channel error sending search result");
                    }
                });

                self.search_thread = Some(handle);
            }
            UciMessage::Stop => {
                self.stop_search();
                tracing::info!("search stopped");
            }
            UciMessage::PonderHit => {
                tracing::info!("received ponderhit command(noop)");
            }
            UciMessage::Quit => {
                tracing::debug!("engine received quit");
            }
            _ => {
                tracing::debug!(?cmd, "engine received command");
            }
        };
    }

    fn id(&self) -> UciMessage {
        UciMessage::id_name("Kramer")
    }
    fn author(&self) -> UciMessage {
        UciMessage::id_author("konni332")
    }
    fn set_option(&mut self, name: &str, value: &Option<String>) {
        match name {
            "Hash" => self.set_hash(value),
            "Threads" => self.set_threads(value),
            _ => {
                tracing::error!(?name, "unknown option received");
            }
        }
    }
    fn set_hash(&mut self, value: &Option<String>) {
        if let Some(val) = value {
            match val.parse::<i64>() {
                Ok(val) => {
                    if (HASH_MIN..=HASH_MAX).contains(&val) {
                        self.options.hash_option = val;
                        tracing::info!(?val, "option set: Hash");
                    } else {
                        tracing::error!(?val, "value out of bounds for option: Hash");
                    }
                }
                Err(err) => {
                    tracing::error!(?err, ?val, "failed to parse value for option: Hash");
                }
            }
        } else {
            self.options.hash_option = 16;
            tracing::info!("option set: Hash default");
        }
    }
    fn set_threads(&mut self, value: &Option<String>) {
        if let Some(val) = value {
            match val.parse::<i64>() {
                Ok(val) => {
                    if (THREADS_MIN
                        ..=std::thread::available_parallelism()
                            .map(|val| val.get() as i64)
                            .unwrap_or(64i64))
                        .contains(&val)
                    {
                        self.options.threads_option = val;
                        tracing::info!(?val, "option set: Threads");
                    } else {
                        tracing::error!(?val, "value out of bounds for option: Threads");
                    }
                }
                Err(err) => {
                    tracing::error!(?err, ?val, "failed to parse value for option: Threads");
                }
            }
        } else {
            self.options.threads_option = 1;
            tracing::info!("option set: Threads default");
        }
    }
    fn options(&self) -> MessageList {
        let hash_opt = UciMessage::Option(UciOptionConfig::Spin {
            name: "Hash".into(),
            default: Some(HASH_DEFAULT),
            min: Some(HASH_MIN),
            max: Some(HASH_MAX),
        });

        let thread_opt = UciMessage::Option(UciOptionConfig::Spin {
            name: "Threads".into(),
            default: Some(THREADS_DEFAULT),
            min: Some(THREADS_MIN),
            max: Some(
                std::thread::available_parallelism()
                    .map(|val| val.get() as i64)
                    .unwrap_or(64i64),
            ),
        });

        vec![hash_opt, thread_opt]
    }

    pub fn setup(&mut self) {
        self.board = Board::startpos();
    }
}

pub const HASH_DEFAULT: i64 = 16;
pub const HASH_MIN: i64 = 1;
pub const HASH_MAX: i64 = 4096;

pub const THREADS_DEFAULT: i64 = 1;
pub const THREADS_MIN: i64 = 1;

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            threads_option: THREADS_DEFAULT,
            hash_option: HASH_DEFAULT,
        }
    }
}

fn invalid_uci_move() -> UciMove {
    UciMove {
        from: UciSquare::default(),
        to: UciSquare::default(),
        promotion: None,
    }
}
