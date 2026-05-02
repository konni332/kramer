use vampirc_uci::{MessageList, UciMessage, UciOptionConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Engine {
    debug: bool,

    threads_option: i64,
    hash_option: i64,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn command(&mut self, cmd: UciMessage) -> MessageList {
        let mut responses = MessageList::new();
        match cmd {
            UciMessage::Uci => {
                responses.push(self.id());
                responses.push(self.author());
                responses.extend(self.options());
                responses.push(UciMessage::UciOk);
            }
            UciMessage::IsReady => {
                responses.push(UciMessage::ReadyOk);
            }
            UciMessage::Debug(debug) => {
                self.debug = debug;
            }
            UciMessage::SetOption { name, value } => {
                self.set_option(&name, &value);
                tracing::info!(?name, ?value, "option set");
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
            } => {}
            UciMessage::Go {
                time_control,
                search_control,
            } => {}
            UciMessage::Stop => {}
            UciMessage::PonderHit => {}
            UciMessage::Quit => {
                tracing::debug!("engine received quit");
            }
            _ => {
                tracing::debug!(?cmd, "engine received command");
            }
        };
        responses
    }

    fn id(&self) -> UciMessage {
        UciMessage::id_name("Kramer")
    }
    fn author(&self) -> UciMessage {
        UciMessage::id_author("konni332")
    }
    fn set_option(&mut self, name: &str, value: &Option<String>) {
        match name {
            "Hash" => self.set_hash_option(value),
            "Threads" => self.set_threads_option(value),
            _ => {
                tracing::error!(?name, "unknown option received");
            }
        }
    }
    fn set_hash_option(&mut self, value: &Option<String>) {
        if let Some(val) = value {
            match val.parse::<i64>() {
                Ok(val) => {
                    if (HASH_MIN..=HASH_MAX).contains(&val) {
                        self.hash_option = val;
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
            self.hash_option = 16;
        }
    }
    fn set_threads_option(&mut self, value: &Option<String>) {
        if let Some(val) = value {
            match val.parse::<i64>() {
                Ok(val) => {
                    if (THREADS_MIN..=THREADS_MAX).contains(&val) {
                        self.threads_option = val;
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
            self.threads_option = 1;
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
            max: Some(THREADS_MAX),
        });

        vec![hash_opt, thread_opt]
    }
}

pub const HASH_DEFAULT: i64 = 16;
pub const HASH_MIN: i64 = 1;
pub const HASH_MAX: i64 = 4096;

pub const THREADS_DEFAULT: i64 = 1;
pub const THREADS_MIN: i64 = 1;
pub const THREADS_MAX: i64 = 64;

impl Default for Engine {
    fn default() -> Self {
        Self {
            debug: false,
            threads_option: THREADS_DEFAULT,
            hash_option: HASH_DEFAULT,
        }
    }
}
