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

fn en_passant_captured_sq(to: u8, turn: Color) -> u64 {
    match turn {
        Color::White => (1u64 << to) >> 8,
        Color::Black => (1u64 << to) << 8,
    }
}

/// Applies an en-passant capture in place.
pub fn apply_en_passant_in_place(bitboards: &mut BBPosition, player_move: &Move) {
    let MoveKind::EnPassant { from, to } = player_move.action else {
        panic!("Fatal Error: apply_en_passant_in_place called with non-EnPassant move");
    };
    let turn = player_move.piece.color;
    let captured_sq = en_passant_captured_sq(to, turn);

    let pawn_bb = bitboards.get_mut(player_move.piece);
    pawn_bb.bits = (pawn_bb.bits & !(1u64 << from)) | (1u64 << to);

    let enemy_pawn = Piece::new(turn.other(), PieceKind::Pawn);
    bitboards.get_mut(enemy_pawn).bits &= !captured_sq;
}

/// Reverses an en-passant capture in place.
pub fn unapply_en_passant_in_place(bitboards: &mut BBPosition, player_move: &Move) {
    let MoveKind::EnPassant { from, to } = player_move.action else {
        unreachable!("unapply_en_passant_in_place called with non-EnPassant move");
    };
    let turn = player_move.piece.color;
    let captured_sq = en_passant_captured_sq(to, turn);

    // Move pawn back.
    let pawn_bb = bitboards.get_mut(player_move.piece);
    pawn_bb.bits = (pawn_bb.bits & !(1u64 << to)) | (1u64 << from);

    // Restore captured pawn.
    let enemy_pawn = Piece::new(turn.other(), PieceKind::Pawn);
    bitboards.get_mut(enemy_pawn).bits |= captured_sq;
}

/// Returns the updated position after an en passant capture.
pub fn bitboards_after_en_passant(
    current_bitboards: &BBPosition,
    player_move: &Move,
) -> BBPosition {
    let mut new_bitboards = current_bitboards.clone();
    apply_en_passant_in_place(&mut new_bitboards, player_move);
    new_bitboards.recompute_occupied();

    new_bitboards
}
