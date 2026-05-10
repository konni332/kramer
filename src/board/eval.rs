use crate::board::{
    BB, BK, BN, BP, BQ, BR, Board, WB, WHITE, WK, WN, WP, WQ, WR,
    eval::{
        passed_pawns::{
            BLACK_PASSED_MASK, PASSED_PAWN_EG_BONUS, PASSED_PAWN_MG_BONUS, WHITE_PASSED_MASK,
        },
        pst::{EG_TABLE, MG_TABLE},
    },
};

mod passed_pawns;
mod pst;

// phase increments per piece type — pawns and kings don't count
// indices match our piece-1 array: 0=WP,1=WN,2=WB,3=WR,4=WQ,5=WK,6=BP..11=BK
const PHASE_INC: [i32; 12] = [0, 1, 1, 2, 4, 0, 0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

impl Board {
    pub fn evaluate(&self) -> i32 {
        let mut mg_white = 0i32;
        let mut eg_white = 0i32;
        let mut mg_black = 0i32;
        let mut eg_black = 0i32;
        let mut phase = 0i32;

        // white pieces (array indices 0..5)
        score_pieces(
            self.pieces[WP as usize - 1],
            0,
            &mut mg_white,
            &mut eg_white,
            &mut phase,
        );
        score_pieces(
            self.pieces[WN as usize - 1],
            1,
            &mut mg_white,
            &mut eg_white,
            &mut phase,
        );
        score_pieces(
            self.pieces[WB as usize - 1],
            2,
            &mut mg_white,
            &mut eg_white,
            &mut phase,
        );
        score_pieces(
            self.pieces[WR as usize - 1],
            3,
            &mut mg_white,
            &mut eg_white,
            &mut phase,
        );
        score_pieces(
            self.pieces[WQ as usize - 1],
            4,
            &mut mg_white,
            &mut eg_white,
            &mut phase,
        );
        score_pieces(
            self.pieces[WK as usize - 1],
            5,
            &mut mg_white,
            &mut eg_white,
            &mut phase,
        );

        // black pieces (array indices 6..11)
        score_pieces(
            self.pieces[BP as usize - 1],
            6,
            &mut mg_black,
            &mut eg_black,
            &mut phase,
        );
        score_pieces(
            self.pieces[BN as usize - 1],
            7,
            &mut mg_black,
            &mut eg_black,
            &mut phase,
        );
        score_pieces(
            self.pieces[BB as usize - 1],
            8,
            &mut mg_black,
            &mut eg_black,
            &mut phase,
        );
        score_pieces(
            self.pieces[BR as usize - 1],
            9,
            &mut mg_black,
            &mut eg_black,
            &mut phase,
        );
        score_pieces(
            self.pieces[BQ as usize - 1],
            10,
            &mut mg_black,
            &mut eg_black,
            &mut phase,
        );
        score_pieces(
            self.pieces[BK as usize - 1],
            11,
            &mut mg_black,
            &mut eg_black,
            &mut phase,
        );

        // bishop pair bonus
        if self.pieces[WB as usize - 1].count_ones() >= 2 {
            mg_white += 30;
            eg_white += 50;
        }
        if self.pieces[BB as usize - 1].count_ones() >= 2 {
            mg_black += 30;
            eg_black += 50;
        }

        // passed pawns bonus
        let bp = self.pieces[BP as usize - 1];
        let wp = self.pieces[WP as usize - 1];

        let mut white_pawns = wp;
        while white_pawns != 0 {
            let sq = white_pawns.trailing_zeros() as usize;
            white_pawns &= white_pawns - 1;
            if WHITE_PASSED_MASK[sq] & bp == 0 {
                let rank = sq / 8;
                mg_white += PASSED_PAWN_MG_BONUS[rank];
                eg_white += PASSED_PAWN_EG_BONUS[rank];
            }
        }

        let mut black_pawns = bp;
        while black_pawns != 0 {
            let sq = black_pawns.trailing_zeros() as usize;
            black_pawns &= black_pawns - 1;
            if BLACK_PASSED_MASK[sq] & bp == 0 {
                let rank = 7 - (sq / 8);
                mg_black += PASSED_PAWN_MG_BONUS[rank];
                eg_black += PASSED_PAWN_EG_BONUS[rank];
            }
        }

        let phase = phase.min(MAX_PHASE);
        let eg_phase = MAX_PHASE - phase;

        let mg_score = mg_white - mg_black;
        let eg_score = eg_white - eg_black;

        let score = (mg_score * phase + eg_score * eg_phase) / MAX_PHASE;

        if self.side_to_move == WHITE as u8 {
            score
        } else {
            -score
        }
    }
}

#[inline(always)]
fn score_pieces(mut bb: u64, piece_idx: usize, mg: &mut i32, eg: &mut i32, phase: &mut i32) {
    while bb != 0 {
        let sq = bb.trailing_zeros() as usize;
        bb &= bb - 1;
        *mg += MG_TABLE[piece_idx][sq];
        *eg += EG_TABLE[piece_idx][sq];
        *phase += PHASE_INC[piece_idx];
    }
}
