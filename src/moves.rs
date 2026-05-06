use vampirc_uci::{UciMove, UciPiece, UciSquare};

use crate::board::Square;

pub const FLAG_CAPTURE: u32 = 1 << 24;
pub const FLAG_DOUBLE: u32 = 1 << 25;
pub const FLAG_EP: u32 = 1 << 26;
pub const FLAG_CASTLE: u32 = 1 << 27;
pub const FLAG_PROMOTION: u32 = 1 << 28;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveList {
    moves: [Move; 256],
    len: usize,
}

impl Move {
    pub const fn new(from: u8, to: u8, piece: u8, captured: u8, promo: u8, flags: u32) -> Self {
        Self(
            (from as u32)
                | ((to as u32) << 6)
                | ((piece as u32) << 12)
                | ((captured as u32) << 16)
                | ((promo as u32) << 20)
                | flags,
        )
    }
    #[inline(always)]
    pub const fn from(self) -> u8 {
        (self.0 & 0x3F) as u8
    }

    #[inline(always)]
    pub const fn to(self) -> u8 {
        ((self.0 >> 6) & 0x3F) as u8
    }

    #[inline(always)]
    pub const fn piece(self) -> u8 {
        ((self.0 >> 12) & 0xF) as u8
    }

    #[inline(always)]
    pub const fn captured(self) -> u8 {
        ((self.0 >> 16) & 0xF) as u8
    }

    #[inline(always)]
    pub const fn promo(self) -> u8 {
        ((self.0 >> 20) & 0xF) as u8
    }

    #[inline(always)]
    pub const fn flags(self) -> u32 {
        self.0 & 0xFF00_0000
    }

    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        self.flags() & FLAG_CAPTURE != 0
    }

    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        self.flags() & FLAG_PROMOTION != 0
    }

    #[inline(always)]
    pub const fn is_castle(self) -> bool {
        self.flags() & FLAG_CASTLE != 0
    }

    #[inline(always)]
    pub const fn is_ep(self) -> bool {
        self.flags() & FLAG_EP != 0
    }

    #[inline(always)]
    pub const fn is_double_push(self) -> bool {
        self.flags() & FLAG_DOUBLE != 0
    }
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move(0); 256],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        debug_assert!(self.len < 256);
        self.moves[self.len] = mv;
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        &mut self.moves[..self.len]
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_square(f, self.from())?;
        write_square(f, self.to())?;

        if self.is_promotion() {
            let ch = promo_char(self.promo());
            f.write_str(ch.encode_utf8(&mut [0; 4]))?;
        }

        Ok(())
    }
}

impl From<UciMove> for Move {
    fn from(value: UciMove) -> Self {
        let from = sq_from_uci(value.from);
        let to = sq_from_uci(value.to);
        // we don't have full board context here so we can't fill piece/captured
        // this is only used for parsing position moves, where we match by from/to string
        // so piece and captured don't matter — we look up the real move from the legal list
        Move::new(from, to, 0, 0, 0, 0)
    }
}

impl From<Move> for UciMove {
    fn from(value: Move) -> Self {
        let promo = if value.is_promotion() {
            Some(match value.promo() {
                2 | 8 => UciPiece::Knight,
                3 | 9 => UciPiece::Bishop,
                4 | 10 => UciPiece::Rook,
                5 | 11 => UciPiece::Queen,
                _ => unreachable!("invalid promotion piece"),
            })
        } else {
            None
        };

        UciMove {
            from: sq_to_uci(value.from()),
            to: sq_to_uci(value.to()),
            promotion: promo,
        }
    }
}

fn sq_from_uci(sq: UciSquare) -> u8 {
    let file = sq.file as u8 - b'a';
    let rank = sq.rank - 1;
    rank * 8 + file
}

fn sq_to_uci(sq: u8) -> UciSquare {
    UciSquare {
        file: (b'a' + (sq % 8)) as char,
        rank: (sq / 8) + 1,
    }
}

#[inline(always)]
fn write_square(f: &mut std::fmt::Formatter<'_>, sq: u8) -> std::fmt::Result {
    let file = (b'a' + (sq % 8)) as char;
    let rank = (b'1' + (sq / 8)) as char;

    f.write_str(file.encode_utf8(&mut [0; 4]))?;
    f.write_str(rank.encode_utf8(&mut [0; 4]))
}

#[inline(always)]
fn promo_char(piece: u8) -> char {
    match piece {
        // white promos
        2 | 8 => 'n',
        3 | 9 => 'b',
        4 | 10 => 'r',
        5 | 11 => 'q',
        _ => unreachable!("invalid promotion piece"),
    }
}

#[inline(always)]
pub fn sq(file: char, rank: char) -> Square {
    let f = file as u8 - b'a';
    let r = rank as u8 - b'1';
    r * 8 + f
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{BN, BP, WK, WN, WP, WQ, WR};

    fn check(mv: Move, expected: &str) {
        assert_eq!(mv.to_string(), expected, "move display mismatch");
    }

    #[test]
    fn quiet_moves() {
        check(Move::new(sq('e', '2'), sq('e', '4'), WP, 0, 0, 0), "e2e4");

        check(Move::new(sq('e', '7'), sq('e', '5'), BP, 0, 0, 0), "e7e5");
    }

    #[test]
    fn knight_moves() {
        check(Move::new(sq('b', '1'), sq('c', '3'), WN, 0, 0, 0), "b1c3");

        check(Move::new(sq('b', '1'), sq('a', '3'), WN, 0, 0, 0), "b1a3");
    }

    #[test]
    fn rook_moves() {
        check(Move::new(sq('a', '1'), sq('a', '3'), WR, 0, 0, 0), "a1a3");

        check(Move::new(sq('h', '1'), sq('h', '5'), WR, 0, 0, 0), "h1h5");
    }

    #[test]
    fn king_moves_and_castle_style() {
        check(Move::new(sq('e', '1'), sq('g', '1'), WK, 0, 0, 0), "e1g1");

        check(Move::new(sq('e', '1'), sq('c', '1'), WK, 0, 0, 0), "e1c1");
    }

    #[test]
    fn promotions() {
        check(
            Move::new(sq('e', '7'), sq('e', '8'), WP, 0, WQ, FLAG_PROMOTION),
            "e7e8q",
        );

        check(
            Move::new(sq('e', '2'), sq('e', '1'), BP, 0, BN, FLAG_PROMOTION),
            "e2e1n",
        );
    }

    #[test]
    fn mixed_promotion_captures() {
        check(
            Move::new(
                sq('d', '7'),
                sq('d', '8'),
                WP,
                BP,
                WQ,
                FLAG_CAPTURE | FLAG_PROMOTION,
            ),
            "d7d8q",
        );
    }

    #[test]
    fn edge_files() {
        check(Move::new(sq('a', '1'), sq('h', '1'), WR, 0, 0, 0), "a1h1");

        check(Move::new(sq('a', '8'), sq('h', '8'), WR, 0, 0, 0), "a8h8");
    }

    #[test]
    fn identity_stability() {
        let mv = Move::new(sq('e', '2'), sq('e', '4'), WP, 0, 0, 0);
        let s = mv.to_string();

        assert_eq!(s, "e2e4");
        assert_eq!(mv.to_string(), s);
    }
}
