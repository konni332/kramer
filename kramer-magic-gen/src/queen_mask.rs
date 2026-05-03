use crate::{bishop_mask::BISHOP_MASKS, rook_mask::ROOK_MASK};

pub const QUEEN_MASK: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut i = 0;

    while i < 64 {
        table[i] = BISHOP_MASKS[i] | ROOK_MASK[i];
        i += 1;
    }

    table
};
