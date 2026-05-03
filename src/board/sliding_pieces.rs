use kramer_magic_bb::{bishop_attacks, queen_attacks, rook_attacks};

use crate::{
    board::{BLACK, BOTH, BR, Board, WHITE, WR},
    moves::MoveList,
};

impl Board {
    pub fn generate_bishop_moves(&self, list: &mut MoveList) {
        let white = self.side_to_move == WHITE as u8;
        let piece = if white { WR } else { BR };
        let own = if white {
            self.occ[WHITE]
        } else {
            self.occ[BLACK]
        };
        let enemy = if white {
            self.occ[BLACK]
        } else {
            self.occ[WHITE]
        };

        let mut bb = self.pieces[piece as usize - 1];
        while bb != 0 {
            let from = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let attacks = bishop_attacks(from as usize, self.occ[BOTH]) & !own;
            let quiet = attacks & !enemy;
            let caps = attacks & enemy;

            self.push_quiets(from, quiet, piece, list);
            self.push_caps(from, caps, piece, list);
        }
    }
    pub fn generate_rook_moves(&self, list: &mut MoveList) {
        let white = self.side_to_move == WHITE as u8;
        let piece = if white { WR } else { BR };
        let own = if white {
            self.occ[WHITE]
        } else {
            self.occ[BLACK]
        };
        let enemy = if white {
            self.occ[BLACK]
        } else {
            self.occ[WHITE]
        };

        let mut bb = self.pieces[piece as usize - 1];
        while bb != 0 {
            let from = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let attacks = rook_attacks(from as usize, self.occ[BOTH]) & !own;
            let quiet = attacks & !enemy;
            let caps = attacks & enemy;

            self.push_quiets(from, quiet, piece, list);
            self.push_caps(from, caps, piece, list);
        }
    }
    pub fn generate_queen_moves(&self, list: &mut MoveList) {
        let white = self.side_to_move == WHITE as u8;
        let piece = if white { WR } else { BR };
        let own = if white {
            self.occ[WHITE]
        } else {
            self.occ[BLACK]
        };
        let enemy = if white {
            self.occ[BLACK]
        } else {
            self.occ[WHITE]
        };

        let mut bb = self.pieces[piece as usize - 1];
        while bb != 0 {
            let from = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let attacks = queen_attacks(from as usize, self.occ[BOTH]) & !own;
            let quiet = attacks & !enemy;
            let caps = attacks & enemy;

            self.push_quiets(from, quiet, piece, list);
            self.push_caps(from, caps, piece, list);
        }
    }
}
