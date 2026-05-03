use crate::find_magic;

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

fn bishop_attacks_on_the_fly(sq: u8, blockers: u64) -> u64 {
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

pub fn generate_bishop_magics() {
    println!("pub const BISHOP_MAGICS: [u64; 64] = [");
    for sq in 0..64u8 {
        let mask = bishop_mask(sq);
        let magic = find_magic(sq, mask, bishop_attacks_on_the_fly);
        if sq % 4 == 0 {
            print!("    ");
        }
        print!("0x{:016x}", magic);
        if sq < 63 {
            print!(", ");
            if sq % 4 == 3 {
                println!();
            }
        }
    }
    println!("\n];");
}
