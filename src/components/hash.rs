use std::sync::OnceLock;

use super::{
    castle::Castle,
    pieces::{Color, PieceKind},
};

// Layout of the Zobrist random table:
//   [0 .. 768)  = piece (12 types) × square (64)
//   [768]       = side to move (White)
//   [769]       = white can castle kingside
//   [770]       = white can castle queenside
//   [771]       = black can castle kingside
//   [772]       = black can castle queenside
const ZOBRIST_SIZE: usize = 773;
static ZOBRIST_TABLE: OnceLock<[u64; ZOBRIST_SIZE]> = OnceLock::new();

pub(crate) fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;

    *state
}

fn init_zobrist() -> [u64; ZOBRIST_SIZE] {
    let mut state: u64 = 0x123_4567_89AB_CDEF;
    let mut table = [0u64; ZOBRIST_SIZE];
    for val in &mut table {
        *val = xorshift64(&mut state);
    }

    table
}

fn piece_index(color: Color, kind: PieceKind) -> usize {
    let color_offset = match color {
        Color::White => 0,
        Color::Black => 6,
    };
    let kind_offset = match kind {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    };

    color_offset + kind_offset
}

/// Returns the Zobrist key for a (piece, square) pair.
pub fn piece_square_hash(color: Color, kind: PieceKind, square: u8) -> u64 {
    let table = ZOBRIST_TABLE.get_or_init(init_zobrist);

    table[piece_index(color, kind) * 64 + square as usize]
}

/// Returns the key for toggling the side to move.
pub fn side_to_move_hash() -> u64 {
    ZOBRIST_TABLE.get_or_init(init_zobrist)[768]
}

/// Returns the combined key for a pair of castling rights.
/// XOR-ing this in twice cancels out.
pub fn castle_rights_hash(white: Castle, black: Castle) -> u64 {
    let table = ZOBRIST_TABLE.get_or_init(init_zobrist);
    let mut h = 0u64;
    if matches!(white, Castle::King | Castle::Both) {
        h ^= table[769];
    }
    if matches!(white, Castle::Queen | Castle::Both) {
        h ^= table[770];
    }
    if matches!(black, Castle::King | Castle::Both) {
        h ^= table[771];
    }
    if matches!(black, Castle::Queen | Castle::Both) {
        h ^= table[772];
    }

    h
}
