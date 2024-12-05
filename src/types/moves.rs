use std::ops::Deref;

use crate::moves::generator;

use super::{
    board::Board,
    constants::{EIGHT_ROW, FIRST_ROW},
    piece::{self, Bitboard, Color, Piece},
};

#[derive(Debug, Clone)]
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
                    if new_board.position.is_in_check(pieces_moves.piece.color) {
                        // discard position, is not legal
                        continue;
                    }
                    let promotions = new_board.generate_promotion_variants();
                    if promotions.is_empty() {
                        scenarios.push(Self::from_board(new_board));
                    } else {
                        for promotion in promotions {
                            scenarios.push(Self::from_board(promotion));
                        }
                    }
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
