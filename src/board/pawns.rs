use crate::{
    board::{
        BB, BLACK, BN, BOTH, BP, BQ, BR, Bitboard, Board, WB, WHITE, WN, WP, WQ, WR,
        generation::{FILE_A, FILE_H, PROMO_RANK_BLACK, PROMO_RANK_WHITE, RANK_2, RANK_7},
    },
    moves::{FLAG_CAPTURE, FLAG_DOUBLE, FLAG_EP, FLAG_PROMOTION, Move, MoveList},
};

impl Board {
    pub fn generate_pawn_moves(&self, list: &mut MoveList) {
        let white = self.side_to_move == WHITE as u8;

        let pawns = self.pieces[(if white { WP } else { BP }) as usize - 1];

        let empty = !self.occ[BOTH];
        let enemy = if white {
            self.occ[BLACK]
        } else {
            self.occ[WHITE]
        };

        if white {
            self.gen_white_pawns(pawns, empty, enemy, list);
        } else {
            self.gen_black_pawns(pawns, empty, enemy, list);
        }
    }

    fn gen_white_pawns(
        &self,
        pawns: Bitboard,
        empty: Bitboard,
        enemy: Bitboard,
        list: &mut MoveList,
    ) {
        let single = (pawns << 8) & empty;

        let double = ((((pawns & RANK_2) << 8) & empty) << 8) & empty;

        let promo_push = single & PROMO_RANK_WHITE;
        let normal_push = single & !PROMO_RANK_WHITE;

        self.push_white_pushes(normal_push, list);
        self.push_white_promotions(promo_push, list);
        self.push_white_doubles(double, list);

        let left = ((pawns & !FILE_A) << 7) & enemy;
        let right = ((pawns & !FILE_H) << 9) & enemy;

        self.push_white_caps_left(left, list);
        self.push_white_caps_right(right, list);

        if self.ep_square != 64 {
            let ep = self.ep_square;
            let ep_bb = 1u64 << ep;

            let left = ((pawns & !FILE_A) << 7) & ep_bb;
            let right = ((pawns & !FILE_H) << 9) & ep_bb;

            if left != 0 {
                let from = ep - 7;
                list.push(Move::new(from, ep, WP, BP, 0, FLAG_CAPTURE | FLAG_EP));
            }
            if right != 0 {
                let from = ep - 9;
                list.push(Move::new(from, ep, WP, BP, 0, FLAG_CAPTURE | FLAG_EP));
            }
        }
    }

    fn gen_black_pawns(
        &self,
        pawns: Bitboard,
        empty: Bitboard,
        enemy: Bitboard,
        list: &mut MoveList,
    ) {
        let single = (pawns >> 8) & empty;

        let double = ((((pawns & RANK_7) >> 8) & empty) >> 8) & empty;

        let promo = single & PROMO_RANK_BLACK;
        let normal = single & !PROMO_RANK_BLACK;

        self.push_black_pushes(normal, list);
        self.push_black_promotions(promo, list);
        self.push_black_doubles(double, list);

        let left = ((pawns & !FILE_A) >> 9) & enemy;
        let right = ((pawns & !FILE_H) >> 7) & enemy;

        self.push_black_caps_left(left, list);
        self.push_black_caps_right(right, list);

        if self.ep_square != 64 {
            let ep = self.ep_square;
            let ep_bb = 1u64 << ep;

            let left = ((pawns & !FILE_A) >> 9) & ep_bb;
            let right = ((pawns & !FILE_H) >> 7) & ep_bb;

            if left != 0 {
                let from = ep + 9;
                list.push(Move::new(from, ep, BP, WP, 0, FLAG_CAPTURE | FLAG_EP));
            }
            if right != 0 {
                let from = ep + 7;
                list.push(Move::new(from, ep, BP, WP, 0, FLAG_CAPTURE | FLAG_EP));
            }
        }
    }

