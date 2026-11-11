use crate::components::{
    board::Board,
    castle::CastleSide,
    constants,
    pieces::{Piece, PieceKind},
};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum MoveKind {
    Standard {
        from: u8,
        to: u8,
        captured: Option<Piece>,
    },
    Castle(CastleSide),
    Promote {
        from: u8,
        to: u8,
        to_piece: PieceKind,
        captured: Option<Piece>,
    },
    /// En passant capture: the capturing pawn moves from `from` to `to`,
    /// and the captured pawn (sitting one rank behind `to`) is removed.
    EnPassant {
        from: u8,
        to: u8,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub piece: Piece,
    pub action: MoveKind,
}

impl Move {
    pub fn is_promotion(&self) -> bool {
        match self.action {
            MoveKind::Standard { to, .. } => {
                self.piece.kind == PieceKind::Pawn
                    && ((1 << to) & constants::EIGHT_ROW != 0
                        || (1 << to) & constants::FIRST_ROW != 0)
            }
            MoveKind::Castle(_) | MoveKind::EnPassant { .. } => false,
            MoveKind::Promote { .. } => true,
        }
    }

    /// Encodes the move in 14 bits, enough to identify it among the legal moves of a position:
    ///   bits 0-5  -> from square
    ///   bits 6-11 -> to square (for castling: the king's squares)
    ///   bits 12-13 -> promotion piece (0 = queen, 1 = rook, 2 = bishop, 3 = knight)
    ///
    /// Never returns 0 (from and to are always different), so 0 can mean "no move".
    pub fn to_compact(&self) -> u16 {
        let (from, to, promo) = match self.action {
            MoveKind::Standard { from, to, .. } | MoveKind::EnPassant { from, to } => (from, to, 0),
            MoveKind::Castle(side) => {
                let (king_from, king_to, _, _) =
                    Board::castle_piece_squares(self.piece.color, side);
                (king_from, king_to, 0)
            }
            MoveKind::Promote {
                from, to, to_piece, ..
            } => {
                let promo = match to_piece {
                    PieceKind::Rook => 1,
                    PieceKind::Bishop => 2,
                    PieceKind::Knight => 3,
                    _ => 0,
                };
                (from, to, promo)
            }
        };

        from as u16 | (to as u16) << 6 | promo << 12
    }

    pub fn is_capture(&self) -> bool {
        match self.action {
            MoveKind::Castle(_) => false,
            MoveKind::EnPassant { .. } => true,
            MoveKind::Standard { captured, .. } | MoveKind::Promote { captured, .. } => {
                captured.is_some()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub board: Board,
    /// `board.moves_count` when the search started: the root of the search tree.
    root_moves_count: u32,
}

impl Scenario {
    /// Creates a scenario whose search root is the given board.
    pub fn new(board: Board) -> Self {
        Self {
            root_moves_count: board.moves_count,
            board,
        }
    }

    /// Number of plies (half-moves, null moves included) played since the root of the search.
    /// Used to score mates by distance: a closer mate is worth more than a farther one.
    pub fn plies_from_root(&self) -> i32 {
        (self.board.moves_count - self.root_moves_count) as i32
    }
}
