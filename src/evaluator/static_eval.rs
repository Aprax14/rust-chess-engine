use crate::components::{
    board::Board,
    constants,
    pieces::{Bitboard, Color, PieceKind},
};

use super::utils;

#[derive(Debug, Clone, Default)]
pub struct StaticEval {
    pub white: i32,
    pub black: i32,
}

impl StaticEval {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, side: Color, value: i32) {
        match side {
            Color::White => self.white += value,
            Color::Black => self.black += value,
        }
    }
}

impl StaticEval {
    pub fn static_evaluate(board: &Board) -> Self {
        let mut eval = Self::new();

        for (piece, bitboard) in &board.position {
            // consider material:
            let value = bitboard.count_bits() * piece.kind.value();
            eval.add(piece.color, value);

            // consider attacked pieces and squares:
            // giving attacked_squares_score the entire piecekind bitboard in 1 call should work
            let attacks_score = utils::attacked_squares_score(&board.position, *piece, *bitboard);
            eval.add(piece.color, attacks_score);

            if piece.kind == PieceKind::King {
                let table = if board.is_endgame() {
                    &constants::KING_ENDGAME_TABLE
                } else {
                    &constants::KING_MIDDLEGAME_TABLE
                };
                for shift in bitboard.single_squares() {
                    eval.add(piece.color, table[(63 - shift) as usize]);
                }
            } else {
                let central = Bitboard {
                    bits: bitboard.bits & constants::CENTRAL_MASK,
                };
                if central.bits != 0 {
                    for shift in central.single_squares() {
                        eval.add(piece.color, constants::SQUARES_VALUE[(63 - shift) as usize]);
                    }
                }
            }
        }

        eval
    }
}