    fn push_white_pushes(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to - 8;

            list.push(Move::new(from, to, WP, 0, 0, 0));
        }
    }
    fn push_white_promotions(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to - 8;

            list.push(Move::new(from, to, WP, 0, WQ, FLAG_PROMOTION));
            list.push(Move::new(from, to, WP, 0, WR, FLAG_PROMOTION));
            list.push(Move::new(from, to, WP, 0, WB, FLAG_PROMOTION));
            list.push(Move::new(from, to, WP, 0, WN, FLAG_PROMOTION));
        }
    }
    fn push_black_pushes(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to + 8;

            list.push(Move::new(from, to, BP, 0, 0, 0));
        }
    }
    fn push_black_promotions(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to + 8;

            list.push(Move::new(from, to, BP, 0, BQ, FLAG_PROMOTION));
            list.push(Move::new(from, to, BP, 0, BR, FLAG_PROMOTION));
            list.push(Move::new(from, to, BP, 0, BB, FLAG_PROMOTION));
            list.push(Move::new(from, to, BP, 0, BN, FLAG_PROMOTION));
        }
    }
    fn push_white_doubles(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to - 16;

            list.push(Move::new(from, to, WP, 0, 0, FLAG_DOUBLE));
        }
    }
    fn push_black_doubles(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to + 16;

            list.push(Move::new(from, to, BP, 0, 0, FLAG_DOUBLE));
        }
    }

    fn push_white_caps_left(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to - 7;
            let cap = self.mailbox[to as usize];

            if to >= 56 {
                self.push_white_promo_caps(from, to, cap, list);
            } else {
                list.push(Move::new(from, to, WP, cap, 0, FLAG_CAPTURE));
            }
        }
    }

    fn push_white_caps_right(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to - 9;
            let cap = self.mailbox[to as usize];

            if to >= 56 {
                self.push_white_promo_caps(from, to, cap, list);
            } else {
                list.push(Move::new(from, to, WP, cap, 0, FLAG_CAPTURE));
            }
        }
    }

    fn push_white_promo_caps(&self, from: u8, to: u8, cap: u8, list: &mut MoveList) {
        let flags = FLAG_CAPTURE | FLAG_PROMOTION;

        list.push(Move::new(from, to, WP, cap, WQ, flags));
        list.push(Move::new(from, to, WP, cap, WR, flags));
        list.push(Move::new(from, to, WP, cap, WB, flags));
        list.push(Move::new(from, to, WP, cap, WN, flags));
    }

    fn push_black_caps_left(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to + 9;
            let cap = self.mailbox[to as usize];

            if to <= 7 {
                self.push_black_promo_caps(from, to, cap, list);
            } else {
                list.push(Move::new(from, to, BP, cap, 0, FLAG_CAPTURE));
            }
        }
    }

    fn push_black_caps_right(&self, mut bb: Bitboard, list: &mut MoveList) {
        while bb != 0 {
            let to = bb.trailing_zeros() as u8;
            bb &= bb - 1;

            let from = to + 7;
            let cap = self.mailbox[to as usize];

            if to <= 7 {
                self.push_black_promo_caps(from, to, cap, list);
            } else {
                list.push(Move::new(from, to, BP, cap, 0, FLAG_CAPTURE));
            }
        }
    }

    fn push_black_promo_caps(&self, from: u8, to: u8, cap: u8, list: &mut MoveList) {
        let flags = FLAG_CAPTURE | FLAG_PROMOTION;

        list.push(Move::new(from, to, BP, cap, BQ, flags));
        list.push(Move::new(from, to, BP, cap, BR, flags));
        list.push(Move::new(from, to, BP, cap, BB, flags));
        list.push(Move::new(from, to, BP, cap, BN, flags));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::{MoveList, sq};

    fn collect(board: &Board) -> Vec<String> {
        let mut list = MoveList::new();
        board.generate_pawn_moves(&mut list);

        let mut moves: Vec<String> = list.as_slice().iter().map(|m| m.to_string()).collect();

        moves.sort();
        moves
    }

    fn assert_contains(moves: &[String], expected: &str) {
        assert!(
            moves.contains(&expected.to_string()),
            "missing move: {}",
            expected
        );
    }

    #[test]
    fn startpos_white_pawns() {
        let board = Board::startpos();

        // force white move for deterministic test
        let mut board = board;
        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        let expected = [
            "a2a3", "a2a4", "b2b3", "b2b4", "c2c3", "c2c4", "d2d3", "d2d4", "e2e3", "e2e4", "f2f3",
            "f2f4", "g2g3", "g2g4", "h2h3", "h2h4",
        ];

        assert_eq!(moves.len(), 16, "expected: {:?} got: {:?}", expected, moves);

        for mv in expected {
            assert_contains(&moves, mv);
        }
    }
    #[test]
    fn startpos_black_pawns() {
        let mut board = Board::startpos();
        board.side_to_move = BLACK as u8;

        let moves = collect(&board);

        assert_eq!(moves.len(), 16, "{:?}", moves);

        assert_contains(&moves, "a7a6");
        assert_contains(&moves, "a7a5");
        assert_contains(&moves, "h7h6");
        assert_contains(&moves, "h7h5");
    }

    #[test]
    fn pawn_blocked_no_double() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('e', '2'));
        board.put_piece(BP, sq('e', '3')); // block

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(moves.is_empty());
    }
    #[test]
    fn pawn_captures_basic() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('e', '4'));
        board.put_piece(BP, sq('d', '5'));
        board.put_piece(BP, sq('f', '5'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(moves.contains(&"e4d5".to_string()));
        assert!(moves.contains(&"e4f5".to_string()));
        assert!(moves.contains(&"e4e5".to_string())); // forward push
    }

    #[test]
    fn pawn_file_masking() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('a', '4'));
        board.put_piece(BP, sq('h', '5'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        // a-pawn must NOT generate capture to h-file via wrap
        assert!(!moves.contains(&"a4b5".to_string())); // only valid capture would be b5 if piece exists
    }

    #[test]
    fn pawn_promotion_white() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('e', '7'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(moves.contains(&"e7e8q".to_string()));
        assert!(moves.contains(&"e7e8r".to_string()));
        assert!(moves.contains(&"e7e8b".to_string()));
        assert!(moves.contains(&"e7e8n".to_string()));
    }
    #[test]
    fn pawn_promotion_capture() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('e', '7'));
        board.put_piece(BP, sq('d', '8'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(moves.contains(&"e7d8q".to_string()));
    }
    #[test]
    fn pawn_double_push_only_from_start() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('a', '3')); // NOT rank 2

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert_eq!(moves, vec!["a3a4"]);
    }
    #[test]
    fn pawn_symmetry_white_black() {
        let mut b1 = Board::startpos();
        b1.side_to_move = WHITE as u8;

        let mut b2 = Board::startpos();
        b2.side_to_move = BLACK as u8;

        let w = collect(&b1);
        let b = collect(&b2);

        assert_eq!(w.len(), b.len());
    }
    #[test]
    fn pawn_double_blocked_on_target_square() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('e', '2'));
        board.put_piece(BP, sq('e', '4'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert_eq!(moves, vec!["e2e3"]);
    }
    #[test]
    fn pawn_captures_when_forward_blocked() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('e', '4'));
        board.put_piece(BP, sq('e', '5'));
        board.put_piece(BP, sq('d', '5'));
        board.put_piece(BP, sq('f', '5'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(moves.contains(&"e4d5".to_string()));
        assert!(moves.contains(&"e4f5".to_string()));
        assert!(!moves.contains(&"e4e5".to_string()));
    }
    #[test]
    fn pawn_promotion_blocked_forward() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('e', '7'));
        board.put_piece(BP, sq('e', '8'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(!moves.contains(&"e7e8q".to_string()));
        assert!(moves.is_empty());
    }
    #[test]
    fn pawn_promotion_capture_all_pieces() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('e', '7'));
        board.put_piece(BP, sq('d', '8'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(moves.contains(&"e7d8q".to_string()));
        assert!(moves.contains(&"e7d8r".to_string()));
        assert!(moves.contains(&"e7d8b".to_string()));
        assert!(moves.contains(&"e7d8n".to_string()));
    }
    #[test]
    fn black_promotion_push() {
        let mut board = Board::empty();

        board.put_piece(BP, sq('e', '2'));
        board.side_to_move = BLACK as u8;

        let moves = collect(&board);

        assert!(moves.contains(&"e2e1q".to_string()));
        assert!(moves.contains(&"e2e1r".to_string()));
        assert!(moves.contains(&"e2e1b".to_string()));
        assert!(moves.contains(&"e2e1n".to_string()));
    }
    #[test]
    fn black_promotion_capture() {
        let mut board = Board::empty();

        board.put_piece(BP, sq('e', '2'));
        board.put_piece(WP, sq('f', '1'));

        board.side_to_move = BLACK as u8;

        let moves = collect(&board);

        assert!(moves.contains(&"e2f1q".to_string()));
    }
    #[test]
    fn pawn_cannot_capture_own_piece() {
        let mut board = Board::empty();

        board.put_piece(WP, sq('e', '4'));
        board.put_piece(WP, sq('d', '5'));

        board.side_to_move = WHITE as u8;

        let moves = collect(&board);

        assert!(!moves.contains(&"e4d5".to_string()));
    }
    #[test]
    fn white_ep_capture_left() {
        let mut board = Board::empty();
        board.put_piece(WP, sq('e', '5'));
        board.put_piece(BP, sq('d', '5'));
        board.side_to_move = WHITE as u8;
        board.ep_square = sq('d', '6');

        let moves = collect(&board);

        assert_contains(&moves, "e5d6");
    }

    #[test]
    fn white_ep_capture_right() {
        let mut board = Board::empty();
        board.put_piece(WP, sq('e', '5'));
        board.put_piece(BP, sq('f', '5'));
        board.side_to_move = WHITE as u8;
        board.ep_square = sq('f', '6');

        let moves = collect(&board);

        assert_contains(&moves, "e5f6");
    }

    #[test]
    fn white_ep_both_sides() {
        let mut board = Board::empty();
        board.put_piece(WP, sq('e', '5'));
        board.put_piece(BP, sq('d', '5'));
        board.put_piece(BP, sq('f', '5'));
        board.side_to_move = WHITE as u8;
        board.ep_square = sq('f', '6');

        let moves = collect(&board);

        // only right ep is valid for this ep_square
        assert_contains(&moves, "e5f6");
        assert!(!moves.contains(&"e5d6".to_string()));
    }

    #[test]
    fn black_ep_capture_left() {
        let mut board = Board::empty();
        board.put_piece(BP, sq('e', '4'));
        board.put_piece(WP, sq('d', '4'));
        board.side_to_move = BLACK as u8;
        board.ep_square = sq('d', '3');

        let moves = collect(&board);

        assert_contains(&moves, "e4d3");
    }

    #[test]
    fn black_ep_capture_right() {
        let mut board = Board::empty();
        board.put_piece(BP, sq('e', '4'));
        board.put_piece(WP, sq('f', '4'));
        board.side_to_move = BLACK as u8;
        board.ep_square = sq('f', '3');

        let moves = collect(&board);

        assert_contains(&moves, "e4f3");
    }

    #[test]
    fn ep_not_generated_when_none() {
        let mut board = Board::empty();
        board.put_piece(WP, sq('e', '5'));
        board.put_piece(BP, sq('d', '5'));
        board.side_to_move = WHITE as u8;
        // ep_square defaults to 64 = none

        let moves = collect(&board);

        assert!(!moves.contains(&"e5d6".to_string()));
    }

    #[test]
    fn ep_no_wrap_a_file() {
        // black pawn on a4, white pawn doubled to h4 — ep square on h3
        // a-file pawn must not wrap around and generate ep capture
        let mut board = Board::empty();
        board.put_piece(BP, sq('a', '4'));
        board.put_piece(WP, sq('h', '4'));
        board.side_to_move = BLACK as u8;
        board.ep_square = sq('h', '3');

        let moves = collect(&board);

        assert!(!moves.contains(&"a4h3".to_string()));
    }

    #[test]
    fn ep_no_wrap_h_file() {
        // white pawn on h5, black pawn doubled to a5 — ep square on a6
        let mut board = Board::empty();
        board.put_piece(WP, sq('h', '5'));
        board.put_piece(BP, sq('a', '5'));
        board.side_to_move = WHITE as u8;
        board.ep_square = sq('a', '6');

        let moves = collect(&board);

        assert!(!moves.contains(&"h5a6".to_string()));
    }
}
