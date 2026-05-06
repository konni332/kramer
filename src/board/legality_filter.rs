use kramer_magic_bb::{bishop_attacks, rook_attacks};

use crate::{
    board::{
        BB, BK, BLACK, BN, BOTH, BP, BQ, BR, Board, WB, WHITE, WK, WN, WP, WQ, WR,
        generation::{NOT_A, NOT_H},
        king::KING_ATTACKS,
        knight::KNIGHT_ATTACKS,
    },
    moves::MoveList,
};

impl Board {
    pub fn square_attacked(&self, sq: u8, by_side: usize) -> bool {
        let occ = self.occ[BOTH];

        // by pawns
        if by_side == WHITE {
            let wp = self.pieces[WP as usize - 1];
            let bit = 1u64 << sq;
            let from_right = (bit >> 7) & NOT_A;
            let from_left = (bit >> 9) & NOT_H;
            if (from_right | from_left) & wp != 0 {
                return true;
            }
        } else {
            let bp = self.pieces[BP as usize - 1];
            let bit = 1u64 << sq;
            // pawn on sq+7 attacks sq if that pawn is not on file H (would wrap)
            // pawn on sq+9 attacks sq if that pawn is not on file A (would wrap)
            let from_right = (bit << 7) & NOT_H; // potential pawn square to the right
            let from_left = (bit << 9) & NOT_A; // potential pawn square to the left
            if (from_right | from_left) & bp != 0 {
                return true;
            }
        }

        let knights = if by_side == WHITE {
            self.pieces[WN as usize - 1]
        } else {
            self.pieces[BN as usize - 1]
        };
        if KNIGHT_ATTACKS[sq as usize] & knights != 0 {
            return true;
        }

        let bishops_queens = if by_side == WHITE {
            self.pieces[WB as usize - 1] | self.pieces[WQ as usize - 1]
        } else {
            self.pieces[BB as usize - 1] | self.pieces[BQ as usize - 1]
        };
        if bishop_attacks(sq as usize, occ) & bishops_queens != 0 {
            return true;
        }

        let rooks_queens = if by_side == WHITE {
            self.pieces[WR as usize - 1] | self.pieces[WQ as usize - 1]
        } else {
            self.pieces[BR as usize - 1] | self.pieces[BQ as usize - 1]
        };
        if rook_attacks(sq as usize, occ) & rooks_queens != 0 {
            return true;
        }

        let king = if by_side == WHITE {
            self.pieces[WK as usize - 1]
        } else {
            self.pieces[BK as usize - 1]
        };
        if KING_ATTACKS[sq as usize] & king != 0 {
            return true;
        }

        false
    }
    pub fn king_in_check(&self, side: usize) -> bool {
        let king = if side == WHITE {
            self.pieces[WK as usize - 1]
        } else {
            self.pieces[BK as usize - 1]
        };
        let king_sq = king.trailing_zeros() as u8;
        let opponent = if side == WHITE { BLACK } else { WHITE };
        self.square_attacked(king_sq, opponent)
    }

    pub fn generate_legal_moves(&mut self, list: &mut MoveList) {
        let mut pseudo = MoveList::new();
        self.generate_all_moves(&mut pseudo);

        let side = self.side_to_move as usize;
        let opponent = if side == WHITE { BLACK } else { WHITE };

        for &mv in pseudo.as_slice() {
            if mv.is_castle() {
                let transit_sq = match mv.to() {
                    6 => 5,
                    2 => 3,
                    62 => 61,
                    58 => 59,
                    _ => unreachable!(),
                };
                if self.square_attacked(mv.from(), opponent) {
                    continue;
                }
                if self.square_attacked(transit_sq, opponent) {
                    continue;
                }
            }

            let undo = self.make_move(mv);
            if !self.king_in_check(side) {
                list.push(mv);
            }
            self.unmake_move(mv, undo);
        }
    }

    pub fn generate_legal_captures(&mut self, list: &mut MoveList) {
        let mut pseudo = MoveList::new();
        self.generate_capture_moves(&mut pseudo);

        let side = self.side_to_move as usize;

        for &mv in pseudo.as_slice() {
            let undo = self.make_move(mv);
            if !self.king_in_check(side) {
                list.push(mv);
            }
            self.unmake_move(mv, undo);
            // board is copied, so no unmake needed
        }
    }

