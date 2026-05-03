use crate::{bishop::generate_bishop_magics, rook::generate_rook_magics};

mod bishop;
mod rook;

fn main() {
    generate_rook_magics();
    println!();
    generate_bishop_magics();
}

fn next_rand(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn find_magic(sq: u8, mask: u64, attacks_fn: fn(u8, u64) -> u64) -> u64 {
    let bits = mask.count_ones() as u8;
    let shift = 64 - bits;
    let table_size = 1usize << bits;

    let mut subsets = vec![0u64; table_size];
    let mut attacks = vec![0u64; table_size];

    let mut subset = 0u64;
    let mut i = 0;
    loop {
        subsets[i] = subset;
        attacks[i] = attacks_fn(sq, subset);
        i += 1;
        subset = subset.wrapping_sub(mask) & mask;
        if subset == 0 {
            break;
        }
    }

    let mut rng = 0x123456789ABCDEF0u64 ^ (sq as u64).wrapping_mul(0x9e3779b97f4a7c15);

    'outer: loop {
        let magic = next_rand(&mut rng) & next_rand(&mut rng) & next_rand(&mut rng);

        if (mask.wrapping_mul(magic) >> 56).count_ones() < 6 {
            continue;
        }

        let mut used = vec![0u64; table_size];
        let mut used_flag = vec![false; table_size];

        for j in 0..table_size {
            let idx = (subsets[j].wrapping_mul(magic) >> shift) as usize;
            if !used_flag[idx] {
                used_flag[idx] = true;
                used[idx] = attacks[j];
            } else if used[idx] != attacks[j] {
                continue 'outer;
            }
        }

        return magic;
    }
}
