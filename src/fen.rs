use crate::{board::*, error::FenError};

fn piece_from_char(ch: char) -> Option<u8> {
    Some(match ch {
        'P' => WP,
        'N' => WN,
        'B' => WB,
        'R' => WR,
        'Q' => WQ,
        'K' => WK,

        'p' => BP,
        'n' => BN,
        'b' => BB,
        'r' => BR,
        'q' => BQ,
        'k' => BK,

        _ => return None,
    })
}

fn piece_to_char(piece: u8) -> char {
    match piece {
        WP => 'P',
        WN => 'N',
        WB => 'B',
        WR => 'R',
        WQ => 'Q',
        WK => 'K',

        BP => 'p',
        BN => 'n',
        BB => 'b',
        BR => 'r',
        BQ => 'q',
        BK => 'k',

        _ => unreachable!(),
    }
}

pub fn parse_square(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();

    if bytes.len() != 2 {
        return None;
    }

    let file = bytes[0];
    let rank = bytes[1];

    if !(b'a'..=b'h').contains(&file) {
        return None;
    }

    if !(b'1'..=b'8').contains(&rank) {
        return None;
    }

    let f = file - b'a';
    let r = rank - b'1';

    Some(r * 8 + f)
}

fn push_square(out: &mut String, sq: Square) {
    let file = sq % 8;
    let rank = sq / 8;

    out.push((b'a' + file) as char);
    out.push((b'1' + rank) as char);
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let mut board = Self::empty();

        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(FenError::FieldCount);
        }

        let mut rank: i32 = 7;
        let mut file: i32 = 0;

        for ch in parts[0].chars() {
            match ch {
                '/' => {
                    if file != 8 {
                        return Err(FenError::PiecePlacement);
                    }

                    rank -= 1;
                    file = 0;

                    if rank < 0 {
                        return Err(FenError::PiecePlacement);
                    }
                }
                '1'..='8' => {
                    file += ch.to_digit(10).unwrap() as i32;
                }

                _ => {
                    let piece = piece_from_char(ch).ok_or(FenError::PiecePlacement)?;

                    if file >= 8 {
                        return Err(FenError::PiecePlacement);
                    }

                    let sq = (rank * 8 + file) as Square;
                    board.put_piece(piece, sq);

                    file += 1;
                }
            }
        }

        if rank != 0 || file != 8 {
            return Err(FenError::PiecePlacement);
        }

        board.side_to_move = match parts[1] {
            "w" => WHITE as u8,
            "b" => BLACK as u8,
            _ => return Err(FenError::SideToMove),
        };

        board.castling = 0;

        if parts[2] != "-" {
            for ch in parts[2].chars() {
                match ch {
                    'K' => board.castling |= 1,
                    'Q' => board.castling |= 2,
                    'k' => board.castling |= 4,
                    'q' => board.castling |= 8,
                    _ => return Err(FenError::Castling),
                }
            }
        }

        board.ep_square = if parts[3] == "-" {
            64
        } else {
            parse_square(parts[3]).ok_or(FenError::EnPassant)?
        };

        board.halfmove_clock = parts[4].parse().map_err(|_| FenError::Halfmove)?;

        board.fullmove_number = parts[5].parse().map_err(|_| FenError::Fullmove)?;

        Ok(board)
    }

    pub fn to_fen(self) -> String {
        let mut out = String::with_capacity(96);

        for rank in (0..8).rev() {
            let mut empty_run = 0;

            for file in 0..8 {
                let sq = rank * 8 + file;
                let piece = self.mailbox[sq];

                if piece == EMPTY {
                    empty_run += 1;
                } else {
                    if empty_run != 0 {
                        out.push(char::from(b'0' + empty_run as u8));
                        empty_run = 0;
                    }

                    out.push(piece_to_char(piece));
                }
            }

            if empty_run != 0 {
                out.push(char::from(b'0' + empty_run as u8));
            }

            if rank != 0 {
                out.push('/');
            }
        }

        out.push(' ');
        out.push(if self.side_to_move == WHITE as u8 {
            'w'
        } else {
            'b'
        });

        out.push(' ');

        if self.castling == 0 {
            out.push('-');
        } else {
            if self.castling & 1 != 0 {
                out.push('K');
            }
            if self.castling & 2 != 0 {
                out.push('Q');
            }
            if self.castling & 4 != 0 {
                out.push('k');
            }
            if self.castling & 8 != 0 {
                out.push('q');
            }
        }

        out.push(' ');

        if self.ep_square == 64 {
            out.push('-');
        } else {
            push_square(&mut out, self.ep_square);
        }

        out.push(' ');
        out.push_str(&self.halfmove_clock.to_string());

        out.push(' ');
        out.push_str(&self.fullmove_number.to_string());

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(fen: &str) {
        let board = Board::from_fen(fen)
            .unwrap_or_else(|e| panic!("failed to parse FEN:\n{fen}\nerror: {e:?}"));

        let out = board.to_fen();

        assert_eq!(
            out, fen,
            "\nroundtrip mismatch\ninput : {fen}\noutput: {out}\n"
        );
    }

    #[test]
    fn roundtrip_startpos() {
        roundtrip("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }

    #[test]
    fn roundtrip_empty_board() {
        roundtrip("8/8/8/8/8/8/8/8 w - - 0 1");
    }

    #[test]
    fn roundtrip_kings_only() {
        roundtrip("4k3/8/8/8/8/8/8/4K3 w - - 0 1");
    }

    #[test]
    fn roundtrip_castling_all_rights() {
        roundtrip("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    }

    #[test]
    fn roundtrip_castling_no_rights() {
        roundtrip("r3k2r/8/8/8/8/8/8/R3K2R b - - 17 42");
    }

    #[test]
    fn roundtrip_en_passant_white_can_capture() {
        roundtrip("rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 2");
    }

    #[test]
    fn roundtrip_en_passant_black_can_capture() {
        roundtrip("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2");
    }

    #[test]
    fn roundtrip_mixed_midgame() {
        roundtrip("r2q1rk1/pp2bppp/2np1n2/2p1p3/2P1P3/2NP1N2/PPQ1BPPP/R1B2RK1 w - - 3 10");
    }

    #[test]
    fn roundtrip_promotions_present() {
        roundtrip("Q6k/8/8/8/8/8/8/K6q w - - 0 1");
    }

    #[test]
    fn roundtrip_sparse_position() {
        roundtrip("8/3k4/8/8/2Q5/8/4K3/8 b - - 99 150");
    }

    #[test]
    fn startpos_constructor_matches_fen() {
        let board = Board::startpos();

        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn default_is_startpos() {
        let board = Board::default();

        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }
}
