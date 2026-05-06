use crate::{
    board::{
        BK, BLACK, BOTH, Board, WHITE, WK,
        generation::{
            BK_EMPTY, BQ_EMPTY, CASTLE_BK, CASTLE_BQ, CASTLE_WK, CASTLE_WQ, NOT_A, NOT_H, WK_EMPTY,
            WQ_EMPTY,
        },
    },
    moves::{FLAG_CASTLE, Move, MoveList},
};

impl Board {
    pub fn generate_king_moves(&self, list: &mut MoveList) {
        let white = self.side_to_move == WHITE as u8;

        let piece = if white { WK } else { BK };

        let king = self.pieces[(piece - 1) as usize];

        let own = if white {
            self.occ[WHITE]
        } else {
            self.occ[BLACK]
        };
        let enemy = if white {
            self.occ[BLACK]
        } else {
            self.occ[WHITE]
        };

        let mut bb = king;

        while bb != 0 {
            let from = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let attacks = KING_ATTACKS[from as usize] & !own;

            let quiet = attacks & !enemy;
            let caps = attacks & enemy;

            self.push_quiets(from, quiet, piece, list);
            self.push_caps(from, caps, piece, list);
        }

        self.generate_castling(list);
    }

    pub fn generate_king_captures(&self, list: &mut MoveList) {
        let white = self.side_to_move == WHITE as u8;
        let piece = if white { WK } else { BK };
        let own = if white {
            self.occ[WHITE]
        } else {
            self.occ[BLACK]
        };
        let enemy = if white {
            self.occ[BLACK]
        } else {
            self.occ[WHITE]
        };

        let king = self.pieces[piece as usize - 1];
        if king == 0 {
            return;
        }
        let from = king.trailing_zeros() as u8;
        let caps = KING_ATTACKS[from as usize] & !own & enemy;
        self.push_caps(from, caps, piece, list);
    }

    pub fn generate_castling(&self, list: &mut MoveList) {
        let white = self.side_to_move == WHITE as u8;
        let occ = self.occ[BOTH];

        if white {
            if self.castling & CASTLE_WK != 0 && occ & WK_EMPTY == 0 {
                list.push(Move::new(4, 6, WK, 0, 0, FLAG_CASTLE));
            }
            if self.castling & CASTLE_WQ != 0 && occ & WQ_EMPTY == 0 {
                list.push(Move::new(4, 2, WK, 0, 0, FLAG_CASTLE));
            }
        } else {
            if self.castling & CASTLE_BK != 0 && occ & BK_EMPTY == 0 {
                list.push(Move::new(60, 62, BK, 0, 0, FLAG_CASTLE));
            }
            if self.castling & CASTLE_BQ != 0 && occ & BQ_EMPTY == 0 {
                list.push(Move::new(60, 58, BK, 0, 0, FLAG_CASTLE));
            }
        }
    }
}

pub const KING_ATTACKS: [u64; 64] = build_king_table();

const fn build_king_table() -> [u64; 64] {
    let mut table = [0u64; 64];
    let mut i = 0;

    while i < 64 {
        table[i] = king_attacks(i as u8);
        i += 1;
    }

    table
}

const fn king_attacks(sq: u8) -> u64 {
    let bb = 1u64 << sq;

    ((bb << 1) & NOT_A)
        | ((bb >> 1) & NOT_H)
        | (bb << 8)
        | (bb >> 8)
        | ((bb << 7) & NOT_H)
        | ((bb << 9) & NOT_A)
        | ((bb >> 7) & NOT_A)
        | ((bb >> 9) & NOT_H)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::{BP, WP},
        moves::sq,
    };

    fn collect(board: &Board) -> Vec<String> {
        let mut list = MoveList::new();
        board.generate_king_moves(&mut list);

        let mut moves: Vec<String> = list.as_slice().iter().map(|m| m.to_string()).collect();

        moves.sort();
        moves
    }

    #[test]
    fn king_center_full_mobility() {
        let mut board = Board::empty();

        board.put_piece(WK, sq('e', '4'));
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        let expected = vec![
            "e4d3", "e4d4", "e4d5", "e4e3", "e4e5", "e4f3", "e4f4", "e4f5",
        ];

        assert_eq!(moves.len(), 8);
        for m in expected {
            assert!(moves.contains(&m.to_string()));
        }
    }

    #[test]
    fn king_corner_a1() {
        let mut board = Board::empty();

        board.put_piece(WK, sq('a', '1'));
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        let expected = vec!["a1a2", "a1b1", "a1b2"];

        assert_eq!(moves.len(), 3);
        for m in expected {
            assert!(moves.contains(&m.to_string()));
        }
    }

    #[test]
    fn king_corner_h8() {
        let mut board = Board::empty();

        board.put_piece(WK, sq('h', '8'));
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        let expected = vec!["h8g7", "h8g8", "h8h7"];

        assert_eq!(
            moves.len(),
            3,
            "found: {:?}, expected: {:?}",
            moves,
            expected
        );
        for m in expected {
            assert!(moves.contains(&m.to_string()));
        }
    }

    #[test]
    fn king_surrounded_by_own_pieces() {
        let mut board = Board::empty();

        board.put_piece(WK, sq('e', '4'));

        board.put_piece(WP, sq('d', '3'));
        board.put_piece(WP, sq('d', '4'));
        board.put_piece(WP, sq('d', '5'));
        board.put_piece(WP, sq('e', '3'));
        board.put_piece(WP, sq('e', '5'));
        board.put_piece(WP, sq('f', '3'));
        board.put_piece(WP, sq('f', '4'));
        board.put_piece(WP, sq('f', '5'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(moves.is_empty());
    }

    #[test]
    fn king_can_capture() {
        let mut board = Board::empty();

        board.put_piece(WK, sq('e', '4'));
        board.put_piece(BP, sq('e', '5'));
        board.put_piece(BP, sq('f', '4'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(moves.contains(&"e4e5".to_string()));
        assert!(moves.contains(&"e4f4".to_string()));
    }

    #[test]
    fn king_no_file_wrap_bug() {
        let mut board = Board::empty();

        board.put_piece(WK, sq('h', '4'));
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(!moves.contains(&"h4a3".to_string()));
        assert!(!moves.contains(&"h4a4".to_string()));
        assert!(!moves.contains(&"h4a5".to_string()));
    }
}
