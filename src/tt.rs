use crate::moves::Move;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TTFlag {
    Exact,
    LowerBound, // beta cutoff - score is at least this
    UpperBound, // all moves failed low - score is at most this
}

#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,
    pub depth: u8,
    pub score: i32,
    pub flag: TTFlag,
    pub best_move: Option<Move>,
}

impl TTEntry {
    const EMPTY: Self = Self {
        hash: 0,
        depth: 0,
        score: 0,
        flag: TTFlag::Exact,
        best_move: None,
    };
}

#[derive(Debug)]
pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    size: usize,
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        let bytes = mb * 1024 * 1024;
        let size = bytes / std::mem::size_of::<TTEntry>();
        Self {
            entries: vec![TTEntry::EMPTY; size],
            size,
        }
    }

    #[inline(always)]
    pub fn probe(
        &self,
        hash: u64,
        depth: u8,
        alpha: i32,
        beta: i32,
    ) -> Option<(i32, Option<Move>)> {
        let entry = &self.entries[hash as usize % self.size];

        // collision check
        if entry.hash != hash {
            return None;
        }

        if entry.depth < depth {
            return None;
        }

        let score = match entry.flag {
            TTFlag::Exact => entry.score,
            TTFlag::LowerBound => {
                if entry.score >= beta {
                    entry.score
                } else {
                    return None;
                }
            }
            TTFlag::UpperBound => {
                if entry.score <= alpha {
                    entry.score
                } else {
                    return None;
                }
            }
        };

        Some((score, entry.best_move))
    }

    #[inline(always)]
    pub fn probe_move(&self, hash: u64) -> Option<Move> {
        let entry = &self.entries[hash as usize % self.size];
        if entry.hash != hash {
            return None;
        }
        entry.best_move
    }

    #[inline(always)]
    pub fn store(
        &mut self,
        hash: u64,
        depth: u8,
        score: i32,
        flag: TTFlag,
        best_move: Option<Move>,
    ) {
        let idx = hash as usize % self.size;
        if self.entries[idx].depth < depth {
            self.entries[idx] = TTEntry {
                hash,
                depth,
                score,
                flag,
                best_move,
            }
        }
    }

    pub fn clear(&mut self) {
        self.entries.fill(TTEntry::EMPTY);
    }
}
