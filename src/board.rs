use crate::{
    board::generation::CASTLING_RIGHTS_MASK,
    engine::MAX_DEPTH,
    moves::{FLAG_CAPTURE, Move, MoveList},
    zobrist::{ZOBRIST_CASTLING, ZOBRIST_EP, ZOBRIST_PIECE, ZOBRIST_SIDE},
};

mod eval;
mod generation;
mod king;
mod knight;
mod legality_filter;
mod pawns;
mod search;
mod sliding_pieces;

pub type Bitboard = u64;
pub type Square = u8;

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
pub const BOTH: usize = 2;

pub const EMPTY: u8 = 0;

pub const WP: u8 = 1;
pub const WN: u8 = 2;
pub const WB: u8 = 3;
pub const WR: u8 = 4;
pub const WQ: u8 = 5;
pub const WK: u8 = 6;

pub const BP: u8 = 7;
pub const BN: u8 = 8;
pub const BB: u8 = 9;
pub const BR: u8 = 10;
pub const BQ: u8 = 11;
pub const BK: u8 = 12;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub pieces: [Bitboard; 12],

    pub occ: [Bitboard; 3],

    pub mailbox: [u8; 64],

    pub side_to_move: u8, // 0 white, 1 black
    pub castling: u8,     // 4 bits: WK WQ BK BQ
    pub ep_square: u8,    // 64 = none

    pub halfmove_clock: u16,
    pub fullmove_number: u16,

    pub zobrist: u64,

    pub material_mg: i32,
    pub material_eg: i32,

    pub pst_mg: i32,
    pub pst_eg: i32,

    pub history: Vec<u64>,

    pub killers: [[Option<Move>; 2]; (MAX_DEPTH + 1) as usize], // max depth 99 + 1 just to be safe :), 2 moves per depth
}

impl Board {
    pub fn empty() -> Self {
        Self {
            pieces: [0; 12],
            occ: [0; 3],
            mailbox: [EMPTY; 64],

            side_to_move: WHITE as u8,
            castling: 0,
            ep_square: 64,

            halfmove_clock: 0,
            fullmove_number: 1,

            zobrist: 0,

            material_mg: 0,
            material_eg: 0,

            pst_mg: 0,
            pst_eg: 0,

            history: Vec::with_capacity(10),
            killers: [[None; 2]; (MAX_DEPTH + 1) as usize],
        }
    }

