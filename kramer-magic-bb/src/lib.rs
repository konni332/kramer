use crate::{
    bishop_attacks::{BISHOP_OFFSETS, BISHOP_SHIFTS, BISHOP_TABLE, BISHOP_TABLE_SIZE},
    bishop_mask::BISHOP_MASKS,
    magics::{BISHOP_MAGICS, ROOK_MAGICS},
    rook_attacks::{ROOK_OFFSETS, ROOK_SHIFTS, ROOK_TABLE, ROOK_TABLE_SIZE},
    rook_mask::ROOK_MASKS,
};

mod bishop_attacks;
mod bishop_mask;
mod magics;
mod rook_attacks;
mod rook_mask;

pub const fn rook_attacks(sq: usize, occ: u64) -> u64 {
    let idx = (occ & ROOK_MASKS[sq]).wrapping_mul(ROOK_MAGICS[sq]) >> ROOK_SHIFTS[sq];
    ROOK_TABLE.0[ROOK_OFFSETS[sq] + idx as usize]
}

pub const fn bishop_attacks(sq: usize, occ: u64) -> u64 {
    let idx = (occ & BISHOP_MASKS[sq]).wrapping_mul(BISHOP_MAGICS[sq]) >> BISHOP_SHIFTS[sq];
    BISHOP_TABLE.0[BISHOP_OFFSETS[sq] + idx as usize]
}

pub const fn queen_attacks(sq: usize, occ: u64) -> u64 {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}

// quick sanity check — a1 rook with no blockers should attack all of file a and rank 1 except edges
#[test]
fn rook_table_sanity() {
    use crate::rook_attacks::rook_attacks_on_the_fly;
    for sq in 0..64usize {
        let expected = rook_attacks_on_the_fly(sq as u8, 0);
        let got = rook_attacks(sq, 0);
        assert_eq!(expected, got, "mismatch at sq={} with no blockers", sq);
    }
}

#[repr(align(64))]
pub struct AlignedRookTable(pub [u64; ROOK_TABLE_SIZE]);

#[repr(align(64))]
pub struct AlignedBishopTable(pub [u64; BISHOP_TABLE_SIZE]);
