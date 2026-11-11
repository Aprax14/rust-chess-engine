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
