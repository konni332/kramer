use vampirc_uci::{MessageList, UciMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Engine {
    debug: bool,
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
            }
            UciMessage::IsReady => {
                responses.push(UciMessage::ReadyOk);
            }
            _ => {}
        };
        tracing::debug!(?cmd, "engine received command");
        responses
    }

    fn id(&self) -> UciMessage {
        UciMessage::id_name("Kramer")
    }
    fn author(&self) -> UciMessage {
        UciMessage::id_author("konni332")
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self { debug: false }
    }
}
