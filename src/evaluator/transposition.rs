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

/// The general idea of a transposition table is to store all positions already evaluated using a hash.
/// 1. Build the ZOBRIST_TABLE: a fixed array of random u64 values.
/// 2. Each piece type, square, castling right, and side to move maps to a unique index in ZOBRIST_TABLE (the value is its hash).
/// 3. XORing together all the values for the current position yields a single hash representing that position.
/// 4. At TranspositionTable.table[hash & mask] we store the evaluation result for that position.
/// 5. The mask is applied to the hash to index into a fixed-size table, keeping memory usage bounded.
pub struct TranspositionTable {
    table: Vec<TtEntry>,
    mask: usize,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        let size = 1 << 20;
        TranspositionTable {
            table: vec![TtEntry::EMPTY; size],
            mask: size - 1,
        }
    }
}

impl TranspositionTable {
    /// Creates a table with ~1 M entries (≈ 20 MB).
    pub fn new() -> Self {
        Self::default()
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
