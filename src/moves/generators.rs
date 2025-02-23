use crate::components::{constants, pieces::Bitboard};

/*
8 0 0 0 0 0 0 0 0
7 0 0 0 0 0 0 0 0
6 0 0 0 0 0 0 0 0
5 0 0 0 0 0 0 0 0
4 0 0 0 0 0 0 0 0
3 0 0 0 0 0 0 0 0
2 0 0 0 0 0 0 0 0
1 0 0 0 0 0 0 0 0
  a b c d e f g h
*/

/// Returns all the possibile white pawns attacking moves.
/// Black pieces position need to be considered in order to return only the legal attacking moves
/// Calling the function with black_pieces = std::u64::MAX returns all the attacked squares. Some of these moves might not be legal for the pawn.
pub fn white_pawn_attack(
    starting_position: Bitboard,
    _blockers: Bitboard,
    black_pieces: Bitboard,
) -> Bitboard {
    ((starting_position << 7 & Bitboard::new(constants::NOT_A_RANK))
        | (starting_position << 9 & Bitboard::new(constants::NOT_H_RANK)))
        & black_pieces
}

/// Returns all the possibile black pawns attacking moves.
/// White pieces position need to be considered in order to return only the legal attacking moves
/// Calling the function with white_pieces = std::u64::MAX returns all the attacked squares. Some of these moves might not be legal for the pawn.
pub fn black_pawn_attack(
    starting_position: Bitboard,
    _blockers: Bitboard,
    white_pieces: Bitboard,
) -> Bitboard {
    ((starting_position >> 7 & Bitboard::new(constants::NOT_H_RANK))
        | (starting_position >> 9 & Bitboard::new(constants::NOT_A_RANK)))
        & white_pieces
}

/// Returns all possible pawns advancing moves considering other pieces positioned on the board.
/// Running the function with blockers == 0 returns all possbile advancing move, without any blocking pieces in front of the pawn.
pub fn white_pawn_quiet_moves(starting_position: Bitboard, blockers: Bitboard) -> Bitboard {
    if starting_position.bits & constants::SECOND_ROW != 0 {
        return (starting_position << 8 & !blockers)
            | (starting_position << 16 & !blockers & !(blockers << 8));
    } else {
        return starting_position << 8 & !blockers;
    }
}

pub fn black_pawn_quiet_moves(starting_position: Bitboard, blockers: Bitboard) -> Bitboard {
    if starting_position.bits & constants::SEVENTH_ROW != 0 {
        return (starting_position >> 8 & !blockers)
            | (starting_position >> 16 & !blockers & !(blockers >> 8));
    } else {
        return starting_position >> 8 & !blockers;
    }
}

/// blockers = pieces of the same and opposite color
///
/// enemies = opposite color pieces
pub fn white_pawn(starting_position: Bitboard, blockers: Bitboard, enemies: Bitboard) -> Bitboard {
    let atk = white_pawn_attack(starting_position, blockers, enemies);
    let quiet = white_pawn_quiet_moves(starting_position, blockers);

    atk | quiet
}

/// blockers = pieces of the same and opposite color
///
/// enemies = opposite color pieces
pub fn black_pawn(starting_position: Bitboard, blockers: Bitboard, enemies: Bitboard) -> Bitboard {
    let atk = black_pawn_attack(starting_position, blockers, enemies);
    let quiet = black_pawn_quiet_moves(starting_position, blockers);

    atk | quiet
}

/// blockers = pieces of the same color
pub fn knight(starting_position: Bitboard, blockers: Bitboard, _enemies: Bitboard) -> Bitboard {
    Bitboard::new(
        ((starting_position.bits << 15 & constants::NOT_A_RANK)
            | (starting_position.bits >> 15 & constants::NOT_H_RANK)
            | (starting_position.bits << 17 & constants::NOT_H_RANK)
            | (starting_position.bits >> 17 & constants::NOT_A_RANK)
            | (starting_position.bits >> 6 & constants::NOT_H_RANK & constants::NOT_G_RANK)
            | (starting_position.bits << 6 & constants::NOT_A_RANK & constants::NOT_B_RANK)
            | (starting_position.bits << 10 & constants::NOT_H_RANK & constants::NOT_G_RANK)
            | (starting_position.bits >> 10 & constants::NOT_A_RANK & constants::NOT_B_RANK))
            & !blockers.bits,
    )
}

