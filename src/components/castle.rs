use anyhow::anyhow;

use crate::moves::move_type::{Move, MoveKind};

use super::{
    board::Board,
    pieces::{Color, Piece, PieceKind},
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
    pub fn parse_from_str(s: &str) -> Result<(Self, Self), anyhow::Error> {
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

/// Returns (king_from, king_to, rook_from, rook_to) bit-index tuples for a castling move.
fn castle_squares(turn: Color, side: CastleSide) -> (u8, u8, u8, u8) {
    match (turn, side) {
        (Color::White, CastleSide::King) => (3, 1, 0, 2),
        (Color::White, CastleSide::Queen) => (3, 5, 7, 4),
        (Color::Black, CastleSide::King) => (59, 57, 56, 58),
        (Color::Black, CastleSide::Queen) => (59, 61, 63, 60),
    }
}

/// Applies a castling move in place, moving king and rook to their post-castle squares.
pub fn apply_castling_in_place(bitboards: &mut BBPosition, turn: Color, side: CastleSide) {
    let (king_from, king_to, rook_from, rook_to) = castle_squares(turn, side);
    let king = Piece::new(turn, PieceKind::King);
    let rook = Piece::new(turn, PieceKind::Rook);

    let king_bb = bitboards.get_mut(king);
    king_bb.bits = (king_bb.bits & !(1u64 << king_from)) | (1u64 << king_to);

    let rook_bb = bitboards.get_mut(rook);
    rook_bb.bits = (rook_bb.bits & !(1u64 << rook_from)) | (1u64 << rook_to);
}

/// Reverses a castling move in place, restoring king and rook to their pre-castle squares.
pub fn unapply_castling_in_place(bitboards: &mut BBPosition, turn: Color, side: CastleSide) {
    // Reverse: swap from/to relative to apply
    let (king_from, king_to, rook_from, rook_to) = castle_squares(turn, side);
    let king = Piece::new(turn, PieceKind::King);
    let rook = Piece::new(turn, PieceKind::Rook);

    let king_bb = bitboards.get_mut(king);
    king_bb.bits = (king_bb.bits & !(1u64 << king_to)) | (1u64 << king_from);

    let rook_bb = bitboards.get_mut(rook);
    rook_bb.bits = (rook_bb.bits & !(1u64 << rook_to)) | (1u64 << rook_from);
}

/// Calculates the new board position after a castling move is made.
pub fn bitboards_after_castling(
    current_bitboards: &BBPosition,
    turn: Color,
    side: CastleSide,
) -> BBPosition {
    let mut new_bitboards = current_bitboards.clone();
    apply_castling_in_place(&mut new_bitboards, turn, side);

    new_bitboards
}
