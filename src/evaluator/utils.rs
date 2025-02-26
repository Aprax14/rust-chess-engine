use crate::{
    components::{
        constants,
        pieces::{Bitboard, Piece, PieceKind},
        position::BBPosition,
    },
    moves::moves::{Move, MoveKind},
};

pub fn attacked_squares_score(
    board_position: &BBPosition,
    piece: Piece,
    position: Bitboard,
) -> i32 {
    board_position
        .attacks(piece, position)
        .single_squares()
        .map(|shift| {
            if let Some(p) = board_position.piece_at(shift) {
                p.kind.attacked_value()
            } else {
                constants::ATTACKED_EMPTY_SQUARE_VALUE
            }
        })
        .sum()
}

fn inner_move_score_no_captures(m: Move, board_position: &BBPosition) -> i32 {
    match m.action {
        MoveKind::Castle(_) => constants::CASTLING_VALUE,
        MoveKind::Standard { from, to } => {
            let attacked_before =
                attacked_squares_score(board_position, m.piece, Bitboard::new(1 << from));
            let attacked_after =
                attacked_squares_score(board_position, m.piece, Bitboard::new(1 << to));

            // > 0 it means the position is improving. < 0 the piece is going in a worse position
            attacked_after - attacked_before
        }
        MoveKind::Promote {
            from: _,
            to: _,
            to_piece: _,
        } => constants::PROMOTION_VALUE,
    }
}

pub fn move_score_with_mvv_lva(m: Move, board_position: &BBPosition) -> i32 {
    match m.action {
        MoveKind::Castle(_) => constants::CASTLING_VALUE,
        MoveKind::Standard { from, to } => {
            let Some(victim) = board_position.piece_at(to) else {
                return inner_move_score_no_captures(m, board_position);
            };
            let attacker = board_position
                .piece_at(from)
                .expect("from square should contain a piece");

            let mut capture_value = victim.kind.value() - attacker.kind.value();
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
        MoveKind::Promote { from, to, to_piece } => {
            let standard_eval = move_score_with_mvv_lva(
                Move {
                    piece: m.piece,
                    action: MoveKind::Standard { from, to },
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
