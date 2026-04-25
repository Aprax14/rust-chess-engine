use std::sync::atomic::{AtomicU64, Ordering};

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

pub struct ProbeResult {
    pub score: i32,
    pub bound: Bound,
}

/// One slot in the transposition table.
///
/// Stored as two u64s so that reads and writes are individually atomic.  
/// The XOR trick is used for integrity: key = hash XOR data.
/// If two threads writing the same slot concurrently it gets detected at
/// probe time because `stored_key XOR stored_data != hash`. This is
/// considered as a cache miss to avoid making a corrupted move.
///
/// Data layout (64 bits):
///   bits 0-1 ->  Bound 2 bits for 3 variants
///   bits 2-33 -> score reinterpreted as u32
///   bits 34-49 -> depth reinterpreted as u16
///   bits 50-63 -> unused
#[derive(Debug, Default)]
struct TtSlot {
    key: AtomicU64,
    data: AtomicU64,
}

impl TtSlot {
    const fn new() -> Self {
        TtSlot {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }
}

fn compress_to_u64(depth: i32, score: i32, bound: Bound) -> u64 {
    let bound_bits = match bound {
        Bound::Exact => 0u64,
        Bound::Lower => 1u64,
        Bound::Upper => 2u64,
    };
    let score_bits = (score as u32) as u64;
    let depth_bits = (depth as u16) as u64;

    bound_bits | (score_bits << 2) | (depth_bits << 34)
}

fn unpack(data: u64) -> (i32, i32, Bound) {
    let bound = match data & 0b11 {
        0 => Bound::Exact,
        1 => Bound::Lower,
        _ => Bound::Upper,
    };
    let score = (data >> 2) as u32 as i32;
    let depth = (data >> 34) as u16 as i16 as i32;

    (depth, score, bound)
}

/// A lockless transposition table that can be shared across threads without
/// any mutex.  Probes and stores use `Relaxed` atomics. The XOR integrity
/// check catches corrupted reads so no ordering guarantees are needed.
///
/// The general idea of a transposition table is to store all positions already evaluated using a hash.
/// 1. Build the ZOBRIST_TABLE: a fixed array of random u64 values.
/// 2. Each piece type, square, castling right, and side to move maps to a unique index in ZOBRIST_TABLE (the value is its hash).
/// 3. XORing together all the values for the current position yields a single hash representing that position.
/// 4. At TranspositionTable.table[hash & mask] we store the evaluation result for that position.
/// 5. The mask is applied to the hash to index into a fixed-size table, keeping memory usage bounded.
#[derive(Debug, Default)]
pub struct TranspositionTable {
    table: Vec<TtSlot>,
    mask: usize,
}

impl TranspositionTable {
    /// Creates a table with ~1 M entries (≈ 16 MB).
    pub fn new() -> Self {
        let size = 1 << 20;
        TranspositionTable {
            table: (0..size).map(|_| TtSlot::new()).collect(),
            mask: size - 1,
        }
    }

    /// Returns the stored result if the entry matches `hash` and was computed
    /// at least as deep as the requested `depth`.
    pub fn probe(&self, hash: u64, depth: i32) -> Option<ProbeResult> {
        let slot = &self.table[hash as usize & self.mask];
        let key = slot.key.load(Ordering::Relaxed);
        let data = slot.data.load(Ordering::Relaxed);
        // XOR check: detects torn writes and hash collisions in one operation.
        if key ^ data != hash {
            return None;
        }
        let (entry_depth, score, bound) = unpack(data);
        if entry_depth < depth {
            return None;
        }

        Some(ProbeResult { score, bound })
    }

    /// Stores a result.  Uses depth-preferred replacement: an existing entry for
    /// the same hash is only overwritten if the new depth is >= the stored depth.
    /// The store itself is lock-free: data is written before key so a concurrent
    /// reader will fail the XOR check during the brief inconsistent window.
    pub fn store(&self, hash: u64, depth: i32, score: i32, bound: Bound) {
        let slot = &self.table[hash as usize & self.mask];

        // Depth-preferred replacement: only overwrite if new depth is at least as deep.
        let existing_key = slot.key.load(Ordering::Relaxed);
        let existing_data = slot.data.load(Ordering::Relaxed);
        if existing_key ^ existing_data == hash {
            let (existing_depth, _, _) = unpack(existing_data);
            if depth < existing_depth {
                return;
            }
        }

        let data = compress_to_u64(depth, score, bound);
        // Write data before key so a concurrent reader sees key ^ data != hash
        // during the brief window between the two stores.
        slot.data.store(data, Ordering::Relaxed);
        slot.key.store(hash ^ data, Ordering::Relaxed);
    }
}
