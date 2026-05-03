const fn next_rand(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

pub const ZOBRIST_PIECE: [[u64; 64]; 13] = {
    let mut table = [[0u64; 64]; 13];
    let mut state = 0x123456789ABCDEF0u64;
    let mut piece = 1usize; // skip index 0 (EMPTY)

    while piece <= 12 {
        let mut sq = 0usize;
        while sq < 64 {
            table[piece][sq] = next_rand(&mut state);
            sq += 1;
        }
        piece += 1;
    }

    table
};

pub const ZOBRIST_SIDE: u64 = {
    let mut state = 0xDEADBEEFCAFEBABEu64;
    next_rand(&mut state)
};

pub const ZOBRIST_CASTLING: [u64; 16] = {
    let mut table = [0u64; 16];
    let mut state = 0xFEEDFACEDEADC0DEu64;
    let mut i = 0usize;
    while i < 16 {
        table[i] = next_rand(&mut state);
        i += 1;
    }
    table
};

pub const ZOBRIST_EP: [u64; 8] = {
    let mut table = [0u64; 8];
    let mut state = 0xABCDEF0123456789u64;
    let mut i = 0usize;
    while i < 8 {
        table[i] = next_rand(&mut state);
        i += 1;
    }
    table
};

