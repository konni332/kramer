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

#[inline(always)]
pub fn score_move(mv: Move) -> i32 {
    if mv.is_capture() {
        MVV_LVA[mv.captured() as usize][mv.piece() as usize]
    } else {
        0
    }
}

pub fn next_best(moves: &mut [Move], start: usize) -> Option<Move> {
    if start >= moves.len() {
        return None;
    }

    let mut best_idx = start;
    let mut best_score = score_move(moves[start]);

    for i in (start + 1)..moves.len() {
        let score = score_move(moves[i]);
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    moves.swap(start, best_idx);
    Some(moves[start])
}
