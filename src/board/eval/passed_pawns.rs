pub const WHITE_PASSED_MASK: [u64; 64] = build_white_passed_masks();
pub const BLACK_PASSED_MASK: [u64; 64] = build_black_passed_masks();

// bonus for passed pawn by rank (rank 0 = back rank, rank 7 = promotion)
// rank 0 and 1 are impossible for passed pawns in practice
pub const PASSED_PAWN_MG_BONUS: [i32; 8] = [0, 0, 10, 15, 25, 40, 60, 0];
pub const PASSED_PAWN_EG_BONUS: [i32; 8] = [0, 0, 20, 35, 60, 90, 130, 0];

const fn build_white_passed_masks() -> [u64; 64] {
    let mut masks = [0u64; 64];
    let mut sq = 0usize;
    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;
        let mut mask = 0u64;
        let mut r = rank + 1;
        while r < 8 {
            // same file
            mask |= 1u64 << (r * 8 + file);
            // adjacent files
            if file > 0 {
                mask |= 1u64 << (r * 8 + file - 1);
            }
            if file < 7 {
                mask |= 1u64 << (r * 8 + file + 1);
            }
            r += 1;
        }
        masks[sq] = mask;
        sq += 1;
    }
    masks
}

const fn build_black_passed_masks() -> [u64; 64] {
    let mut masks = [0u64; 64];
    let mut sq = 0usize;
    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;
        let mut mask = 0u64;
        // for black, look at ranks below
        let mut r = 0usize;
        while r < rank {
            mask |= 1u64 << (r * 8 + file);
            if file > 0 {
                mask |= 1u64 << (r * 8 + file - 1);
            }
            if file < 7 {
                mask |= 1u64 << (r * 8 + file + 1);
            }
            r += 1;
        }
        masks[sq] = mask;
        sq += 1;
    }
    masks
}
