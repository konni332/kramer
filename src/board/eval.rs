use crate::board::{
    BB, BK, BN, BP, BQ, BR, Board, WB, WHITE, WK, WN, WP, WQ, WR,
    eval::pst::{
        BISHOP_PST, KING_MG_PST, KNIGHT_PST, PAWN_PST, QUEEN_PST, ROOK_PST, black_pst_index,
        white_pst_index,
    },
};

mod pst;

impl Board {
    pub fn evaluate(&self) -> i32 {
        let mut score = 0i32;

        // material
        score += self.pieces[WP as usize - 1].count_ones() as i32 * 100;
        score += self.pieces[WN as usize - 1].count_ones() as i32 * 320;
        score += self.pieces[WB as usize - 1].count_ones() as i32 * 330;
        score += self.pieces[WR as usize - 1].count_ones() as i32 * 500;
        score += self.pieces[WQ as usize - 1].count_ones() as i32 * 900;

        score -= self.pieces[BP as usize - 1].count_ones() as i32 * 100;
        score -= self.pieces[BN as usize - 1].count_ones() as i32 * 320;
        score -= self.pieces[BB as usize - 1].count_ones() as i32 * 330;
        score -= self.pieces[BR as usize - 1].count_ones() as i32 * 500;
        score -= self.pieces[BQ as usize - 1].count_ones() as i32 * 900;

        // PST bonuses
        score += pst_score(self.pieces[WP as usize - 1], &PAWN_PST, false);
        score += pst_score(self.pieces[WN as usize - 1], &KNIGHT_PST, false);
        score += pst_score(self.pieces[WB as usize - 1], &BISHOP_PST, false);
        score += pst_score(self.pieces[WR as usize - 1], &ROOK_PST, false);
        score += pst_score(self.pieces[WQ as usize - 1], &QUEEN_PST, false);
        score += pst_score(self.pieces[WK as usize - 1], &KING_MG_PST, false);
        score -= pst_score(self.pieces[BP as usize - 1], &PAWN_PST, true);
        score -= pst_score(self.pieces[BN as usize - 1], &KNIGHT_PST, true);
        score -= pst_score(self.pieces[BB as usize - 1], &BISHOP_PST, true);
        score -= pst_score(self.pieces[BR as usize - 1], &ROOK_PST, true);
        score -= pst_score(self.pieces[BQ as usize - 1], &QUEEN_PST, true);
        score -= pst_score(self.pieces[BK as usize - 1], &KING_MG_PST, true);

        if self.side_to_move == WHITE as u8 {
            score
        } else {
            -score
        }
    }
}

#[inline(always)]
fn pst_score(mut bb: u64, table: &[i32; 64], flip: bool) -> i32 {
    let mut score = 0i32;
    while bb != 0 {
        let sq = bb.trailing_zeros() as u8;
        bb &= bb - 1;
        let idx = if flip {
            black_pst_index(sq)
        } else {
            white_pst_index(sq)
        };
        score += table[idx];
    }
    score
}