/// blockers = pieces of the same color
pub fn bishop(starting_position: Bitboard, blockers: Bitboard, enemies: Bitboard) -> Bitboard {
    let mut bitboard = Bitboard { bits: 0 };
    let offset = 63 - starting_position.bits.leading_zeros();
    let row = offset / 8;
    let column_from_end = offset % 8;

    let mut moving_row = row as i32 + 1;
    let mut moving_col: i32 = column_from_end as i32 + 1;

    while moving_row < 8 && moving_col < 8 {
        let add_position: u64 = 1 << (moving_row * 8 + moving_col);
        // there is a blocking piece, i don't have to consider this move
        if add_position & blockers.bits != 0 {
            break;
        }
        bitboard.bits |= add_position;
        // there is an enemy piece, i need to consider this move but then i need to stop
        if add_position & enemies.bits != 0 {
            break;
        }
        moving_row += 1;
        moving_col += 1;
    }

    moving_row = row as i32 + 1;
    moving_col = column_from_end as i32 - 1;

    while moving_row < 8 && moving_col >= 0 {
        let add_position: u64 = 1 << (moving_row * 8 + moving_col);
        if add_position & blockers.bits != 0 {
            break;
        }
        bitboard.bits |= add_position;
        if add_position & enemies.bits != 0 {
            break;
        }
        moving_row += 1;
        moving_col -= 1;
    }

    moving_row = row as i32 - 1;
    moving_col = column_from_end as i32 + 1;

    while moving_row >= 0 && moving_col < 8 {
        let add_position: u64 = 1 << (moving_row * 8 + moving_col);
        if add_position & blockers.bits != 0 {
            break;
        }
        bitboard.bits |= add_position;
        if add_position & enemies.bits != 0 {
            break;
        }
        moving_row -= 1;
        moving_col += 1;
    }

    moving_row = row as i32 - 1;
    moving_col = column_from_end as i32 - 1;

    while moving_row >= 0 && moving_col >= 0 {
        let add_position: u64 = 1 << (moving_row * 8 + moving_col);
        if add_position & blockers.bits != 0 {
            break;
        }
        bitboard.bits |= add_position;
        if add_position & enemies.bits != 0 {
            break;
        }
        moving_row -= 1;
        moving_col -= 1;
    }

    bitboard
}

/// blockers = pieces of the same color
pub fn rook(starting_position: Bitboard, blockers: Bitboard, enemies: Bitboard) -> Bitboard {
    let mut bitboard = Bitboard { bits: 0 };
    let offset = 63 - starting_position.bits.leading_zeros();
    let row = offset / 8;
    let column_from_end = offset % 8;

    for moving_row in row + 1..8 {
        let add_pos = 1 << (moving_row * 8 + column_from_end);
        if add_pos & blockers.bits != 0 {
            break;
        }
        bitboard.bits |= add_pos;
        if add_pos & enemies.bits != 0 {
            break;
        }
    }

    for moving_row in (0..row).rev() {
        let add_pos = 1 << (moving_row * 8 + column_from_end);
        if add_pos & blockers.bits != 0 {
            break;
        }
        bitboard.bits |= add_pos;
        if add_pos & enemies.bits != 0 {
            break;
        }
    }

    for moving_col in column_from_end + 1..8 {
        let add_pos = 1 << (row * 8 + moving_col);
        if add_pos & blockers.bits != 0 {
            break;
        }
        bitboard.bits |= add_pos;
        if add_pos & enemies.bits != 0 {
            break;
        }
    }

    for moving_col in (0..column_from_end).rev() {
        let add_pos = 1 << (row * 8 + moving_col);
        if add_pos & blockers.bits != 0 {
            break;
        }
        bitboard.bits |= add_pos;
        if add_pos & enemies.bits != 0 {
            break;
        }
    }

    bitboard
}

/// blockers = pieces of the same color
pub fn queen(starting_position: Bitboard, blockers: Bitboard, enemies: Bitboard) -> Bitboard {
    let rook_bitboard = rook(starting_position, blockers, enemies);
    let bishop_bitboard = bishop(starting_position, blockers, enemies);

    Bitboard {
        bits: (rook_bitboard.bits | bishop_bitboard.bits),
    }
}

/// blockers = pieces of the same color
pub fn king(starting_position: Bitboard, blockers: Bitboard, _enemies: Bitboard) -> Bitboard {
    Bitboard::new(
        ((starting_position.bits << 1 & constants::NOT_H_RANK)
            | (starting_position.bits << 9 & constants::NOT_H_RANK)
            | (starting_position.bits >> 7 & constants::NOT_H_RANK)
            | (starting_position.bits << 8)
            | (starting_position.bits << 7 & constants::NOT_A_RANK)
            | (starting_position.bits >> 1 & constants::NOT_A_RANK)
            | (starting_position.bits >> 9 & constants::NOT_A_RANK)
            | (starting_position.bits >> 8))
            & !blockers.bits,
    )
}
