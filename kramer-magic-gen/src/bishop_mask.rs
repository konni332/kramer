const fn bishop_mask(square: u8) -> u64 {
    let mut mask = 0u64;
    let rank = square / 8;
    let file = square % 8;

    // north-east
    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= 6 && f <= 6 {
        mask |= 1u64 << (r * 8 + f);
        r += 1;
        f += 1;
    }

    // north-west
    let mut r = rank + 1;
    let mut f = file;
    while r <= 6 && f > 1 {
        f -= 1;
        mask |= 1u64 << (r * 8 + f);
        r += 1;
    }

    // south-east
    let mut r = rank;
    let mut f = file + 1;
    while r > 1 && f <= 6 {
        r -= 1;
        mask |= 1u64 << (r * 8 + f);
        f += 1;
    }

    // south-west
    let mut r = rank;
    let mut f = file;
    while r > 1 && f > 1 {
        r -= 1;
        f -= 1;
        mask |= 1u64 << (r * 8 + f);
    }

    mask
}

pub const BISHOP_MASKS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut i = 0;

    while i < 64 {
        table[i] = bishop_mask(i as u8);
        i += 1;
    }

    table
};

// bishop_mask tests
#[cfg(test)]
mod tests {
    use super::*;

    fn bb(rank: u8, file: u8) -> u64 {
        1u64 << (rank * 8 + file)
    }

    #[test]
    fn bishop_d4_ne_diagonal() {
        let mask = bishop_mask(3 * 8 + 3);
        assert!(mask & bb(4, 4) != 0);
        assert!(mask & bb(5, 5) != 0);
        assert!(mask & bb(6, 6) != 0);
    }

    #[test]
    fn bishop_d4_nw_diagonal() {
        let mask = bishop_mask(3 * 8 + 3);
        assert!(mask & bb(4, 2) != 0);
        assert!(mask & bb(5, 1) != 0);
    }

    #[test]
    fn bishop_d4_se_diagonal() {
        let mask = bishop_mask(3 * 8 + 3);
        assert!(mask & bb(2, 4) != 0);
        assert!(mask & bb(1, 5) != 0);
    }

    #[test]
    fn bishop_d4_sw_diagonal() {
        let mask = bishop_mask(3 * 8 + 3);
        assert!(mask & bb(2, 2) != 0);
        assert!(mask & bb(1, 1) != 0);
    }

    #[test]
    fn bishop_d4_no_edge_squares() {
        let mask = bishop_mask(3 * 8 + 3);
        for i in 0..8u8 {
            assert_eq!(mask & bb(0, i), 0, "rank 0 file {} should be excluded", i);
            assert_eq!(mask & bb(7, i), 0, "rank 7 file {} should be excluded", i);
            assert_eq!(mask & bb(i, 0), 0, "file 0 rank {} should be excluded", i);
            assert_eq!(mask & bb(i, 7), 0, "file 7 rank {} should be excluded", i);
        }
    }

    #[test]
    fn bishop_no_edge_for_all_squares() {
        for sq in 0u8..64 {
            let mask = bishop_mask(sq);
            for i in 0..8u8 {
                assert_eq!(
                    mask & bb(0, i),
                    0,
                    "sq={} rank 0 file {} should be excluded",
                    sq,
                    i
                );
                assert_eq!(
                    mask & bb(7, i),
                    0,
                    "sq={} rank 7 file {} should be excluded",
                    sq,
                    i
                );
                assert_eq!(
                    mask & bb(i, 0),
                    0,
                    "sq={} file 0 rank {} should be excluded",
                    sq,
                    i
                );
                assert_eq!(
                    mask & bb(i, 7),
                    0,
                    "sq={} file 7 rank {} should be excluded",
                    sq,
                    i
                );
            }
        }
    }

    #[test]
    fn bishop_source_square_never_set() {
        for sq in 0u8..64 {
            let mask = bishop_mask(sq);
            assert_eq!(
                mask & (1u64 << sq),
                0,
                "sq={} should not include itself",
                sq
            );
        }
    }

