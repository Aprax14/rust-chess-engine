use anyhow::anyhow;

use crate::moves::moves::{Move, MoveKind};

use super::{
    board::Board,
    pieces::{Bitboard, Color, Piece, PieceKind},
    position::BBPosition,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Castle {
    No,
    King,
    Queen,
    Both,
}

impl Castle {
    pub fn from_str(s: &str) -> Result<(Self, Self), anyhow::Error> {
        match s {
            "KQkq" => Ok((Self::Both, Self::Both)),
            "Kkq" => Ok((Self::King, Self::Both)),
            "Qkq" => Ok((Self::Queen, Self::Both)),
            "kq" => Ok((Self::No, Self::Both)),
            "k" => Ok((Self::No, Self::King)),
            "q" => Ok((Self::No, Self::Queen)),
            "KQk" => Ok((Self::Both, Self::King)),
            "KQq" => Ok((Self::Both, Self::Queen)),
            "KQ" => Ok((Self::Both, Self::No)),
            "K" => Ok((Self::King, Self::No)),
            "Q" => Ok((Self::Queen, Self::No)),
            "-" => Ok((Self::No, Self::No)),
            _ => Err(anyhow!("invalid castling right notation: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastleSide {
    Queen,
    King,
}

/// Returns a tuple of 2 elements. The first is Some if castling king side is a valid move.
/// The second is some if castling queen side is a valid move.
pub fn available_castling_moves(
    board: &Board,
    white_can_castle: Castle,
    black_can_castle: Castle,
) -> (Option<Move>, Option<Move>) {
    let castle_king = Move {
        piece: Piece::new(board.turn, PieceKind::King),
        action: MoveKind::Castle(CastleSide::King),
    };
    let castle_queen = Move {
        piece: Piece::new(board.turn, PieceKind::King),
        action: MoveKind::Castle(CastleSide::Queen),
    };
    let occupied_squares = board.position.occupied_cells();

    match (board.turn, white_can_castle, black_can_castle) {
        (Color::White, Castle::King, _) => {
            let attacked_squares = board.attacked_squares(Color::Black);
            if (attacked_squares.bits
                & 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001110
                != 0)
                || (occupied_squares.bits
                    & 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000110
                    != 0)
            {
                return (None, None);
            }

            (Some(castle_king), None)
        }
        (Color::White, Castle::Queen, _) => {
            let attacked_squares = board.attacked_squares(Color::Black);
            if (attacked_squares.bits
                & 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00111000
                != 0)
                || (occupied_squares.bits
                    & 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01110000
                    != 0)
            {
                return (None, None);
            }

            (None, Some(castle_queen))
        }
        (Color::White, Castle::Both, _) => {
            let castle_king = available_castling_moves(board, Castle::King, black_can_castle);
            let castle_queen = available_castling_moves(board, Castle::Queen, black_can_castle);
            (castle_king.0, castle_queen.1)
        }
        (Color::Black, _, Castle::King) => {
            let attacked_squares = board.attacked_squares(Color::White);
            if (attacked_squares.bits
                & 0b00001110_00000000_00000000_00000000_00000000_00000000_00000000_00000000
                != 0)
                || (occupied_squares.bits
                    & 0b00000110_00000000_00000000_00000000_00000000_00000000_00000000_00000000
                    != 0)
            {
                return (None, None);
            }

            (Some(castle_king), None)
        }
        (Color::Black, _, Castle::Queen) => {
            let attacked_squares = board.attacked_squares(Color::White);
            if (attacked_squares.bits
                & 0b00111000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
                != 0)
                || (occupied_squares.bits
                    & 0b01110000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
                    != 0)
            {
                return (None, None);
            }

            (None, Some(castle_queen))
        }
        (Color::Black, _, Castle::Both) => {
            let castle_king = available_castling_moves(board, white_can_castle, Castle::King);
            let castle_queen = available_castling_moves(board, white_can_castle, Castle::Queen);
            (castle_king.0, castle_queen.1)
        }
        _ => (None, None),
    }
}

/// Calculates the new board position after a casling move is made
pub fn bitboards_after_castling(
    current_bitboards: &BBPosition,
    turn: Color,
    side: CastleSide,
) -> BBPosition {
    let mut new_bitboards = current_bitboards.clone();
    let king = Piece::new(turn, PieceKind::King);
    let rook = Piece::new(turn, PieceKind::Rook);

    match (turn, side) {
        (Color::White, CastleSide::King) => {
            let king_position = new_bitboards.get_mut(king);
            *king_position = Bitboard::new(
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010,
            );
            let rooks_position = new_bitboards.get_mut(rook);
            *rooks_position = Bitboard::new(
                (rooks_position.bits
                    & !0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001)
                    | 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000100,
            );
        }
        (Color::White, CastleSide::Queen) => {
            let king_position = new_bitboards.get_mut(king);
            *king_position = Bitboard::new(
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100000,
            );
            let rooks_position = new_bitboards.get_mut(rook);
            *rooks_position = Bitboard::new(
                (rooks_position.bits
                    & !0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000000)
                    | 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000,
            );
        }
        (Color::Black, CastleSide::King) => {
            let king_position = new_bitboards.get_mut(king);
            *king_position = Bitboard::new(
                0b00000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            );
            let rooks_position = new_bitboards.get_mut(rook);
            *rooks_position = Bitboard::new(
                (rooks_position.bits
                    & !0b00000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                    | 0b00000100_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            );
        }
        (Color::Black, CastleSide::Queen) => {
            let king_position = new_bitboards.get_mut(king);
            *king_position = Bitboard::new(
                0b00100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            );
            let rooks_position = new_bitboards.get_mut(rook);
            *rooks_position = Bitboard::new(
                (rooks_position.bits
                    & !0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                    | 0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            );
        }
    }

    new_bitboards
}

impl BBPosition {
    pub fn position_after_castling(&self, turn: Color, side: CastleSide) -> Self {
        let mut new_bitboards = self.clone();
        let king = Piece::new(turn, PieceKind::King);
        let rook = Piece::new(turn, PieceKind::Rook);

        match (turn, side) {
            (Color::White, CastleSide::King) => {
                let king_position = new_bitboards.get_mut(king);
                *king_position = Bitboard::new(
                    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010,
                );
                let rooks_position = new_bitboards.get_mut(rook);
                *rooks_position = Bitboard::new(
                    (rooks_position.bits
                        & !0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001)
                        | 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000100,
                );
            }
            (Color::White, CastleSide::Queen) => {
                let king_position = new_bitboards.get_mut(king);
                *king_position = Bitboard::new(
                    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100000,
                );
                let rooks_position = new_bitboards.get_mut(rook);
                *rooks_position = Bitboard::new(
                    (rooks_position.bits
                        & !0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000000)
                        | 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000,
                );
            }
            (Color::Black, CastleSide::King) => {
                let king_position = new_bitboards.get_mut(king);
                *king_position = Bitboard::new(
                    0b00000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
                );
                let rooks_position = new_bitboards.get_mut(rook);
                *rooks_position = Bitboard::new((rooks_position.bits
                        & !0b00000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                        | 0b00000100_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
                );
            }
            (Color::Black, CastleSide::Queen) => {
                let king_position = new_bitboards.get_mut(king);
                *king_position = Bitboard::new(
                    0b00100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
                );
                let rooks_position = new_bitboards.get_mut(rook);
                *rooks_position = Bitboard::new(
                    (rooks_position.bits
                        & !0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                        | 0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
                );
            }
        }

        new_bitboards
    }
}