    pub fn startpos() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Start position failed to be parsed")
    }

    pub fn generate_all_moves(&self, list: &mut MoveList) {
        self.generate_pawn_moves(list);
        self.generate_knight_moves(list);
        self.generate_king_moves(list);
        self.generate_bishop_moves(list);
        self.generate_rook_moves(list);
        self.generate_queen_moves(list);
    }

    #[inline(always)]
    pub fn put_piece(&mut self, piece: u8, sq: Square) {
        debug_assert!((WP..=BK).contains(&piece));
        debug_assert!(sq < 64);
        debug_assert!(self.mailbox[sq as usize] == EMPTY);

        let idx = sq as usize;
        let bit = 1u64 << sq;

        self.mailbox[idx] = piece;

        self.pieces[(piece - 1) as usize] |= bit;

        if piece <= WK {
            self.occ[WHITE] |= bit;
        } else {
            self.occ[BLACK] |= bit;
        }

        self.occ[BOTH] |= bit;
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, sq: Square) -> u8 {
        debug_assert!(sq < 64);
        debug_assert!(self.mailbox[sq as usize] != EMPTY);

        let idx = sq as usize;
        let bit = !(1u64 << sq);

        let piece = self.mailbox[idx];
        self.mailbox[idx] = EMPTY;

        self.pieces[(piece - 1) as usize] &= bit;

        if piece <= WK {
            self.occ[WHITE] &= bit;
        } else {
            self.occ[BLACK] &= bit;
        }

        self.occ[BOTH] &= bit;

        piece
    }

    #[inline(always)]
    pub fn move_piece(&mut self, from: Square, to: Square) {
        debug_assert!(from < 64 && to < 64);

        let from_idx = from as usize;
        let to_idx = to as usize;

        let piece = self.mailbox[from_idx];

        debug_assert!(piece != EMPTY);
        debug_assert!(self.mailbox[to_idx] == EMPTY);

        let from_bit = 1u64 << from;
        let to_bit = 1u64 << to;
        let delta = from_bit | to_bit;

        self.mailbox[from_idx] = EMPTY;
        self.mailbox[to_idx] = piece;

        self.pieces[(piece - 1) as usize] ^= delta;

        if piece <= WK {
            self.occ[WHITE] ^= delta;
        } else {
            self.occ[BLACK] ^= delta;
        }

        self.occ[BOTH] ^= delta;
    }

    fn push_quiets(&self, from: u8, mut bb: Bitboard, piece: u8, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;
            list.push(Move::new(from, to, piece, 0, 0, 0));
        }
    }

    fn push_caps(&self, from: u8, mut bb: Bitboard, piece: u8, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;
            let captured = self.mailbox[to as usize];
            list.push(Move::new(from, to, piece, captured, 0, FLAG_CAPTURE));
        }
    }

    pub fn make_move(&mut self, mv: Move) -> Undo {
        let undo = Undo {
            castling: self.castling,
            ep_square: self.ep_square,
            halfmove_clock: self.halfmove_clock,
            zobrist: self.zobrist,
        };

        let from = mv.from();
        let to = mv.to();
        let piece = mv.piece();

        // update halfmove clock
        if piece == WP || piece == BP || mv.is_capture() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        self.zobrist ^= ZOBRIST_CASTLING[self.castling as usize];
        if self.ep_square != 64 {
            self.zobrist ^= ZOBRIST_EP[(self.ep_square % 8) as usize];
        }

        if mv.is_ep() {
            let captured_sq = if piece == WP { to - 8 } else { to + 8 };
            self.zobrist ^= ZOBRIST_PIECE[piece as usize][from as usize];
            self.zobrist ^= ZOBRIST_PIECE[piece as usize][to as usize];
            self.zobrist ^= ZOBRIST_PIECE[mv.captured() as usize][captured_sq as usize];
            self.move_piece(from, to);
            self.remove_piece(captured_sq);
        } else if mv.is_castle() {
            let (rook_from, rook_to) = match to {
                6 => (7, 5),    // WK-side: h1 -> f1
                2 => (0, 3),    // WQ-side: a1 -> d1
                62 => (63, 61), // BK-side: h8 -> f8
                58 => (56, 59), // BQ-side: a8 -> d8
                _ => unreachable!("invalid castle to square"),
            };

            let rook = if piece == WK { WR } else { BR };
            self.zobrist ^= ZOBRIST_PIECE[piece as usize][from as usize];
            self.zobrist ^= ZOBRIST_PIECE[piece as usize][to as usize];
            self.zobrist ^= ZOBRIST_PIECE[rook as usize][rook_from as usize];
            self.zobrist ^= ZOBRIST_PIECE[rook as usize][rook_to as usize];
            self.move_piece(from, to);
            self.move_piece(rook_from, rook_to);
        } else if mv.is_promotion() {
            self.zobrist ^= ZOBRIST_PIECE[piece as usize][from as usize];
            // remove pawn, place promoted piece
            if mv.is_capture() {
                self.zobrist ^= ZOBRIST_PIECE[mv.captured() as usize][to as usize];
                self.remove_piece(to);
            }
            self.zobrist ^= ZOBRIST_PIECE[mv.promo() as usize][to as usize];
            self.remove_piece(from);
            self.put_piece(mv.promo(), to);
        } else {
            self.zobrist ^= ZOBRIST_PIECE[piece as usize][from as usize];
            self.zobrist ^= ZOBRIST_PIECE[piece as usize][to as usize];
            // normal or capture move
            if mv.is_capture() {
                self.zobrist ^= ZOBRIST_PIECE[mv.captured() as usize][to as usize];
                self.remove_piece(to);
            }
            self.move_piece(from, to);
        }

        self.ep_square = 64;
        if mv.is_double_push() {
            self.ep_square = if piece == WP { to - 8 } else { to + 8 };
        }

        self.castling &= CASTLING_RIGHTS_MASK[from as usize];
        self.castling &= CASTLING_RIGHTS_MASK[to as usize];

        self.zobrist ^= ZOBRIST_CASTLING[self.castling as usize];
        if self.ep_square != 64 {
            self.zobrist ^= ZOBRIST_EP[(self.ep_square % 8) as usize];
        }

        self.zobrist ^= ZOBRIST_SIDE;
        self.side_to_move ^= 1;

        if self.side_to_move == WHITE as u8 {
            self.fullmove_number += 1;
        }

        self.history.push(self.zobrist);

        undo
    }
    pub fn unmake_move(&mut self, mv: Move, undo: Undo) {
        self.history.pop();
        self.side_to_move ^= 1;

        let from = mv.from();
        let to = mv.to();
        let piece = mv.piece();

        if mv.is_ep() {
            self.move_piece(to, from);
            let captured_sq = if piece == WP { to - 8 } else { to + 8 };
            self.put_piece(mv.captured(), captured_sq);
        } else if mv.is_castle() {
            self.move_piece(to, from);
            let (rook_from, rook_to) = match to {
                6 => (7, 5),
                2 => (0, 3),
                62 => (63, 61),
                58 => (56, 59),
                _ => unreachable!("invalid castle to square"),
            };
            // rook_to is where the rook currently is, rook_from is where it came from
            self.move_piece(rook_to, rook_from);
        } else if mv.is_promotion() {
            self.remove_piece(to);
            self.put_piece(piece, from);
            if mv.is_capture() {
                self.put_piece(mv.captured(), to);
            }
        } else {
            self.move_piece(to, from);
            if mv.is_capture() {
                self.put_piece(mv.captured(), to);
            }
        }

        self.castling = undo.castling;
        self.ep_square = undo.ep_square;
        self.halfmove_clock = undo.halfmove_clock;
        self.zobrist = undo.zobrist;

        if self.side_to_move == BLACK as u8 {
            self.fullmove_number -= 1;
        }
    }

    pub fn make_null_move(&mut self) -> NullUndo {
        let undo = NullUndo {
            ep_square: self.ep_square,
            zobrist: self.zobrist,
        };

        if self.ep_square != 64 {
            self.zobrist ^= ZOBRIST_EP[(self.ep_square % 8) as usize];
        }
        self.zobrist ^= ZOBRIST_SIDE;
        self.side_to_move ^= 1;

        self.ep_square = 64;

        self.halfmove_clock += 1;

        undo
    }

    pub fn unmake_null_move(&mut self, undo: NullUndo) {
        self.side_to_move ^= 1;
        self.ep_square = undo.ep_square;
        self.zobrist = undo.zobrist;
        self.halfmove_clock -= 1;
    }

    pub fn compute_zobrist(&self) -> u64 {
        let mut hash = 0u64;

        #[allow(clippy::needless_range_loop)]
        for sq in 0..64usize {
            let piece = self.mailbox[sq];
            if piece != EMPTY {
                hash ^= ZOBRIST_PIECE[piece as usize][sq];
            }
        }

        if self.side_to_move == BLACK as u8 {
            hash ^= ZOBRIST_SIDE;
        }

        hash ^= ZOBRIST_CASTLING[self.castling as usize];

        if self.ep_square != 64 {
            hash ^= ZOBRIST_EP[(self.ep_square % 8) as usize];
        }

        hash
    }
    pub fn has_non_pawn_material(&self, side: usize) -> bool {
        match side {
            WHITE => {
                self.pieces[WN as usize - 1] != 0
                    || self.pieces[WB as usize - 1] != 0
                    || self.pieces[WR as usize - 1] != 0
                    || self.pieces[WQ as usize - 1] != 0
            }
            BLACK => {
                self.pieces[BN as usize - 1] != 0
                    || self.pieces[BB as usize - 1] != 0
                    || self.pieces[BR as usize - 1] != 0
                    || self.pieces[BQ as usize - 1] != 0
            }
            _ => unreachable!("has_non_pawn_material called with invalid side"),
        }
    }
    pub fn is_repitition(&self) -> bool {
        let current = self.zobrist;
        let len = self.history.len();
        if len < 2 {
            return false;
        }
        let end = len - 1;
        let lookback = end.saturating_sub(self.halfmove_clock as usize);
        self.history[lookback..end].contains(&current)
    }

    pub fn store_killer(&mut self, mv: Move, depth: u8) {
        let d = depth as usize;
        // dont store if it is already killer 0
        if self.killers[d][0] != Some(mv) {
            self.killers[d][1] = self.killers[d][0];
            self.killers[d][0] = Some(mv);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Undo {
    pub castling: u8,
    pub ep_square: u8,
    pub halfmove_clock: u16,
    pub zobrist: u64,
}

pub struct NullUndo {
    pub ep_square: u8,
    pub zobrist: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self::startpos()
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::Board, moves::MoveList};

    #[test]
    fn zobrist_incremental_matches_recomputed() {
        let mut board = Board::startpos();
        let mut list = MoveList::new();
        board.generate_all_moves(&mut list);

        for mv in list.as_slice().iter().copied() {
            eprintln!(
                "making: {} piece={} cap={} flags={:#010x} mailbox[to]={}",
                mv,
                mv.piece(),
                mv.captured(),
                mv.flags(),
                board.mailbox[mv.to() as usize]
            );
            let undo = board.make_move(mv);
            assert_eq!(
                board.zobrist,
                board.compute_zobrist(),
                "zobrist mismatch after {}",
                mv
            );
            board.unmake_move(mv, undo);
            assert_eq!(
                board.zobrist,
                board.compute_zobrist(),
                "zobrist mismatch after unmaking {}",
                mv
            );
        }
    }
}
