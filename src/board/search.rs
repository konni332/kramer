use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use chrono::TimeDelta;
use crossbeam::channel::Sender;
use vampirc_uci::{UciInfoAttribute, UciMessage};

use crate::{
    board::Board,
    move_ordering::next_best,
    moves::{Move, MoveList},
    tt::{TTFlag, TranspositionTable},
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
        tt: &mut TranspositionTable,
    ) -> Option<SearchResult> {
        let nodes = Arc::new(AtomicU64::new(0));
        let start = std::time::Instant::now();
        let mut result = SearchResult {
            best_move: None,
            score: 0,
            depth: 0,
        };

        for depth in 1..=max_depth {
            if stop.load(Ordering::Relaxed) {
                break;
            }

            let search = self.search_root(depth, stop, tt, &nodes);

            if stop.load(Ordering::Relaxed) {
                break;
            }

            let (mv, score) = match search {
                Some(v) => v,
                None => break,
            };

            result.best_move = mv;
            result.score = score;
            result.depth = depth;

            tracing::debug!(
                depth,
                score,
                mv = mv.map(|m| m.to_string()),
                "depth complete"
            );

            let elapsed = start.elapsed();
            let node_count = nodes.load(Ordering::Relaxed);
            let nps = if elapsed.as_secs() > 0 {
                node_count / elapsed.as_secs()
            } else {
                0
            };

            let _ = tx.send(UciMessage::Info(vec![
                UciInfoAttribute::Depth(depth),
                UciInfoAttribute::Nodes(node_count),
                UciInfoAttribute::Time(TimeDelta::milliseconds(elapsed.as_millis() as i64)),
                UciInfoAttribute::Nps(nps),
                UciInfoAttribute::Score {
                    cp: Some(score),
                    mate: None,
                    lower_bound: None,
                    upper_bound: None,
                },
            ]));

            if score.abs() >= MATE_SCORE - 100 {
                break;
            }
        }

        if result.best_move.is_some() {
            Some(result)
        } else {
            None
        }
    }

    pub fn search_root(
        &mut self,
        depth: u8,
        stop: &Arc<AtomicBool>,
        tt: &mut TranspositionTable,
        nodes: &Arc<AtomicU64>,
    ) -> Option<(Option<Move>, i32)> {
        nodes.fetch_add(1, Ordering::Relaxed);
        let mut list = MoveList::new();
        self.generate_legal_moves(&mut list);

        if list.is_empty() {
            return None;
        }

        let hash = self.zobrist;
        let mut alpha = -INF;
        let beta = INF;
        let mut best_move = None;
        let mut best_score = -INF;

        if let Some(tt_move) = tt.probe_move(hash)
            && list
                .as_slice()
                .iter()
                .any(|m| m.from() == tt_move.from() && m.to() == tt_move.to())
        {
            list.move_to_front(tt_move);
        }

        let moves = list.as_mut_slice();
        for i in 0..moves.len() {
            if stop.load(Ordering::Relaxed) {
                return None;
            }

            let mv = next_best(moves, i).unwrap();
            let undo = self.make_move(mv);
            let score = -self.negamax(depth - 1, -beta, -alpha, stop, tt, nodes)?;
            self.unmake_move(mv, undo);

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }

            if score > alpha {
                alpha = score;
            }
        }

        tt.store(hash, depth, best_score, TTFlag::Exact, best_move);

        Some((best_move, best_score))
    }

    pub fn negamax(
        &mut self,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        stop: &Arc<AtomicBool>,
        tt: &mut TranspositionTable,
        nodes: &Arc<AtomicU64>,
    ) -> Option<i32> {
        nodes.fetch_add(1, Ordering::Relaxed);
        if stop.load(Ordering::Relaxed) {
            return None;
        }

        if self.halfmove_clock >= 100 {
            return Some(0);
        }

        if self.is_repitition() {
            return Some(0);
        }

        if depth == 0 {
            return self.quiescence(alpha, beta, stop, tt, nodes);
        }

        let hash = self.zobrist;
        let alpha_orig = alpha;

        if let Some((score, _)) = tt.probe(hash, depth, alpha, beta) {
            return Some(score);
        }

        let mut list = MoveList::new();
        self.generate_legal_moves(&mut list);

        if list.is_empty() {
            return if self.king_in_check(self.side_to_move as usize) {
                Some(-(MATE_SCORE - depth as i32))
            } else {
                Some(0)
            };
        }

        if let Some(tt_move) = tt.probe_move(hash)
            && list
                .as_slice()
                .iter()
                .any(|m| m.from() == tt_move.from() && m.to() == tt_move.to())
        {
            list.move_to_front(tt_move);
        }

        let static_eval = self.evaluate();
        let in_check = self.king_in_check(self.side_to_move as usize);

        if !in_check
            && depth >= 3
            && self.has_non_pawn_material(self.side_to_move as usize)
            && static_eval >= beta
        {
            let r = if depth >= 6 { 3 } else { 2 };
            let null_undo = self.make_null_move();
            let null_score = -self.negamax(depth - 1 - r, -beta, -beta + 1, stop, tt, nodes)?;
            self.unmake_null_move(null_undo);
            if null_score >= beta {
                return Some(beta);
            }
        }

        let mut best = -INF;
        let mut best_move = None;

        let moves = list.as_mut_slice();
        for i in 0..moves.len() {
            if stop.load(Ordering::Relaxed) {
                return None;
            }

            let mv = next_best(moves, i).unwrap();
            let undo = self.make_move(mv);
            let score = -self.negamax(depth - 1, -beta, -alpha, stop, tt, nodes)?;
            self.unmake_move(mv, undo);

            if score > best {
                best = score;
                best_move = Some(mv);
            }

            if score > alpha {
                alpha = score;
            }

            if alpha >= beta {
                tt.store(hash, depth, best, TTFlag::LowerBound, Some(mv));
                return Some(best);
            }
        }

        let flag = if best <= alpha_orig {
            TTFlag::UpperBound
        } else {
            TTFlag::Exact
        };

        tt.store(hash, depth, best, flag, best_move);

        Some(best)
    }

    pub fn quiescence(
        &mut self,
        mut alpha: i32,
        beta: i32,
        stop: &Arc<AtomicBool>,
        tt: &mut TranspositionTable,
        nodes: &Arc<AtomicU64>,
    ) -> Option<i32> {
        nodes.fetch_add(1, Ordering::Relaxed);
        if stop.load(Ordering::Relaxed) {
            return None;
        }

        let stand_pat = self.evaluate();

        if stand_pat >= beta {
            return Some(beta);
        }

        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut list = MoveList::new();
        self.generate_legal_captures(&mut list);

        if let Some(tt_move) = tt.probe_move(self.zobrist) {
            list.move_to_front(tt_move);
        }

        let moves = list.as_mut_slice();
        for i in 0..moves.len() {
            if stop.load(Ordering::Relaxed) {
                return None;
            }

            let mv = next_best(moves, i).unwrap();
            let undo = self.make_move(mv);
            let score = -self.quiescence(-beta, -alpha, stop, tt, nodes)?;
            self.unmake_move(mv, undo);

            if score >= beta {
                return Some(beta);
            }

            if score > alpha {
                alpha = score;
            }
        }

        Some(alpha)
    }
}
