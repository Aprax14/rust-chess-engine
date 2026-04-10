use std::sync::OnceLock;

use crate::components::{
    board::Board,
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

fn xorshift64(state: &mut u64) -> u64 {
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

/// Computes the Zobrist hash of a board position from scratch.
pub fn zobrist_hash(board: &Board) -> u64 {
    let table = ZOBRIST_TABLE.get_or_init(init_zobrist);
    let mut hash = 0u64;

    for (piece, bitboard) in &board.position {
        let pidx = piece_index(piece.color, piece.kind);
        for square in bitboard.single_squares() {
            hash ^= table[pidx * 64 + square as usize];
        }
    }

    if board.turn == Color::White {
        hash ^= table[768];
    }

    if matches!(board.white_can_castle, Castle::King | Castle::Both) {
        hash ^= table[769];
    }
    if matches!(board.white_can_castle, Castle::Queen | Castle::Both) {
        hash ^= table[770];
    }
    if matches!(board.black_can_castle, Castle::King | Castle::Both) {
        hash ^= table[771];
    }
    if matches!(board.black_can_castle, Castle::Queen | Castle::Both) {
        hash ^= table[772];
    }

    hash
}

/// Describes the reliability of a stored score relative to the true minimax value.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    /// The stored score is the exact minimax value.
    Exact,
    /// The stored score is a lower bound on the true value (fail-high / beta cutoff).
    Lower,
    /// The stored score is an upper bound on the true value (fail-low / alpha node).
    Upper,
}

#[derive(Clone, Copy)]
struct TtEntry {
    hash: u64,
    depth: i32,
    score: i32,
    bound: Bound,
}

impl TtEntry {
    const EMPTY: Self = TtEntry {
        hash: 0,
        depth: -1,
        score: 0,
        bound: Bound::Exact,
    };
}

pub struct ProbeResult {
    pub score: i32,
    pub bound: Bound,
}

pub struct TranspositionTable {
    table: Vec<TtEntry>,
    mask: usize,
}

impl TranspositionTable {
    /// Creates a table with ~1 M entries (≈ 20 MB).
    pub fn new() -> Self {
        let size = 1 << 20;
        TranspositionTable {
            table: vec![TtEntry::EMPTY; size],
            mask: size - 1,
        }
    }

    /// Returns the stored result if the entry matches `hash` and was computed
    /// at least as deep as the requested `depth`.
    pub fn probe(&self, hash: u64, depth: i32) -> Option<ProbeResult> {
        let entry = self.table[hash as usize & self.mask];
        if entry.hash == hash && entry.depth >= depth {
            Some(ProbeResult {
                score: entry.score,
                bound: entry.bound,
            })
        } else {
            None
        }
    }

    /// Stores a result.
    ///
    /// Replaces the existing entry if the new position is different (hash collision) or
    /// if the new entry was computed at greater or equal depth (depth-preferred replacement).
    pub fn store(&mut self, hash: u64, depth: i32, score: i32, bound: Bound) {
        let idx = hash as usize & self.mask;
        let entry = &mut self.table[idx];
        if entry.hash != hash || depth >= entry.depth {
            *entry = TtEntry {
                hash,
                depth,
                score,
                bound,
            };
        }
    }
}