    #[test]
    fn bishop_corner_a1() {
        let mask = bishop_mask(0);
        assert!(mask & bb(1, 1) != 0);
        assert!(mask & bb(2, 2) != 0);
        assert!(mask & bb(3, 3) != 0);
        assert!(mask & bb(4, 4) != 0);
        assert!(mask & bb(5, 5) != 0);
        assert!(mask & bb(6, 6) != 0);
        assert_eq!(mask.count_ones(), 6);
    }

    #[test]
    fn bishop_corner_h1() {
        let mask = bishop_mask(7);
        assert!(mask & bb(1, 6) != 0);
        assert!(mask & bb(2, 5) != 0);
        assert!(mask & bb(3, 4) != 0);
        assert!(mask & bb(4, 3) != 0);
        assert!(mask & bb(5, 2) != 0);
        assert!(mask & bb(6, 1) != 0);
        assert_eq!(mask.count_ones(), 6);
    }

    #[test]
    fn bishop_corner_a8() {
        let mask = bishop_mask(7 * 8);
        assert!(mask & bb(6, 1) != 0);
        assert!(mask & bb(5, 2) != 0);
        assert!(mask & bb(4, 3) != 0);
        assert!(mask & bb(3, 4) != 0);
        assert!(mask & bb(2, 5) != 0);
        assert!(mask & bb(1, 6) != 0);
        assert_eq!(mask.count_ones(), 6);
    }

    #[test]
    fn bishop_corner_h8() {
        let mask = bishop_mask(7 * 8 + 7);
        assert!(mask & bb(6, 6) != 0);
        assert!(mask & bb(5, 5) != 0);
        assert!(mask & bb(4, 4) != 0);
        assert!(mask & bb(3, 3) != 0);
        assert!(mask & bb(2, 2) != 0);
        assert!(mask & bb(1, 1) != 0);
        assert_eq!(mask.count_ones(), 6);
    }

    #[test]
    fn bishop_d4_exact_bit_count() {
        let mask = bishop_mask(3 * 8 + 3);
        assert_eq!(mask.count_ones(), 9);
    }

    #[test]
    fn bishop_e4_exact_bit_count() {
        let mask = bishop_mask(3 * 8 + 4);
        assert_eq!(mask.count_ones(), 9);
    }

    #[test]
    fn bishop_center_e5_exact_bit_count() {
        let mask = bishop_mask(4 * 8 + 4);
        assert_eq!(mask.count_ones(), 9);
    }

    #[test]
    fn bishop_only_diagonal_bits() {
        for sq in 0u8..64 {
            let rank = sq / 8;
            let file = sq % 8;
            let mask = bishop_mask(sq);
            for bit in 0u8..64 {
                if mask & (1u64 << bit) != 0 {
                    let r = bit / 8;
                    let f = bit % 8;
                    let dr = (r as i8 - rank as i8).abs();
                    let df = (f as i8 - file as i8).abs();
                    assert_eq!(
                        dr, df,
                        "sq={}: bit {} at ({},{}) is not on a diagonal",
                        sq, bit, r, f
                    );
                }
            }
        }
    }

    #[test]
    fn bishop_masks_table_matches_function() {
        for sq in 0u8..64 {
            assert_eq!(
                BISHOP_MASKS[sq as usize],
                bishop_mask(sq),
                "BISHOP_MASKS[{}] does not match bishop_mask({})",
                sq,
                sq
            );
        }
    }

    #[test]
    fn bishop_masks_table_no_all_zero_inner_squares() {
        for rank in 1..7u8 {
            for file in 1..7u8 {
                let sq = rank * 8 + file;
                assert_ne!(
                    BISHOP_MASKS[sq as usize], 0,
                    "sq={} ({},{}) should have a non-zero mask",
                    sq, rank, file
                );
            }
        }
    }
}
