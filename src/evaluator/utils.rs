use crate::{
    components::{
        constants,
        pieces::{Bitboard, Piece, PieceKind},
        position::BBPosition,
    },
    moves::move_type::{Move, MoveKind},
};

pub fn attacked_squares_score(
    board_position: &BBPosition,
    piece: Piece,
    position: Bitboard,
) -> i32 {
    let attacks = board_position.attacks(piece, position).bits;
    if attacks == 0 {
        return 0;
    }

    // Attribute empty-square value to all attacked squares, then adjust for occupied ones.
    // This iterates the 12 bitboards exactly once instead of once per attacked square.
    let mut score = attacks.count_ones() as i32 * constants::ATTACKED_EMPTY_SQUARE_VALUE;
    for (p, bb) in board_position.into_iter() {
        let overlap = attacks & bb.bits;
        if overlap != 0 {
            score += overlap.count_ones() as i32
                * (p.kind.attacked_value() - constants::ATTACKED_EMPTY_SQUARE_VALUE);
        }
    }

    score
}

fn inner_move_score_no_captures(m: &Move, board_position: &BBPosition) -> i32 {
    match m.action {
        MoveKind::Castle(_) => constants::CASTLING_VALUE,
        MoveKind::EnPassant { to, .. } => {
            if board_position.square_is_defended_by(to, m.piece.color.other()) {
                0
            } else {
                PieceKind::Pawn.value()
            }
        }
        MoveKind::Standard { from, to, .. } => {
            let attacked_before =
                attacked_squares_score(board_position, m.piece, Bitboard::new(1 << from));
            let attacked_after =
                attacked_squares_score(board_position, m.piece, Bitboard::new(1 << to));

            // > 0 it means the position is improving. < 0 the piece is going in a worse position
            attacked_after - attacked_before
        }
        MoveKind::Promote { .. } => constants::PROMOTION_VALUE,
    }
}

pub fn move_score_with_mvv_lva(m: &Move, board_position: &BBPosition) -> i32 {
    match m.action {
        MoveKind::Castle(_) => constants::CASTLING_VALUE,
        // En passant always captures a pawn of equal value (pawn for pawn).
        // The target square is empty, but it can still be defended by another piece.
        MoveKind::EnPassant { to, .. } => {
            if board_position.square_is_defended_by(to, m.piece.color.other()) {
                0
            } else {
                PieceKind::Pawn.value()
            }
        }
        MoveKind::Standard { to, captured, .. } => {
            let Some(victim) = captured else {
                return inner_move_score_no_captures(m, board_position);
            };

            let mut capture_value = victim.kind.value() - m.piece.kind.value();
            if board_position.square_is_defended_by(to, victim.color) {
                if capture_value < 0 {
                    // we are capturing a defended less valuable piece with a more valuable piece
                    capture_value = capture_value * 3 / 2;
                }
            } else if capture_value < 0 {
                // the piece is not defended so this is not a bad move
                // we consider the material gain
                capture_value = victim.kind.value();
            }

            capture_value
        }
        MoveKind::Promote {
            from,
            to,
            to_piece,
            captured,
        } => {
            let standard_eval = move_score_with_mvv_lva(
                &Move {
                    piece: m.piece,
                    action: MoveKind::Standard { from, to, captured },
                },
                board_position,
            );
            if !board_position.square_is_defended_by(to, m.piece.color.other()) {
                // square is not defended so the promoted piece is going to remain on the board
                standard_eval + to_piece.value() - PieceKind::Pawn.value()
            } else {
                // promoted piece is probably not going to stay on the board
                standard_eval
            }
        }
    }
}
