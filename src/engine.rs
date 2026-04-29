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
}

impl Default for Engine {
    fn default() -> Self {
        Self { debug: false }
    }
}

pub fn boot() -> Engine {
    Engine::default()
}
