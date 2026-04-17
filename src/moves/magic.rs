/// Magic Bitboards for sliding piece attack generation.
///
/// For each square, a "relevant occupancy mask" is precomputed (all squares
/// that can block the piece, excluding board-edge squares).  At runtime the
/// attack set is retrieved with a single multiplication + right-shift:
///
///   index = ((occupancy & mask).wrapping_mul(magic)) >> shift
///   attacks = table[index]
///
/// Tables are built once at first use
use std::sync::OnceLock;

use crate::components::hash::xorshift64;

/// Return all squares attacked by a bishop on `sq` given `occupancy`
/// (all occupied squares).  The result includes the first
/// blocker in every diagonal (potential captures) but does NOT filter
/// out friendly pieces so the caller must apply `& !own_pieces`.
pub fn bishop_attacks(sq: u8, occupancy: u64) -> u64 {
    let table = BISHOP_TABLE.get_or_init(|| build_table(false));
    table[sq as usize].lookup(occupancy)
}

/// Return all squares attacked by a rook on `sq` given `occupancy`.
/// Same contract as `bishop_attacks`.
pub fn rook_attacks(sq: u8, occupancy: u64) -> u64 {
    let table = ROOK_TABLE.get_or_init(|| build_table(true));
    table[sq as usize].lookup(occupancy)
}

/// Eagerly initialise both tables.  Call once at engine startup to avoid
/// any latency on the first move-generation call.
pub fn init() {
    let _ = ROOK_TABLE.get_or_init(|| build_table(true));
    let _ = BISHOP_TABLE.get_or_init(|| build_table(false));
}

// Static tables
static ROOK_TABLE: OnceLock<Vec<MagicEntry>> = OnceLock::new();
static BISHOP_TABLE: OnceLock<Vec<MagicEntry>> = OnceLock::new();

struct MagicEntry {
    /// This is needed to filter the relevant cells that
    /// could make the difference in the moves generation.
    /// (diagonals for the bishop and rows and columns for the rooks)
    mask: u64,
    /// Chosen so that `(relevant_occ * magic) >> shift`
    /// produces a unique index for every possible subset of `mask`.
    magic: u64,
    /// How many bits to discard after the multiplication: `64 - mask.count_ones()`.
    /// Keeps only the top `mask.count_ones()` bits, giving an index in `0 .. 2^mask.count_ones()`.
    shift: u32,
    /// Precomputed attack bitboards, one per occupancy index.
    /// `attacks[index]` is the set of squares attacked given the occupancy
    /// pattern that maps to `index` through the magic formula.
    attacks: Box<[u64]>,
}

impl MagicEntry {
    #[inline(always)]
    fn lookup(&self, occupancy: u64) -> u64 {
        let idx = ((occupancy & self.mask).wrapping_mul(self.magic) >> self.shift) as usize;
        self.attacks[idx]
    }
}

/// Relevant occupancy mask for a rook on `sq`.
/// Includes all rank/file squares that can act as blockers edges exluded.
fn rook_mask(sq: u8) -> u64 {
    let row = (sq / 8) as u64;
    let col = (sq % 8) as u64;
    let mut mask = 0u64;

    // Row occupancy (same row, interior columns only)
    for c in 1u64..=6 {
        if c != col {
            mask |= 1u64 << (row * 8 + c);
        }
    }
    // Column occupancy (same column, interior rows only)
    for r in 1u64..=6 {
        if r != row {
            mask |= 1u64 << (r * 8 + col);
        }
    }

    mask
}

/// Relevant occupancy mask for a bishop on `sq`.
/// Includes all diagonal squares, edges exluded.
fn bishop_mask(sq: u8) -> u64 {
    let row = (sq / 8) as i32;
    let col = (sq % 8) as i32;
    let mut mask = 0u64;

    for &(dr, dc) in &[(1i32, 1i32), (1, -1), (-1, 1), (-1, -1)] {
        let mut r = row + dr;
        let mut c = col + dc;
        while (0..8).contains(&r) && (0..8).contains(&c) {
            // Only interior squares matter; edge squares never block
            if r > 0 && r < 7 && c > 0 && c < 7 {
                mask |= 1u64 << (r * 8 + c);
            }
            r += dr;
            c += dc;
        }
    }

    mask
}

/// All squares a rook on `sq` attacks given `occupancy`
/// The first blocker in each ray is included.
/// This is slow and used only during table initialisation.
fn rook_attacks_slow(sq: u8, occupancy: u64) -> u64 {
    let row = (sq / 8) as i32;
    let col = (sq % 8) as i32;
    let mut attacks = 0u64;

    for &(dr, dc) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
        let mut r = row + dr;
        let mut c = col + dc;
        while (0..8).contains(&r) && (0..8).contains(&c) {
            let bit = 1u64 << (r * 8 + c);
            attacks |= bit;
            if occupancy & bit != 0 {
                break;
            }
            r += dr;
            c += dc;
        }
    }

    attacks
}

