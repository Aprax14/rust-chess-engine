use std::iter;

use crate::moves::generator;

use super::{
    board::Board,
    constants::{EIGHT_ROW, FIRST_ROW},
    piece::{self, Bitboard, Color, Kind, Piece},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleSide {
    Queen,
    King,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveVariant {
    Standard {
        from: Bitboard,
        to: Bitboard,
    },
    Castle(CastleSide),
    Promote {
        from: Bitboard,
        to: Bitboard,
        to_piece: Kind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub piece: Piece,
    pub action: MoveVariant,
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

    pub fn apply_move(&self, player_move: &Move) -> Option<Scenario> {
        let new_board = self.board.make_unchecked_move(player_move);

        if new_board.position.is_in_check(player_move.piece.color) {
            // discard position, is not legal
            return None;
        } else {
            return Some(Scenario { board: new_board });
        }
    }

    pub fn white_in_check(&self) -> bool {
        self.board.position.is_in_check(Color::White)
    }

    pub fn black_in_check(&self) -> bool {
        self.board.position.is_in_check(Color::Black)
    }
}
