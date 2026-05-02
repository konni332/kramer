#[derive(Debug, Clone)]
pub enum FenError {
    FieldCount,
    PiecePlacement,
    SideToMove,
    Castling,
    EnPassant,
    Halfmove,
    Fullmove,
}