/// All squares a bishop on `sq` attacks given `occupancy`.
/// The first blocker in each ray is included.
/// This is slow and used only during table initialisation.
fn bishop_attacks_slow(sq: u8, occupancy: u64) -> u64 {
    let row = (sq / 8) as i32;
    let col = (sq % 8) as i32;
    let mut attacks = 0u64;

    for &(dr, dc) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut r = row + dr;
        let mut c = col + dc;
        while (0..8).contains(&r) && (0..8).contains(&c) {
            let bit = 1u64 << (r * 8 + c);
            attacks |= bit;
            if occupancy & bit != 0 {
                break;
            }
            r += dr;
            c += dc;
        }
    }

    attacks
}

// Occupancy enumeration (Carry-Rippler subset trick)
// Magic number search + table construction

/// Find a magic number for `sq` and build its attack table.
fn find_magic(sq: u8, mask: u64, is_rook: bool) -> MagicEntry {
    let bits = mask.count_ones(); // pieces on relevant squares
    let n = 1usize << bits; // possible combinations
    let shift = 64 - bits; // `shift` used in the index formula

    let attacks_for = |occ| {
        if is_rook {
            rook_attacks_slow(sq, occ)
        } else {
            bishop_attacks_slow(sq, occ)
        }
    };

    // Seed differently for every square so searches diverge.
    // See Knuth "The Art of Computer Programming" for the constants.
    // Those 2 constants guarantee a period of 2^64 before any repetition
    let mut state: u64 = (sq as u64)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    // Sparse magic candidates (few set bits) converge faster.
    let mut next_sparse =
        || xorshift64(&mut state) & xorshift64(&mut state) & xorshift64(&mut state);

    let mut table = vec![u64::MAX; n];

    loop {
        let magic = next_sparse();

        // Quick filter: a good magic scatters the top byte of (mask * magic) well.
        if ((mask.wrapping_mul(magic)) >> 56).count_ones() < 6 {
            continue;
        }

        table.fill(u64::MAX);
        let mut ok = true;

        // Carry-Rippler: enumerate all 2^N subsets of mask.
        let mut occ = 0u64;
        loop {
            let idx = (occ.wrapping_mul(magic) >> shift) as usize;
            let atk = attacks_for(occ);
            if table[idx] == u64::MAX {
                table[idx] = atk;
            } else if table[idx] != atk {
                // collision!!
                ok = false;
                break;
            }
            // Carry-Rippler - see: https://analog-hors.github.io/site/magic-bitboards/
            occ = occ.wrapping_sub(mask) & mask;
            if occ == 0 {
                break;
            }
        }

        if ok {
            return MagicEntry {
                mask,
                magic,
                shift,
                attacks: table.into_boxed_slice(),
            };
        }
    }
}

/// Build all 64 magic entries for either rooks or bishops.
fn build_table(is_rook: bool) -> Vec<MagicEntry> {
    (0u8..64)
        .map(|sq| {
            let mask = if is_rook {
                rook_mask(sq)
            } else {
                bishop_mask(sq)
            };
            find_magic(sq, mask, is_rook)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the magic lookup matches the slow ray-casting for every
    /// square and a representative set of occupancy patterns.
    #[test]
    fn magic_matches_slow_rook() {
        init();
        let occ_patterns: &[u64] = &[
            0,
            u64::MAX,
            0x5555555555555555,
            0xAAAAAAAAAAAAAAAA,
            0x0F0F0F0F0F0F0F0F,
            0xFF00FF00FF00FF00,
            0x1C0F9D6B3C32679C,
        ];
        for sq in 0u8..64 {
            for &occ in occ_patterns {
                let expected = rook_attacks_slow(sq, occ);
                let got = rook_attacks(sq, occ);
                assert_eq!(
                    got, expected,
                    "rook sq={sq} occ={occ:#018x}: expected {expected:#018x}, got {got:#018x}"
                );
            }
        }
    }

    #[test]
    fn magic_matches_slow_bishop() {
        init();
        let occ_patterns: &[u64] = &[
            0,
            u64::MAX,
            0x5555555555555555,
            0xAAAAAAAAAAAAAAAA,
            0x0F0F0F0F0F0F0F0F,
            0xFF00FF00FF00FF00,
            0x1C0F9D6B3C32679C,
        ];
        for sq in 0u8..64 {
            for &occ in occ_patterns {
                let expected = bishop_attacks_slow(sq, occ);
                let got = bishop_attacks(sq, occ);
                assert_eq!(
                    got, expected,
                    "bishop sq={sq} occ={occ:#018x}: expected {expected:#018x}, got {got:#018x}"
                );
            }
        }
    }
}
