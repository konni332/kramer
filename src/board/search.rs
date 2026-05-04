use crate::{
    board::Board,
    moves::{Move, MoveList},
};

pub const INF: i32 = 1_000_000;
pub const MATE_SCORE: i32 = 900_000;

impl Board {
    pub fn negamax(&mut self, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            return self.evaluate();
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
            let score = -self.negamax(depth - 1, -beta, -alpha);
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

    pub fn best_move(&mut self, depth: u8) -> Option<Move> {
        let mut list = MoveList::new();
        self.generate_legal_moves(&mut list);

        if list.len() == 0 {
            return None;
        }

        let mut best_move = None;
        let mut best_score = -INF;
        let alpha = -INF;
        let beta = INF;

        for &mv in list.as_slice() {
            let undo = self.make_move(mv);
            let score = -self.negamax(depth - 1, -beta, -alpha);
            self.unmake_move(mv, undo);

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
        }

        best_move
    }
}
