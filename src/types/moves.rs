use crate::moves::generator;

use super::{
    board::Board,
    constants::{EIGHT_ROW, FIRST_ROW},
    piece::{self, Bitboard, Color, Piece},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub piece: Piece,
    pub from: Bitboard,
    pub to: Bitboard,
}

impl Move {
    /// Returns the Bitboard of the promotion Square.
    ///
    /// Returns Bitboard 0 if there is no promotion going.
    pub fn promotion_square(&self) -> Bitboard {
        match (self.piece.kind, self.piece.color) {
            (piece::Kind::Pawn, Color::White) => Bitboard {
                bits: self.to.bits & EIGHT_ROW,
            },
            (piece::Kind::Pawn, Color::Black) => Bitboard {
                bits: self.to.bits & FIRST_ROW,
            },
            _ => Bitboard { bits: 0 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub board: Board,
}

impl Scenario {
    pub fn from_board(board: &Board) -> Self {
        Self {
            board: board.clone(),
        }
    }

    pub fn generate_moves(&self, only_critical: bool, current_pv: &Vec<Move>) -> Vec<Move> {
        generator::generate_moves_ordered(&self.board, only_critical, current_pv)
    }

    pub fn apply_moves(&self, moves: Vec<Move>) -> Vec<(Move, Self)> {
        let mut scenarios = Vec::new();
        for piece_move in moves.into_iter() {
            let new_board = self.board.make_unchecked_move(&piece_move);
            if new_board.position.is_in_check(piece_move.piece.color) {
                // discard position, is not legal
                continue;
            }
            let promotions = new_board.generate_promotion_variants();
            if promotions.is_empty() {
                scenarios.push((piece_move, Self::from_board(&new_board)));
            } else {
                for promotion in promotions {
                    scenarios.push((piece_move.clone(), Self::from_board(&promotion)));
                }
            }
        }

        scenarios
    }

    pub fn white_in_check(&self) -> bool {
        self.board.position.is_in_check(Color::White)
    }

    pub fn black_in_check(&self) -> bool {
        self.board.position.is_in_check(Color::Black)
    }
}
