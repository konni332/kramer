use crate::{
    bishop_attacks::{BISHOP_OFFSETS, BISHOP_SHIFTS, BISHOP_TABLE},
    bishop_mask::BISHOP_MASKS,
    magics::{BISHOP_MAGICS, ROOK_MAGICS},
    rook_attacks::{ROOK_OFFSETS, ROOK_SHIFTS, ROOK_TABLE},
    rook_mask::ROOK_MASKS,
};

mod bishop_attacks;
mod bishop_mask;
mod magics;
mod rook_attacks;
mod rook_mask;

pub const fn rook_attacks(sq: usize, occ: u64) -> u64 {
    let idx = (occ & ROOK_MASKS[sq]).wrapping_mul(ROOK_MAGICS[sq]) >> ROOK_SHIFTS[sq];
    ROOK_TABLE[ROOK_OFFSETS[sq] + idx as usize]
}

pub const fn bishop_attacks(sq: usize, occ: u64) -> u64 {
    let idx = (occ & BISHOP_MASKS[sq]).wrapping_mul(BISHOP_MAGICS[sq]) >> BISHOP_SHIFTS[sq];
    BISHOP_TABLE[BISHOP_OFFSETS[sq] + idx as usize]
}

pub const fn queen_attacks(sq: usize, occ: u64) -> u64 {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}
