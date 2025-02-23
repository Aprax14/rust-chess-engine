use crate::components::{
    board::Board,
    castle::CastleSide,
    constants,
    pieces::{Bitboard, Piece, PieceKind},
    position::BBPosition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveKind {
    Standard {
        from: Bitboard,
        to: Bitboard,
    },
    Castle(CastleSide),
    Promote {
        from: Bitboard,
        to: Bitboard,
        to_piece: PieceKind,
    },
}

#[derive(Debug, Clone)]
pub struct Move {
    pub piece: Piece,
    pub action: MoveKind,
}

impl Move {
    pub fn is_promotion(&self) -> bool {
        match self.action {
            MoveKind::Standard { from: _, to } => {
                self.piece.kind == PieceKind::Pawn
                    && (to.bits & constants::EIGHT_ROW != 0 || to.bits & constants::FIRST_ROW != 0)
            }
            MoveKind::Castle(_) => false,
            MoveKind::Promote {
                from: _,
                to: _,
                to_piece: _,
            } => true,
        }
    }

    pub fn is_capture(&self, position: &BBPosition) -> bool {
        match self.action {
            MoveKind::Castle(_) => false,
            MoveKind::Standard { from: _, to }
            | MoveKind::Promote {
                from: _,
                to,
                to_piece: _,
            } => position.occupied_by(self.piece.color.other()) & to != Bitboard::new(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub board: Board,
}

impl Scenario {
    pub fn new(board: Board) -> Self {
        Self {
            board: board.clone(),
        }
    }
}
