use std::cmp::Reverse;

use crate::types::{
    board::Board,
    constants::{EIGHT_ROW, FIRST_ROW},
    moves::Move,
    piece::{self, Bitboard, Piece},
};

/*
    in the following function I don't need to check if the generated moves are valid beacause:
    - the player is moving an opponent piece: not possible, already filtered by color at the start,
    - there is the piece in the starting square: always, beacause we generate starting from a piece in a square
    - the move is one of the possible generated moves: always, we are generating them with the generators functions
    - in the resulting position the player is suiciding the king: the non static evaluator is going to discard it anyways
*/

/// returns all the possible legal moves order by:
///
/// - possible best given from the principal variation
/// - moves that stops ongoing checks
/// - pawns promotions
/// - checks
/// - captures
/// - quiet moves
pub fn generate_moves_ordered(
    board: &Board,
    only_critical: bool,
    current_pv: &Vec<Move>,
) -> Vec<Move> {
    let side = board.turn;
    let other_side = side.other();
    let opponent_squares = board.position.squares_occupied_by_color(other_side);

    let mut possible_best: Vec<Move> = Vec::new();
    let mut stop_checks: Vec<Move> = Vec::new();
    let mut promotions: Vec<Move> = Vec::new();
    let mut checks: Vec<Move> = Vec::new();
    let mut captures: Vec<(Move, i32)> = Vec::new();
    let mut quiet_moves: Vec<(Move, i32)> = Vec::new();

    for (piece, bitboard) in &board.position.by_piece {
        if piece.color != side {
            continue;
        }

        let moves_generator = piece.get_moves_generator();
        let pieces_position = bitboard.single_squares();
        for piece_position in pieces_position {
            let moves_bitboard = match piece.kind {
                piece::Kind::Pawn => moves_generator(
                    piece_position,
                    board.position.occupied_cells(),
                    board.position.squares_occupied_by_color(other_side),
                ),
                _ => moves_generator(
                    piece_position,
                    board.position.squares_occupied_by_color(side),
                    board.position.squares_occupied_by_color(other_side),
                ),
            };

            for to_square in moves_bitboard.single_squares() {
                let current_move = Move {
                    piece: *piece,
                    from: piece_position,
                    to: to_square,
                };

                let attacked_squares = match piece.kind {
                    piece::Kind::Pawn => moves_generator(
                        to_square,
                        board.position.occupied_cells(),
                        board.position.squares_occupied_by_color(other_side),
                    ),
                    _ => moves_generator(
                        to_square,
                        board.position.squares_occupied_by_color(side),
                        board.position.squares_occupied_by_color(other_side),
                    ),
                }
                .bits;

                // discard illegal moves
                let next_board = board.make_unchecked_move(&current_move);
                if next_board.position.is_in_check(side) {
                    continue;
                }

                // insert the move in one of the initialized vectors for better ordering
                if current_pv.contains(&current_move) && !only_critical {
                    // previously saved in principal variation
                    possible_best.push(current_move);
                } else if board.position.is_in_check(side) {
                    // player is in check, the move we generate are all captures or moves that puts the kind out of check
                    stop_checks.push(current_move);
                } else if piece.kind == piece::Kind::Pawn
                    && (to_square.bits & EIGHT_ROW != 0 || to_square.bits & FIRST_ROW != 0)
                    && !only_critical
                {
                    // this move is a promotion
                    promotions.push(current_move);
                } else if attacked_squares
                    & board
                        .position
                        .bitboard_by_piece(Piece {
                            kind: piece::Kind::King,
                            color: other_side,
                        })
                        .bits
                    != 0
                    && !only_critical
                {
                    // this move is a check
                    checks.push(current_move);
                } else if to_square.bits & opponent_squares.bits != 0 {
                    let target = board
                        .position
                        .get_piece_in_square(to_square)
                        .expect("this square should not be empty");
                    let move_rating = (target.kind.value() - piece.kind.value()) * 100;
                    captures.push((current_move, move_rating));
                } else {
                    // its a quiet move
                    let attacked_squares_with_pieces = attacked_squares & opponent_squares.bits;
                    let mut move_rating = 0;

                    for square in (Bitboard {
                        bits: attacked_squares_with_pieces,
                    })
                    .single_squares()
                    {
                        let target = board
                            .position
                            .get_piece_in_square(square)
                            .expect("square should no be empty");
                        move_rating += target.kind.value() * 10;
                    }

                    quiet_moves.push((current_move, move_rating));
                }
            }
        }
    }

    captures.sort_by_key(|(_, rating)| Reverse(*rating));

    if only_critical {
        return [stop_checks, captures.into_iter().map(|(m, _)| m).collect()]
            .into_iter()
            .flatten()
            .collect();
    }

    quiet_moves.sort_by_key(|(_, rating)| Reverse(*rating));
    [
        possible_best,
        stop_checks,
        promotions,
        checks,
        captures.into_iter().map(|(m, _)| m).collect(),
        quiet_moves.into_iter().map(|(m, _)| m).collect(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
