use crate::{magics::ROOK_MAGICS, rook_mask::ROOK_MASKS};
pub const ROOK_SHIFTS: [u8; 64] = {
    let mut shifts = [0u8; 64];
    let mut i = 0;
    while i < 64 {
        shifts[i] = 64 - ROOK_MASKS[i].count_ones() as u8;
        i += 1;
    }
    shifts
};

pub const ROOK_OFFSETS: [usize; 64] = {
    let mut offsets = [0usize; 64];
    let mut i = 1;
    while i < 64 {
        let bits = ROOK_MASKS[i - 1].count_ones() as usize;
        offsets[i] = offsets[i - 1] + (1 << bits);
        i += 1;
    }
    offsets
};

pub const ROOK_TABLE_SIZE: usize = {
    let bits = ROOK_MASKS[63].count_ones() as usize;
    ROOK_OFFSETS[63] + (1 << bits)
};

pub const ROOK_TABLE: [u64; ROOK_TABLE_SIZE] = {
    let mut table = [0u64; ROOK_TABLE_SIZE];
    let mut sq = 0usize;

    while sq < 64 {
        let mask = ROOK_MASKS[sq];
        let magic = ROOK_MAGICS[sq];
        let shift = ROOK_SHIFTS[sq];
        let offset = ROOK_OFFSETS[sq];

        let mut subset = 0u64;
        loop {
            let attacks = rook_attacks_on_the_fly(sq as u8, subset);
            let idx = subset.wrapping_mul(magic) >> shift;
            table[offset + idx as usize] = attacks;

            subset = subset.wrapping_sub(mask) & mask;
            if subset == 0 {
                break;
            }
        }

        sq += 1;
    }

    table
};

pub const fn rook_attacks_on_the_fly(sq: u8, blockers: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = sq / 8;
    let file = sq % 8;

    // north
    let mut r = rank + 1;
    while r <= 7 {
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if blockers & bit != 0 {
            break;
        }
        r += 1;
    }

    // south
    let mut r = rank;
    while r > 0 {
        r -= 1;
        let bit = 1u64 << (r * 8 + file);
        attacks |= bit;
        if blockers & bit != 0 {
            break;
        }
    }

    // east
    let mut f = file + 1;
    while f <= 7 {
        let bit = 1u64 << (rank * 8 + f);
        attacks |= bit;
        if blockers & bit != 0 {
            break;
        }
        f += 1;
    }

    // west
    let mut f = file;
    while f > 0 {
        f -= 1;
        let bit = 1u64 << (rank * 8 + f);
        attacks |= bit;
        if blockers & bit != 0 {
            break;
        }
    }

    attacks
}
