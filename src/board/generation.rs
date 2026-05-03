pub const RANK_1: u64 = 0x00000000000000FF;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_7: u64 = 0x00FF000000000000;

pub const RANK_3: u64 = 0x0000000000FF0000;
pub const RANK_6: u64 = 0x0000FF0000000000;
pub const RANK_8: u64 = 0xFF00000000000000;

pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

pub const NOT_A: u64 = !FILE_A;
pub const NOT_AB: u64 = !(FILE_A | FILE_B);
pub const NOT_H: u64 = !FILE_H;
pub const NOT_GH: u64 = !(FILE_G | FILE_H);

pub const PROMO_RANK_WHITE: u64 = 0xFF00000000000000;
pub const PROMO_RANK_BLACK: u64 = 0x00000000000000FF;

pub const CASTLE_WK: u8 = 1;
pub const CASTLE_WQ: u8 = 2;
pub const CASTLE_BK: u8 = 4;
pub const CASTLE_BQ: u8 = 8;

pub const WK_EMPTY: u64 = (1 << 5) | (1 << 6); // f1, g1
pub const WQ_EMPTY: u64 = (1 << 1) | (1 << 2) | (1 << 3); // b1, c1, d1
pub const BK_EMPTY: u64 = (1 << 61) | (1 << 62); // f8, g8
pub const BQ_EMPTY: u64 = (1 << 57) | (1 << 58) | (1 << 59); // b8, c8, d8
