use crate::types::{
    board::Board,
    constants,
    piece::{Bitboard, Color},
};

/*
Some notes...
What i need to consider for a static evaluation:
- Material
- Central position
- Quantity of attacked squares
- ??
Only for opening and middle game:
- Castling Rights
- King safety
*/

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
    pub fn static_evaluate(board: &Board) -> StaticEval {
        let mut eval = StaticEval::new();

        let white_attacked_squares = board.position.attacked_squares(Color::White);
        let black_attacked_squares = board.position.attacked_squares(Color::Black);

        for (piece, bitboard) in &board.position.by_piece {
            // consider material:
            let value = bitboard.count_bits() * piece.kind.value();
            eval.add(piece.color, value);

            // consider attacked pieces:
            match piece.color {
                Color::White => {
                    let attacked_by_black = Bitboard {
                        bits: bitboard.bits & black_attacked_squares.bits,
                    };
                    eval.add(
                        Color::Black,
                        attacked_by_black.count_bits() * piece.kind.attacked_value(),
                    );
                }
                Color::Black => {
                    let attacked_by_white = Bitboard {
                        bits: bitboard.bits & white_attacked_squares.bits,
                    };
                    eval.add(
                        Color::White,
                        attacked_by_white.count_bits() * piece.kind.attacked_value(),
                    );
                }
            }

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

        // consider the attacked squares:
        eval.add(
            Color::White,
            white_attacked_squares.count_bits() * constants::ATTACKED_SQUARE_ADVANTAGE,
        );
        eval.add(
            Color::Black,
            black_attacked_squares.count_bits() * constants::ATTACKED_SQUARE_ADVANTAGE,
        );

        eval
    }
}
