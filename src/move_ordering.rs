use crate::moves::Move;

/// indexed by [victim][attacker]
/// higher score = search first
#[rustfmt::skip]
const MVV_LVA: [[i32; 13]; 13] = [
//  attacker: empty  WP    WN    WB    WR    WQ    WK    BP    BN    BB    BR    BQ    BK
    [0,        0,    0,    0,    0,    0,    0,    0,    0,    0,    0,    0,    0 ], // victim: empty
    [0,        105,  104,  103,  102,  101,  100,  105,  104,  103,  102,  101,  100], // victim: WP
    [0,        205,  204,  203,  202,  201,  200,  205,  204,  203,  202,  201,  200], // victim: WN
    [0,        305,  304,  303,  302,  301,  300,  305,  304,  303,  302,  301,  300], // victim: WB
    [0,        405,  404,  403,  402,  401,  400,  405,  404,  403,  402,  401,  400], // victim: WR
    [0,        505,  504,  503,  502,  501,  500,  505,  504,  503,  502,  501,  500], // victim: WQ
    [0,        605,  604,  603,  602,  601,  600,  605,  604,  603,  602,  601,  600], // victim: WK
    [0,        105,  104,  103,  102,  101,  100,  105,  104,  103,  102,  101,  100], // victim: BP
    [0,        205,  204,  203,  202,  201,  200,  205,  204,  203,  202,  201,  200], // victim: BN
    [0,        305,  304,  303,  302,  301,  300,  305,  304,  303,  302,  301,  300], // victim: BB
    [0,        405,  404,  403,  402,  401,  400,  405,  404,  403,  402,  401,  400], // victim: BR
    [0,        505,  504,  503,  502,  501,  500,  505,  504,  503,  502,  501,  500], // victim: BQ
    [0,        605,  604,  603,  602,  601,  600,  605,  604,  603,  602,  601,  600], // victim: BK
];

const KILLER_SCORE_0: i32 = 90;
const KILLER_SCORE_1: i32 = 80;

#[inline(always)]
pub fn score_move(mv: Move, killers: &[Option<Move>; 2]) -> i32 {
    if mv.is_capture() {
        MVV_LVA[mv.captured() as usize][mv.piece() as usize]
    } else if killers[0] == Some(mv) {
        KILLER_SCORE_0
    } else if killers[1] == Some(mv) {
        KILLER_SCORE_1
    } else {
        0
    }
}

pub fn next_best(moves: &mut [Move], start: usize, killers: &[Option<Move>; 2]) -> Option<Move> {
    if start >= moves.len() {
        return None;
    }

    let mut best_idx = start;
    let mut best_score = score_move(moves[start], killers);

    #[allow(clippy::needless_range_loop)]
    for i in (start + 1)..moves.len() {
        let score = score_move(moves[i], killers);
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    moves.swap(start, best_idx);
    Some(moves[start])
}
