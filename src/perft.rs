use crate::{board::Board, moves::MoveList};

impl Board {
    pub fn perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut list = MoveList::new();
        self.generate_legal_moves(&mut list);

        if depth == 1 {
            return list.len() as u64;
        }

        let mut nodes = 0u64;
        for &mv in list.as_slice() {
            let undo = self.make_move(mv);
            nodes += self.perft(depth - 1);
            self.unmake_move(mv, undo);
        }

        nodes
    }

    pub fn perft_divide(&mut self, depth: u8) -> u64 {
        let mut list = MoveList::new();
        self.generate_legal_moves(&mut list);

        let mut total = 0u64;
        for &mv in list.as_slice() {
            let undo = self.make_move(mv);
            let nodes = self.perft(depth - 1);
            self.unmake_move(mv, undo);
            println!("{}: {}", mv, nodes);
            total += nodes;
        }

        println!("total: {}", total);
        total
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn perft_startpos_depth1() {
        let mut board = Board::startpos();
        assert_eq!(board.perft(1), 20);
    }

    #[test]
    fn perft_startpos_depth2() {
        let mut board = Board::startpos();
        assert_eq!(board.perft(2), 400);
    }

    #[test]
    fn perft_startpos_depth3() {
        let mut board = Board::startpos();
        assert_eq!(board.perft(3), 8902);
    }

    #[test]
    fn perft_startpos_depth4() {
        let mut board = Board::startpos();
        assert_eq!(board.perft(4), 197281);
    }

    #[test]
    fn perft_startpos_depth5() {
        let mut board = Board::startpos();
        assert_eq!(board.perft(5), 4865609);
    }

    #[test]
    fn perft_kiwipete_depth1() {
        let mut board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        assert_eq!(board.perft(1), 48);
    }

    #[test]
    fn perft_kiwipete_depth2() {
        let mut board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        assert_eq!(board.perft(2), 2039);
    }

    #[test]
    fn perft_kiwipete_depth3() {
        let mut board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        assert_eq!(board.perft(3), 97862);
    }
    #[test]
    fn perft_divide_kiwipete_depth3() {
        let mut board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        board.perft_divide(3);
    }

    #[test]
    fn perft_position3_depth1() {
        let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        assert_eq!(board.perft(1), 14);
    }

    #[test]
    fn perft_position3_depth2() {
        let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        assert_eq!(board.perft(2), 191);
    }

    #[test]
    fn perft_position3_depth3() {
        let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        assert_eq!(board.perft(3), 2812);
    }
}
