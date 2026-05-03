use crate::{
    board::{
        BLACK, BN, Bitboard, Board, WHITE, WN,
        generation::{NOT_A, NOT_AB, NOT_GH, NOT_H},
    },
    moves::{FLAG_CAPTURE, Move, MoveList},
};

impl Board {
    pub fn generate_knight_moves(&self, list: &mut MoveList) {
        let white = self.side_to_move == WHITE as u8;

        let piece = if white { WN } else { BN };

        let knights = self.pieces[piece as usize - 1];

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

        let mut bb = knights;

        while bb != 0 {
            let from = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let attacks = KNIGHT_ATTACKS[from as usize] & !own;

            let quiet = attacks & !enemy;
            let caps = attacks & enemy;

            self.push_knight_quiets(from, quiet, piece, list);
            self.push_knight_caps(from, caps, piece, list);
        }
    }

    fn push_knight_quiets(&self, from: u8, mut bb: Bitboard, piece: u8, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            list.push(Move::new(from, to, piece, 0, 0, 0));
        }
    }

    fn push_knight_caps(&self, from: u8, mut bb: Bitboard, piece: u8, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let captured = self.mailbox[to as usize];

            list.push(Move::new(from, to, piece, captured, 0, FLAG_CAPTURE));
        }
    }
}

const fn knight_attacks(sq: u8) -> u64 {
    let bb = 1u64 << sq;

    ((bb << 17) & NOT_A)
        | ((bb << 15) & NOT_H)
        | ((bb << 10) & NOT_AB)
        | ((bb << 6) & NOT_GH)
        | ((bb >> 17) & NOT_H)
        | ((bb >> 15) & NOT_A)
        | ((bb >> 10) & NOT_GH)
        | ((bb >> 6) & NOT_AB)
}

const fn build_knight_table() -> [u64; 64] {
    let mut table = [0u64; 64];
    let mut i = 0;

    while i < 64 {
        table[i] = knight_attacks(i as u8);
        i += 1;
    }

    table
}

const KNIGHT_ATTACKS: [u64; 64] = build_knight_table();

#[cfg(test)]
mod tests {
    use crate::{
        board::{BLACK, BN, BP, Board, WHITE, WN, WP},
        moves::{MoveList, sq},
    };

    fn collect(board: &Board) -> Vec<String> {
        let mut list = MoveList::new();
        board.generate_knight_moves(&mut list);

        let mut moves: Vec<String> = list.as_slice().iter().map(|m| m.to_string()).collect();

        moves.sort();
        moves
    }

    fn contains(moves: &[String], mv: &str) {
        assert!(
            moves.contains(&mv.to_string()),
            "missing move {} in {:?}",
            mv,
            moves
        );
    }

    #[test]
    fn startpos_white_knights() {
        let mut board = Board::startpos();
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert_eq!(moves.len(), 4);
        contains(&moves, "b1a3");
        contains(&moves, "b1c3");
        contains(&moves, "g1f3");
        contains(&moves, "g1h3");
    }

    #[test]
    fn startpos_black_knights() {
        let mut board = Board::startpos();
        board.side_to_move = BLACK as u8;

        let moves = collect(&board);

        assert_eq!(moves.len(), 4);
        contains(&moves, "b8a6");
        contains(&moves, "b8c6");
        contains(&moves, "g8f6");
        contains(&moves, "g8h6");
    }

    #[test]
    fn knight_center_has_eight_moves() {
        let mut board = Board::empty();

        board.put_piece(WN, sq('d', '4'));
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert_eq!(moves.len(), 8);

        let expected = [
            "d4b3", "d4b5", "d4c2", "d4c6", "d4e2", "d4e6", "d4f3", "d4f5",
        ];

        for mv in expected {
            contains(&moves, mv);
        }
    }

    #[test]
    fn knight_corner_has_two_moves() {
        let mut board = Board::empty();

        board.put_piece(WN, sq('a', '1'));
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert_eq!(moves.len(), 2);
        contains(&moves, "a1b3");
        contains(&moves, "a1c2");
    }

    #[test]
    fn knight_respects_own_blockers() {
        let mut board = Board::empty();

        board.put_piece(WN, sq('d', '4'));
        board.put_piece(WP, sq('b', '5'));
        board.put_piece(WP, sq('e', '6'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(!moves.contains(&"d4b5".to_string()));
        assert!(!moves.contains(&"d4e6".to_string()));
        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn knight_can_capture_enemy() {
        let mut board = Board::empty();

        board.put_piece(WN, sq('d', '4'));
        board.put_piece(BP, sq('b', '5'));
        board.put_piece(BP, sq('f', '5'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        contains(&moves, "d4b5");
        contains(&moves, "d4f5");
        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn black_knight_can_capture_white() {
        let mut board = Board::empty();

        board.put_piece(BN, sq('e', '5'));
        board.put_piece(WP, sq('d', '3'));
        board.put_piece(WP, sq('f', '3'));

        board.side_to_move = BLACK as u8;

        let moves = collect(&board);

        contains(&moves, "e5d3");
        contains(&moves, "e5f3");
    }

    #[test]
    fn knight_no_wraparound_from_h_file() {
        let mut board = Board::empty();

        board.put_piece(WN, sq('h', '4'));
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert_eq!(moves.len(), 4);

        contains(&moves, "h4f3");
        contains(&moves, "h4f5");
        contains(&moves, "h4g2");
        contains(&moves, "h4g6");
    }

    #[test]
    fn knight_no_wraparound_from_a_file() {
        let mut board = Board::empty();

        board.put_piece(WN, sq('a', '4'));
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert_eq!(moves.len(), 4);

        contains(&moves, "a4b2");
        contains(&moves, "a4b6");
        contains(&moves, "a4c3");
        contains(&moves, "a4c5");
    }
}
