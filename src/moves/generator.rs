use crate::types::{
    board::Board,
    constants::{EIGHT_ROW, FIRST_ROW},
    moves::Move,
    piece,
};

/*
    in the following function I don't need to check if the generated moves are valid beacause:
    - the player is moving an opponent piece: not possible, already filtered by color at the start,
    - there is the piece in the starting square: always, beacause we generate starting from a piece in a square
    - the move is one of the possible generated moves: always, we are generating them with the generators functions
    - in the resulting position the player is suiciding the king: the non static evaluator is going to discard it anyways
*/

pub fn generate_moves_ordered(board: &Board, only_captures: bool) -> Vec<Move> {
    let side = board.turn;
    let other_side = side.other();
    let opponent_squares = board.position.squares_occupied_by_color(other_side);

    let mut promotions: Vec<Move> = Vec::new();
    let mut captures: Vec<Move> = Vec::new();
    let mut quiet_moves: Vec<Move> = Vec::new();

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
                if piece.kind == piece::Kind::Pawn
                    && (to_square.bits & EIGHT_ROW != 0 || to_square.bits & FIRST_ROW != 0)
                {
                    promotions.push(Move {
                        piece: *piece,
                        from: piece_position,
                        to: to_square,
                    });
                } else if to_square.bits & opponent_squares.bits != 0 {
                    captures.push(Move {
                        piece: *piece,
                        from: piece_position,
                        to: to_square,
                    });
                } else {
                    quiet_moves.push(Move {
                        piece: *piece,
                        from: piece_position,
                        to: to_square,
                    });
                }
            }
        }
    }
    if only_captures {
        return captures;
    }

    captures.extend(quiet_moves);
    promotions.extend(captures);
    promotions
}
