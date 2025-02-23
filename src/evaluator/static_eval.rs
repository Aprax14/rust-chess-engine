use crate::components::{
    board::Board,
    constants,
    pieces::{Bitboard, Color},
};

use super::utils;

#[derive(Debug, Clone)]
pub struct StaticEval {
    pub white: i32,
    pub black: i32,
}

impl StaticEval {
    pub fn new() -> Self {
        StaticEval { white: 0, black: 0 }
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

            // consider the central position of the pieces:
            let central = Bitboard {
                bits: bitboard.bits & constants::CENTRAL_MASK,
            };
            if central.bits != 0 {
                let single_bitboards = central.single_squares();
                for b in single_bitboards {
                    let index = b.bits.leading_zeros();
                    eval.add(piece.color, constants::SQUARES_VALUE[index as usize]);
                }
            }
        }

        eval
    }
}
