use crate::components::{constants, pieces::Bitboard};

/*
* 8 0 0 0 0 0 0 0 0
* 7 0 0 0 0 0 0 0 0
* 6 0 0 0 0 0 0 0 0
* 5 0 0 0 0 0 0 0 0
* 4 0 0 0 0 0 0 0 0
* 3 0 0 0 0 0 0 0 0
* 2 0 0 0 0 0 0 0 0
* 1 0 0 0 0 0 0 0 0
*   a b c d e f g h
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
        (starting_position << 8 & !blockers)
            | (starting_position << 16 & !blockers & !(blockers << 8))
    } else {
        starting_position << 8 & !blockers
    }
}

pub fn black_pawn_quiet_moves(starting_position: Bitboard, blockers: Bitboard) -> Bitboard {
    if starting_position.bits & constants::SEVENTH_ROW != 0 {
        (starting_position >> 8 & !blockers)
            | (starting_position >> 16 & !blockers & !(blockers >> 8))
    } else {
        starting_position >> 8 & !blockers
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

fn bishop_single(sq: u8, blockers: Bitboard, enemies: Bitboard) -> u64 {
    let mut bits = 0u64;
    let row = (sq / 8) as i32;
    let col = (sq % 8) as i32;

    let mut r = row + 1;
    let mut c = col + 1;
    while r < 8 && c < 8 {
        let pos: u64 = 1 << (r * 8 + c);
        if pos & blockers.bits != 0 {
            break;
        }
        bits |= pos;
        if pos & enemies.bits != 0 {
            break;
        }
        r += 1;
        c += 1;
    }

    r = row + 1;
    c = col - 1;
    while r < 8 && c >= 0 {
        let pos: u64 = 1 << (r * 8 + c);
        if pos & blockers.bits != 0 {
            break;
        }
        bits |= pos;
        if pos & enemies.bits != 0 {
            break;
        }
        r += 1;
        c -= 1;
    }

    r = row - 1;
    c = col + 1;
    while r >= 0 && c < 8 {
        let pos: u64 = 1 << (r * 8 + c);
        if pos & blockers.bits != 0 {
            break;
        }
        bits |= pos;
        if pos & enemies.bits != 0 {
            break;
        }
        r -= 1;
        c += 1;
    }

    r = row - 1;
    c = col - 1;
    while r >= 0 && c >= 0 {
        let pos: u64 = 1 << (r * 8 + c);
        if pos & blockers.bits != 0 {
            break;
        }
        bits |= pos;
        if pos & enemies.bits != 0 {
            break;
        }
        r -= 1;
        c -= 1;
    }

    bits
}

/// blockers = pieces of the same color
pub fn bishop(starting_position: Bitboard, blockers: Bitboard, enemies: Bitboard) -> Bitboard {
    let mut result = Bitboard { bits: 0 };
    for sq in starting_position.single_squares() {
        result.bits |= bishop_single(sq, blockers, enemies);
    }
    result
}

fn rook_single(sq: u8, blockers: Bitboard, enemies: Bitboard) -> u64 {
    let mut bits = 0u64;
    let row = (sq / 8) as u32;
    let col = (sq % 8) as u32;

    for r in row + 1..8 {
        let pos = 1u64 << (r * 8 + col);
        if pos & blockers.bits != 0 {
            break;
        }
        bits |= pos;
        if pos & enemies.bits != 0 {
            break;
        }
    }

    for r in (0..row).rev() {
        let pos = 1u64 << (r * 8 + col);
        if pos & blockers.bits != 0 {
            break;
        }
        bits |= pos;
        if pos & enemies.bits != 0 {
            break;
        }
    }

    for c in col + 1..8 {
        let pos = 1u64 << (row * 8 + c);
        if pos & blockers.bits != 0 {
            break;
        }
        bits |= pos;
        if pos & enemies.bits != 0 {
            break;
        }
    }

    for c in (0..col).rev() {
        let pos = 1u64 << (row * 8 + c);
        if pos & blockers.bits != 0 {
            break;
        }
        bits |= pos;
        if pos & enemies.bits != 0 {
            break;
        }
    }

    bits
}

/// blockers = pieces of the same color
pub fn rook(starting_position: Bitboard, blockers: Bitboard, enemies: Bitboard) -> Bitboard {
    let mut result = Bitboard { bits: 0 };
    for sq in starting_position.single_squares() {
        result.bits |= rook_single(sq, blockers, enemies);
    }
    result
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
