use crate::board::{BB, BN, BP, BQ, BR, Board, WB, WHITE, WN, WP, WQ, WR};

impl Board {
    pub fn evaluate(&self) -> i32 {
        let mut score = 0i32;

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

        if self.side_to_move == WHITE as u8 {
            score
        } else {
            -score
        }
    }
}
