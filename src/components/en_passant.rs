use crate::moves::{
    generators,
    move_type::{Move, MoveKind},
};

use super::{
    board::Board,
    pieces::{Bitboard, Color, Piece, PieceKind},
    position::BBPosition,
};

/// Returns the (at most two) en passant captures available in this position.
///
/// Uses the same return style as [`crate::components::castle::available_castling_moves`]:
/// a tuple where each element is `Some(Move)` if that capture exists.
///
/// The trick for finding capturing pawns: a white pawn at square P attacks T if
/// a black pawn at T would attack P, so we call `black_pawn_attack(target, 0, white_pawns)`
/// (and vice versa for black). This reuses the existing generator logic with correct
/// file-wrapping masks.
pub fn available_en_passant_moves(board: &Board) -> (Option<Move>, Option<Move>) {
    if board.en_passant_target.bits == 0 {
        return (None, None);
    }

    let target = board.en_passant_target;
    let capturing_pawn = Piece::new(board.turn, PieceKind::Pawn);
    let pawns = board.position.get(capturing_pawn);

    // Reverse the pawn attack: find which of our pawns can reach the target square.
    let attackers = match board.turn {
        Color::White => generators::black_pawn_attack(target, Bitboard::new(0), pawns),
        Color::Black => generators::white_pawn_attack(target, Bitboard::new(0), pawns),
    };

    let to = target.bits.trailing_zeros() as u8;
    let mut result = (None, None);
    let mut iter = attackers.single_squares();

    if let Some(from) = iter.next() {
        result.0 = Some(Move {
            piece: capturing_pawn,
            action: MoveKind::EnPassant { from, to },
        });
    }
    if let Some(from) = iter.next() {
        result.1 = Some(Move {
            piece: capturing_pawn,
            action: MoveKind::EnPassant { from, to },
        });
    }

    result
}

/// Returns the updated position after an en passant capture.
/// The capturing pawn moves to the target square and the captured pawn
/// is removed from the board.
pub fn bitboards_after_en_passant(
    current_bitboards: &BBPosition,
    player_move: &Move,
) -> BBPosition {
    let MoveKind::EnPassant { from, to } = player_move.action else {
        unreachable!("bitboards_after_en_passant called with non-EnPassant move");
    };

    let from_bb = Bitboard::new(1 << from);
    let to_bb = Bitboard::new(1 << to);
    let turn = player_move.piece.color;

    // The captured pawn sits one rank behind the en passant target square.
    let captured_sq = match turn {
        Color::White => to_bb.bits >> 8,
        Color::Black => to_bb.bits << 8,
    };

    let mut new_bitboards = current_bitboards.clone();

    // Move the capturing pawn.
    let pawn_bb = new_bitboards.get_mut(player_move.piece);
    pawn_bb.bits &= !from_bb.bits;
    pawn_bb.bits |= to_bb.bits;

    // Remove the captured pawn.
    let enemy_pawn = Piece::new(turn.other(), PieceKind::Pawn);
    new_bitboards.get_mut(enemy_pawn).bits &= !captured_sq;

    new_bitboards
}
