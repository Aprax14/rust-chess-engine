use std::ops::Deref;

use crate::{moves::generator, utils::static_evaluation::StaticEval};

use super::{
    board::Board,
    piece::{Bitboard, Piece},
};

#[derive(Debug, Clone)]
pub struct Move {
    pub piece: Piece,
    pub from: Bitboard,
    pub to: Bitboard,
}

#[derive(Debug, Clone)]
pub struct PossibleMoves {
    pub from: Bitboard,
    pub to: Vec<Bitboard>,
}

#[derive(Debug, Clone)]
pub struct PiecePossibleMoves {
    pub piece: Piece,
    pub moves: Vec<PossibleMoves>,
}

#[derive(Debug, Clone)]
pub struct MovesByPiece(pub Vec<PiecePossibleMoves>);

impl Deref for MovesByPiece {
    type Target = Vec<PiecePossibleMoves>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub board: Board,
}

impl Scenario {
    pub fn from_board(board: Board) -> Self {
        Self { board }
    }

    pub fn generate_moves(&self, only_captures: bool, captures_first: bool) -> MovesByPiece {
        if captures_first {
            generator::generate_moves_captures_first(&self.board, only_captures)
        } else {
            generator::generate_moves_unordered(&self.board, only_captures)
        }
    }

    pub fn apply_moves(&self, moves_by_piece: MovesByPiece) -> Vec<Self> {
        let mut scenarios = Vec::new();
        for pieces_moves in moves_by_piece.iter() {
            for piece_possible_moves in &pieces_moves.moves {
                let from = &piece_possible_moves.from;
                for to in &piece_possible_moves.to {
                    let new_board = self.board.make_unchecked_move(&Move {
                        piece: pieces_moves.piece,
                        from: *from,
                        to: *to,
                    });
                    scenarios.push(Self::from_board(new_board));
                }
            }
        }

        scenarios
    }
}
