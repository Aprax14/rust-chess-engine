use crate::types::{
    board::Board,
    moves::{MovesByPiece, PiecePossibleMoves, PossibleMoves},
    piece::{self, Bitboard},
};

/*
    in the following function I don't need to check if the generated moves are valid beacause:
    - the player is moving an opponent piece: not possible, already filtered by color at the start,
    - there is the piece in the starting square: always, beacause we generate starting from a piece in a square
    - the move is one of the possible generated moves: always, we are generating them with the generators functions
    - in the resulting position the player is suiciding the king: the non static evaluator is going to discard it anyways
*/

/// generates all the possible valid moves on a given board for the player who need to play.
pub fn generate_moves_unordered(board: &Board, only_captures: bool) -> MovesByPiece {
    let side = board.turn;
    let other_side = side.other();
    let opponent_squares = if only_captures {
        board.position.squares_occupied_by_color(other_side)
    } else {
        Bitboard { bits: 0 }
    };

    let mut moves_by_piece: Vec<PiecePossibleMoves> = Vec::new();

    for (piece, bitboard) in &board.position.by_piece {
        if piece.color != side {
            continue;
        }
        let mut piece_moves = PiecePossibleMoves {
            piece: *piece,
            moves: Vec::new(),
        };
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

            piece_moves.moves.push(PossibleMoves {
                from: piece_position,
                to: {
                    if only_captures {
                        moves_bitboard
                            .single_squares()
                            .into_iter()
                            .filter(|b| b.bits & opponent_squares.bits != 0)
                            .collect::<Vec<Bitboard>>()
                    } else {
                        moves_bitboard.single_squares()
                    }
                },
            });
        }
        moves_by_piece.push(piece_moves);
    }
    MovesByPiece(moves_by_piece)
}

pub fn generate_moves_captures_first(board: &Board, only_captures: bool) -> MovesByPiece {
    let side = board.turn;
    let other_side = side.other();
    let opponent_squares = board.position.squares_occupied_by_color(other_side);

    let mut captures_by_piece: Vec<PiecePossibleMoves> = Vec::new();
    let mut moves_by_piece: Vec<PiecePossibleMoves> = Vec::new();

    for (piece, bitboard) in &board.position.by_piece {
        if piece.color != side {
            continue;
        }
        let mut piece_captures = PiecePossibleMoves {
            piece: *piece,
            moves: Vec::new(),
        };
        let mut piece_moves = PiecePossibleMoves {
            piece: *piece,
            moves: Vec::new(),
        };
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

            let (captures, not_captures) = {
                let (to_squares_captures, to_squares_not_captures): (Vec<Bitboard>, Vec<Bitboard>) =
                    moves_bitboard
                        .single_squares()
                        .into_iter()
                        .partition(|b| b.bits & opponent_squares.bits != 0);

                (
                    PossibleMoves {
                        from: piece_position,
                        to: to_squares_captures,
                    },
                    PossibleMoves {
                        from: piece_position,
                        to: to_squares_not_captures,
                    },
                )
            };

            piece_captures.moves.push(captures);
            piece_moves.moves.push(not_captures);
        }
        captures_by_piece.push(piece_captures);
        moves_by_piece.push(piece_moves);
    }
    let all_moves = if only_captures {
        captures_by_piece
    } else {
        captures_by_piece.extend(moves_by_piece);
        captures_by_piece
    };
    MovesByPiece(all_moves)
}