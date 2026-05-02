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
