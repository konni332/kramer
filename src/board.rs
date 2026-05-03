mod generation;
mod king;
mod knight;
mod pawns;

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

/// # Piece encoding:
/// 0 empty
///
/// 1 WP
/// 2 WN
/// 3 WB
/// 4 WR
/// 5 WQ
/// 6 WK
///
/// 7 BP
/// 8 BN
/// 9 BB
/// 10 BR
/// 11 BQ
/// 12 BK
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        }
    }

    pub fn startpos() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Start position failed to be parsed")
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

    #[inline(always)]
    pub fn piece_at(&self, sq: Square) -> u8 {
        self.mailbox[sq as usize]
    }

    fn recompute_incrementals(&mut self) {}
}

impl Default for Board {
    fn default() -> Self {
        Self::startpos()
    }
}
