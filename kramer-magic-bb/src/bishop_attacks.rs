use crate::{AlignedBishopTable, bishop_mask::BISHOP_MASKS, magics::BISHOP_MAGICS};
pub const BISHOP_SHIFTS: [u8; 64] = {
    let mut shifts = [0u8; 64];
    let mut i = 0;
    while i < 64 {
        shifts[i] = 64 - BISHOP_MASKS[i].count_ones() as u8;
        i += 1;
    }

    shifts
};

pub const BISHOP_OFFSETS: [usize; 64] = {
    let mut offsets = [0usize; 64];
    let mut i = 1;
    while i < 64 {
        let bits = BISHOP_MASKS[i - 1].count_ones() as usize;
        offsets[i] = offsets[i - 1] + (1 << bits);
        i += 1;
    }
    offsets
};

pub const BISHOP_TABLE_SIZE: usize = {
    let bits = BISHOP_MASKS[63].count_ones() as usize;
    BISHOP_OFFSETS[63] + (1 << bits)
};

pub static BISHOP_TABLE: AlignedBishopTable = {
    let mut table = [0u64; BISHOP_TABLE_SIZE];
    let mut sq = 0usize;

    while sq < 64 {
        let mask = BISHOP_MASKS[sq];
        let magic = BISHOP_MAGICS[sq];
        let shift = BISHOP_SHIFTS[sq];
        let offset = BISHOP_OFFSETS[sq];

        let mut subset = 0u64;
        loop {
            let attacks = bishop_attacks_on_the_fly(sq as u8, subset);
            let idx = subset.wrapping_mul(magic) >> shift;
            table[offset + idx as usize] = attacks;

            subset = subset.wrapping_sub(mask) & mask;
            if subset == 0 {
                break;
            }
        }

        sq += 1;
    }

    AlignedBishopTable(table)
};

pub const fn bishop_attacks_on_the_fly(sq: u8, blockers: u64) -> u64 {
    let mut attacks = 0u64;
    let rank = sq / 8;
    let file = sq % 8;

    // north-east
    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= 7 && f <= 7 {
        let bit = 1u64 << (r * 8 + f);
        attacks |= bit;
        if blockers & bit != 0 {
            break;
        }
        r += 1;
        f += 1;
    }
    // north-west
    let mut r = rank + 1;
    let mut f = file;
    while r <= 7 && f > 0 {
        f -= 1;
        let bit = 1u64 << (r * 8 + f);
        attacks |= bit;
        if blockers & bit != 0 {
            break;
        }
        r += 1;
    }
    // south-east
    let mut r = rank;
    let mut f = file + 1;
    while r > 0 && f <= 7 {
        r -= 1;
        let bit = 1u64 << (r * 8 + f);
        attacks |= bit;
        if blockers & bit != 0 {
            break;
        }
        f += 1;
    }
    // south-west
    let mut r = rank;
    let mut f = file;
    while r > 0 && f > 0 {
        r -= 1;
        f -= 1;
        let bit = 1u64 << (r * 8 + f);
        attacks |= bit;
        if blockers & bit != 0 {
            break;
        }
    }

    attacks
}
