use crate::components::{
    board::Board,
    castle::CastleSide,
    constants,
    pieces::{Piece, PieceKind},
    position::BBPosition,
};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum MoveKind {
    Standard {
        from: u8,
        to: u8,
    },
    Castle(CastleSide),
    Promote {
        from: u8,
        to: u8,
        to_piece: PieceKind,
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
            MoveKind::Standard { from: _, to } => {
                self.piece.kind == PieceKind::Pawn
                    && ((1 << to) & constants::EIGHT_ROW != 0
                        || (1 << to) & constants::FIRST_ROW != 0)
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
            } => position.occupied_by(self.piece.color.other()).bits & (1 << to) != 0,
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