    pub fn generate_capture_moves(&self, list: &mut MoveList) {
        self.generate_pawn_captures(list);
        self.generate_knight_captures(list);
        self.generate_bishop_captures(list);
        self.generate_rook_captures(list);
        self.generate_queen_captures(list);
        self.generate_king_captures(list);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_white_not_in_check() {
        let board = Board::startpos();
        assert!(!board.king_in_check(WHITE));
    }

    #[test]
    fn startpos_black_not_in_check() {
        let board = Board::startpos();
        assert!(!board.king_in_check(BLACK));
    }

    #[test]
    fn scholar_mate_black_in_check() {
        // Qxf7# — black king on e8 is in check from queen on f7
        let board =
            Board::from_fen("rnb1kbnr/pppp1Qpp/4p3/8/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4")
                .unwrap();
        assert!(board.king_in_check(BLACK));
    }

    #[test]
    fn white_in_check_from_rook() {
        // black rook on e1, white king on e1 — wait, they cant share
        // black rook on e8, white king on e1, open file
        let board = Board::from_fen("4r3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(board.king_in_check(WHITE));
    }

    #[test]
    fn white_in_check_from_bishop() {
        let board = Board::from_fen("8/8/8/8/5b2/8/8/2K5 w - - 0 1").unwrap();
        assert!(board.king_in_check(WHITE));
    }

    #[test]
    fn white_in_check_from_knight() {
        // knight on d3 attacks e1
        let board = Board::from_fen("8/8/8/8/8/3n4/8/4K3 w - - 0 1").unwrap();
        assert!(board.king_in_check(WHITE));
    }

    #[test]
    fn white_in_check_from_pawn() {
        // black pawn on d2 attacks e1
        let board = Board::from_fen("8/8/8/8/8/8/3p4/4K3 w - - 0 1").unwrap();
        assert!(board.king_in_check(WHITE));
    }

    #[test]
    fn black_in_check_from_pawn() {
        // white pawn on d7 attacks e8
        let board = Board::from_fen("4k3/3P4/8/8/8/8/8/4K3 b - - 0 1").unwrap();
        assert!(board.king_in_check(BLACK));
    }

    #[test]
    fn white_in_check_from_queen() {
        let board = Board::from_fen("8/8/8/8/8/8/5q2/4K3 w - - 0 1").unwrap();
        assert!(board.king_in_check(WHITE));
    }

    #[test]
    fn not_in_check_blocked_rook() {
        // rook on e8 but pawn on e4 blocks
        let board = Board::from_fen("4r3/8/8/8/4P3/8/8/4K3 w - - 0 1").unwrap();
        assert!(!board.king_in_check(WHITE));
    }

    #[test]
    fn not_in_check_blocked_bishop() {
        // bishop on h4 but pawn on f2 blocks diagonal to e1
        let board = Board::from_fen("8/8/8/8/7b/8/5P2/4K3 w - - 0 1").unwrap();
        assert!(!board.king_in_check(WHITE));
    }

    #[test]
    fn square_attacked_by_white_pawn() {
        let board = Board::from_fen("8/8/8/8/8/3P4/8/8 w - - 0 1").unwrap();
        // white pawn on d3 attacks e4 and c4
        assert!(board.square_attacked(28, WHITE)); // e4
        assert!(board.square_attacked(26, WHITE)); // c4
        assert!(!board.square_attacked(27, WHITE)); // d4 — directly in front, not attacked
    }

    #[test]
    fn square_attacked_by_black_pawn() {
        let board = Board::from_fen("8/8/8/3p4/8/8/8/8 b - - 0 1").unwrap();
        // black pawn on d5 attacks e4 and c4
        assert!(board.square_attacked(28, BLACK)); // e4
        assert!(board.square_attacked(26, BLACK)); // c4
        assert!(!board.square_attacked(35, BLACK)); // d4 — directly in front, not attacked
    }

    #[test]
    fn legal_moves_startpos_count() {
        let mut board = Board::startpos();
        let mut list = MoveList::new();
        board.generate_legal_moves(&mut list);
        assert_eq!(list.len(), 20);
    }

    #[test]
    fn legal_moves_in_check_limited() {
        // king in check from rook on e8, only legal moves are to escape or block
        let mut board = Board::from_fen("4r3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let mut list = MoveList::new();
        board.generate_legal_moves(&mut list);
        // king must move off the e file — d1, d2, f1, f2 are the only options
        assert!(list.len() > 0);
        for mv in list.as_slice() {
            // every legal move must result in king not being in check
            let mut board2 = board;
            board2.make_move(*mv);
            assert!(
                !board2.king_in_check(WHITE),
                "move {} left king in check",
                mv
            );
        }
    }

    #[test]
    fn legal_moves_pinned_piece_cannot_move() {
        // white rook on e4 is pinned by black rook on e8, white king on e1
        let mut board = Board::from_fen("4r3/8/8/8/4R3/8/8/4K3 w - - 0 1").unwrap();
        let mut list = MoveList::new();
        board.generate_legal_moves(&mut list);
        // pinned rook can only move along the e file
        for mv in list.as_slice() {
            if mv.piece() == WR {
                let to_file = mv.to() % 8;
                assert_eq!(to_file, 4, "pinned rook moved off e file: {}", mv);
            }
        }
    }

    #[test]
    fn legal_moves_checkmate_returns_empty() {
        // fool's mate
        let mut board =
            Board::from_fen("rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
                .unwrap();
        let mut list = MoveList::new();
        board.generate_legal_moves(&mut list);
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn legal_moves_stalemate_returns_empty() {
        // classic stalemate
        let mut board = Board::from_fen("k7/8/1Q6/8/8/8/8/7K b - - 0 1").unwrap();
        let mut list = MoveList::new();
        board.generate_legal_moves(&mut list);
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn castling_not_legal_through_check() {
        // white king cannot castle kingside — f1 is attacked by black rook on f8
        let mut board = Board::from_fen("5r2/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();
        let mut list = MoveList::new();
        board.generate_legal_moves(&mut list);
        let has_castle = list.as_slice().iter().any(|mv| mv.is_castle());
        assert!(!has_castle, "castling through check should not be legal");
    }
}
