pub const fn rook_mask(square: u8) -> u64 {
    let mut mask = 0u64;

    let rank = square / 8;
    let file = square % 8;

    // north
    let mut r = rank + 1;
    while r <= 6 {
        mask |= 1u64 << (r * 8 + file);
        r += 1;
    }

    // south
    let mut r = rank;
    while r > 1 {
        r -= 1;
        mask |= 1u64 << (r * 8 + file);
    }

    // east
    let mut f = file + 1;
    while f <= 6 {
        mask |= 1u64 << (rank * 8 + f);
        f += 1;
    }

    // west
    let mut f = file;
    while f > 1 {
        f -= 1;
        mask |= 1u64 << (rank * 8 + f);
    }

    mask
}

pub const ROOK_MASKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut i = 0;

    while i < 64 {
        table[i] = rook_mask(i as u8);
        i += 1;
    }

    table
};

#[cfg(test)]
mod tests {
    use super::*;

    fn bb(rank: u8, file: u8) -> u64 {
        1u64 << (rank * 8 + file)
    }

    #[test]
    fn rook_d4_north_ray() {
        let mask = rook_mask(3 * 8 + 3);
        assert!(mask & bb(4, 3) != 0);
        assert!(mask & bb(5, 3) != 0);
        assert!(mask & bb(6, 3) != 0);
    }

    #[test]
    fn rook_d4_south_ray() {
        let mask = rook_mask(3 * 8 + 3);
        assert!(mask & bb(2, 3) != 0);
        assert!(mask & bb(1, 3) != 0);
    }

    #[test]
    fn rook_d4_east_ray() {
        let mask = rook_mask(3 * 8 + 3);
        assert!(mask & bb(3, 4) != 0);
        assert!(mask & bb(3, 5) != 0);
        assert!(mask & bb(3, 6) != 0);
    }

    #[test]
    fn rook_d4_west_ray() {
        let mask = rook_mask(3 * 8 + 3);
        assert!(mask & bb(3, 2) != 0);
        assert!(mask & bb(3, 1) != 0);
    }

    #[test]
    fn rook_d4_no_edge_squares() {
        let mask = rook_mask(3 * 8 + 3);
        for i in 0..8u8 {
            assert_eq!(mask & bb(0, i), 0, "rank 0 file {} should be excluded", i);
            assert_eq!(mask & bb(7, i), 0, "rank 7 file {} should be excluded", i);
            assert_eq!(mask & bb(i, 0), 0, "file 0 rank {} should be excluded", i);
            assert_eq!(mask & bb(i, 7), 0, "file 7 rank {} should be excluded", i);
        }
    }

    #[test]
    fn rook_no_edge_for_all_squares() {
        for sq in 0u8..64 {
            let rank = sq / 8;
            let file = sq % 8;
            let mask = rook_mask(sq);

            assert_eq!(
                mask & bb(7, file),
                0,
                "sq={} north ray must not reach rank 7",
                sq
            );
            assert_eq!(
                mask & bb(0, file),
                0,
                "sq={} south ray must not reach rank 0",
                sq
            );
            assert_eq!(
                mask & bb(rank, 7),
                0,
                "sq={} east ray must not reach file 7",
                sq
            );
            assert_eq!(
                mask & bb(rank, 0),
                0,
                "sq={} west ray must not reach file 0",
                sq
            );
        }
    }

    #[test]
    fn rook_source_square_never_set() {
        for sq in 0u8..64 {
            let mask = rook_mask(sq);
            assert_eq!(
                mask & (1u64 << sq),
                0,
                "sq={} should not include itself",
                sq
            );
        }
    }

    #[test]
    fn rook_corner_a1() {
        let mask = rook_mask(0);
        assert!(mask & bb(1, 0) != 0);
        assert!(mask & bb(2, 0) != 0);
        assert!(mask & bb(6, 0) != 0);
        assert!(mask & bb(0, 1) != 0);
        assert!(mask & bb(0, 2) != 0);
        assert!(mask & bb(0, 6) != 0);
        assert_eq!(mask & bb(7, 0), 0);
        assert_eq!(mask & bb(0, 7), 0);
        assert_eq!(mask.count_ones(), 12);
    }

    #[test]
    fn rook_corner_h8() {
        let mask = rook_mask(7 * 8 + 7);
        assert!(mask & bb(6, 7) != 0);
        assert!(mask & bb(1, 7) != 0);
        assert!(mask & bb(7, 6) != 0);
        assert!(mask & bb(7, 1) != 0);
        assert_eq!(mask.count_ones(), 12);
    }

    #[test]
    fn rook_d4_exact_bit_count() {
        let mask = rook_mask(3 * 8 + 3);
        assert_eq!(mask.count_ones(), 10);
    }

    #[test]
    fn rook_e5_exact_bit_count() {
        let mask = rook_mask(4 * 8 + 4);
        assert_eq!(mask.count_ones(), 10);
    }

    #[test]
    fn rook_a1_exact_bit_count() {
        let mask = rook_mask(0);
        assert_eq!(mask.count_ones(), 12);
    }

    #[test]
    fn rook_d1_exact_bit_count() {
        let mask = rook_mask(3);
        assert_eq!(mask.count_ones(), 11);
    }

    #[test]
    fn rook_only_rank_and_file_bits() {
        for sq in 0u8..64 {
            let rank = sq / 8;
            let file = sq % 8;
            let mask = rook_mask(sq);
            for bit in 0u8..64 {
                if mask & (1u64 << bit) != 0 {
                    let r = bit / 8;
                    let f = bit % 8;
                    assert!(
                        r == rank || f == file,
                        "sq={}: bit {} at ({},{}) is not on the same rank or file",
                        sq,
                        bit,
                        r,
                        f
                    );
                }
            }
        }
    }

    #[test]
    fn rook_mask_table_matches_function() {
        for sq in 0u8..64 {
            assert_eq!(
                ROOK_MASKS[sq as usize],
                rook_mask(sq),
                "ROOK_MASK[{}] does not match rook_mask({})",
                sq,
                sq
            );
        }
    }

    #[test]
    fn rook_mask_table_no_all_zero_squares() {
        for sq in 0..64 {
            assert_ne!(ROOK_MASKS[sq], 0, "ROOK_MASK[{}] should not be zero", sq);
        }
    }
}
