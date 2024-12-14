use std::cmp::Reverse;

use strum::IntoEnumIterator;

use crate::types::{
    board::{Bitboards, Board, Castle},
    constants::{EIGHT_ROW, FIRST_ROW},
    moves::{CastleSide, Move, MoveVariant, Scenario},
    piece::{self, Bitboard, Color, Kind, Piece},
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
/// - castling
/// - quiet moves
pub fn generate_moves_ordered(
    board: &Board,
    only_critical: bool,
    current_pv: &Vec<Move>,
) -> Vec<Move> {
    let side = board.turn;
    let other_side = side.other();

    let our_squares = board.position.squares_occupied_by_color(side);
    let opponent_squares = board.position.squares_occupied_by_color(other_side);

    let mut possible_best: Vec<Move> = Vec::new();
    let mut stop_checks: Vec<Move> = Vec::new();
    let mut promotions: Vec<Move> = Vec::new();
    let mut checks: Vec<Move> = Vec::new();
    let mut captures: Vec<(Move, i32)> = Vec::new();
    let castling = castling_moves(board);
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
                    opponent_squares,
                ),
                _ => moves_generator(piece_position, our_squares, opponent_squares),
            };

            for to_square in moves_bitboard.single_squares() {
                let current_move = Move {
                    piece: *piece,
                    action: MoveVariant::Standard {
                        from: piece_position,
                        to: to_square,
                    },
                };

                let attacked_squares = match piece.kind {
                    piece::Kind::Pawn => moves_generator(
                        to_square,
                        board.position.occupied_cells(),
                        opponent_squares,
                    ),
                    _ => moves_generator(to_square, our_squares, opponent_squares),
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
                    // this move is a promotion -> save all possible promotion moves
                    for piece_kind in piece::Kind::iter() {
                        if piece_kind == piece::Kind::Pawn || piece_kind == piece::Kind::King {
                            continue;
                        }
                        let promotion = Move {
                            piece: *piece,
                            action: MoveVariant::Promote {
                                from: piece_position,
                                to: to_square,
                                to_piece: piece_kind,
                            },
                        };
                        promotions.push(promotion);
                    }
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
                            .expect("square shouldn't be empty");
                        move_rating += target.kind.value() * 10;
                    }

                    quiet_moves.push((current_move, move_rating));
                }
            }
        }
    }

    captures.sort_by_key(|(_, rating)| Reverse(*rating));

    if only_critical {
        return stop_checks
            .into_iter()
            .chain(captures.into_iter().map(|(m, _)| m))
            .collect();
    }

    quiet_moves.sort_by_key(|(_, rating)| Reverse(*rating));

    possible_best
        .into_iter()
        .chain(stop_checks)
        .chain(promotions)
        .chain(checks)
        .chain(captures.into_iter().map(|(m, _)| m))
        .chain(castling)
        .chain(quiet_moves.into_iter().map(|(m, _)| m))
        .collect()
}

pub fn castling_moves(board: &Board) -> Vec<Move> {
    inner_castling_moves(board, board.white_can_castle, board.black_can_castle)
}

fn inner_castling_moves(
    board: &Board,
    white_can_castle: Castle,
    black_can_castle: Castle,
) -> Vec<Move> {
    let castle_king = Move {
        piece: Piece {
            color: board.turn,
            kind: piece::Kind::King,
        },
        action: MoveVariant::Castle(CastleSide::King),
    };
    let castle_queen = Move {
        piece: Piece {
            color: board.turn,
            kind: piece::Kind::King,
        },
        action: MoveVariant::Castle(CastleSide::Queen),
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
                return Vec::new();
            } else {
                return vec![castle_king];
            }
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
                return Vec::new();
            } else {
                return vec![castle_queen];
            }
        }
        (Color::White, Castle::Both, _) => {
            let mut castle = inner_castling_moves(board, Castle::King, black_can_castle);
            let castle_queen = inner_castling_moves(board, Castle::Queen, black_can_castle);
            castle.extend(castle_queen);
            castle
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
                return Vec::new();
            } else {
                return vec![castle_king];
            }
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
                return Vec::new();
            } else {
                return vec![castle_queen];
            }
        }
        (Color::Black, _, Castle::Both) => {
            let mut castle = inner_castling_moves(board, white_can_castle, Castle::King);
            let castle_queen = inner_castling_moves(board, white_can_castle, Castle::Queen);
            castle.extend(castle_queen);
            castle
        }
        _ => Vec::new(),
    }
}

pub fn bitboards_after_castling(
    current_bitboards: &Bitboards,
    turn: Color,
    side: CastleSide,
) -> Bitboards {
    let mut new_bitboards = current_bitboards.clone();
    let king = Piece {
        color: turn,
        kind: Kind::King,
    };
    let rook = Piece {
        color: turn,
        kind: Kind::Rook,
    };

    match (turn, side) {
        (Color::White, CastleSide::King) => {
            let king_position = new_bitboards
                .by_piece
                .get_mut(&king)
                .expect("failed to get king");
            *king_position = Bitboard {
                bits: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010,
            };
            let rooks_position = new_bitboards
                .by_piece
                .get_mut(&rook)
                .expect("failed to get rook");
            *rooks_position = Bitboard {
                bits: (rooks_position.bits
                    & !0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001)
                    | 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000100,
            };
        }
        (Color::White, CastleSide::Queen) => {
            let king_position = new_bitboards
                .by_piece
                .get_mut(&king)
                .expect("failed to get king");
            *king_position = Bitboard {
                bits: 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100000,
            };
            let rooks_position = new_bitboards
                .by_piece
                .get_mut(&rook)
                .expect("failed to get rook");
            *rooks_position = Bitboard {
                bits: (rooks_position.bits
                    & !0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000000)
                    | 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000,
            };
        }
        (Color::Black, CastleSide::King) => {
            let king_position = new_bitboards
                .by_piece
                .get_mut(&king)
                .expect("failed to get king");
            *king_position = Bitboard {
                bits: 0b00000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            };
            let rooks_position = new_bitboards
                .by_piece
                .get_mut(&rook)
                .expect("failed to get rook");
            *rooks_position = Bitboard {
                bits: (rooks_position.bits
                    & !0b00000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                    | 0b00000100_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            };
        }
        (Color::Black, CastleSide::Queen) => {
            let king_position = new_bitboards
                .by_piece
                .get_mut(&king)
                .expect("failed to get king");
            *king_position = Bitboard {
                bits: 0b00100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            };
            let rooks_position = new_bitboards
                .by_piece
                .get_mut(&rook)
                .expect("failed to get rook");
            *rooks_position = Bitboard {
                bits: (rooks_position.bits
                    & !0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000)
                    | 0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            };
        }
    }

    new_bitboards
}
