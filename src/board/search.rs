use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crossbeam::channel::Sender;
use vampirc_uci::{UciInfoAttribute, UciMessage};

use crate::{
    board::Board,
    moves::{Move, MoveList},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u8,
}

pub const INF: i32 = 1_000_000;
pub const MATE_SCORE: i32 = 900_000;

impl Board {
    pub fn iterative_deepening(
        &mut self,
        max_depth: u8,
        stop: &Arc<AtomicBool>,
        tx: Sender<UciMessage>,
    ) -> SearchResult {
        let mut result = SearchResult {
            best_move: None,
            score: 0,
            depth: 0,
        };

        for depth in 1..=max_depth {
            if stop.load(Ordering::Relaxed) {
                break;
            }

            let (mv, score) = self.search_root(depth, stop);

            // stopped mid search
            // reslut is unreliable
            if stop.load(Ordering::Relaxed) {
                break;
            }

            result.best_move = mv;
            result.score = score;
            result.depth = depth;

            tracing::debug!(
                depth,
                score,
                mv = mv.map(|m| m.to_string()),
                "depth complete"
            );

            let info = UciMessage::Info(vec![
                UciInfoAttribute::Depth(depth),
                UciInfoAttribute::Score {
                    cp: Some(score),
                    mate: None,
                    lower_bound: None,
                    upper_bound: None,
                },
            ]);
            if let Err(err) = tx.send(info) {
                tracing::error!(?err, "channel error sending search info");
            }

            // stop early if mate found
            if score.abs() >= MATE_SCORE - 100 {
                break;
            }
        }

        result
    }

    pub fn search_root(&mut self, depth: u8, stop: &Arc<AtomicBool>) -> (Option<Move>, i32) {
        let mut list = MoveList::new();
        self.generate_legal_moves(&mut list);

        if list.len() == 0 {
            return (None, 0);
        }

        let mut best_move = None;
        let mut best_score = -INF;
        let mut alpha = -INF;
        let beta = INF;

        for &mv in list.as_slice() {
            if stop.load(Ordering::Relaxed) {
                break;
            }

            let undo = self.make_move(mv);
            let score = -self.negamax(depth - 1, -beta, -alpha, stop);
            self.unmake_move(mv, undo);

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            if score > alpha {
                alpha = score;
            }
        }

        (best_move, best_score)
    }
    pub fn negamax(&mut self, depth: u8, mut alpha: i32, beta: i32, stop: &Arc<AtomicBool>) -> i32 {
        if stop.load(Ordering::Relaxed) {
            return 0;
        }

        if depth == 0 {
            return self.quiescence(alpha, beta, stop);
        }

        let mut list = MoveList::new();
        self.generate_legal_moves(&mut list);

        if list.len() == 0 {
            if self.king_in_check(self.side_to_move as usize) {
                // checkmate
                // return mate score, subtract depth to prefer faster mates
                return -(MATE_SCORE - depth as i32);
            } else {
                // stalemate
                return 0;
            }
        }

        let mut best = -INF;

        for &mv in list.as_slice() {
            let undo = self.make_move(mv);
            let score = -self.negamax(depth - 1, -beta, -alpha, stop);
            self.unmake_move(mv, undo);

            if score > best {
                best = score;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break; // beta cutoff
            }
        }

        best
    }
    pub fn quiescence(&mut self, mut alpha: i32, beta: i32, stop: &Arc<AtomicBool>) -> i32 {
        if stop.load(Ordering::Relaxed) {
            return 0;
        }

        let stand_pat = self.evaluate();

        if stand_pat >= beta {
            return beta; // beta cutoff
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // generate only captures
        let mut list = MoveList::new();
        self.generate_capture_moves(&mut list);

        for &mv in list.as_slice() {
            if !mv.is_capture() {
                continue;
            }

            if stop.load(Ordering::Relaxed) {
                return 0;
            }

            let undo = self.make_move(mv);
            let score = self.quiescence(-beta, -alpha, stop);
            self.unmake_move(mv, undo);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    pub fn generate_capture_moves(&self, list: &mut MoveList) {
        self.generate_pawn_captures(list);
        self.generate_knight_captures(list);
        self.generate_bishop_captures(list);
        self.generate_rook_captures(list);
        self.generate_queen_captures(list);
        self.generate_king_captures(list);
    }
}
