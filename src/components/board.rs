use std::fmt;

use anyhow::anyhow;

use crate::moves::move_type::{Move, MoveKind};

use super::{
    castle::{Castle, CastleSide},
    constants, hash,
    pieces::{Bitboard, Color, PieceKind},
    position::BBPosition,
};

#[derive(Debug, Clone)]
pub struct Board {
    pub position: BBPosition,
    pub turn: Color,
    pub en_passant_target: Bitboard,
    pub white_can_castle: Castle,
    pub black_can_castle: Castle,
    pub hash: u64,
    pub reps_50: u8,
    pub moves_count: u32,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        'outer: for pos in 0..64 {
            for (piece, position) in &self.position {
                // this piece is at this board position i'm checking for
                if (position.bits << pos).leading_zeros() == 0_u32 {
                    write!(f, " {} ", piece)?;
                    if (pos + 1) % 8 == 0 {
                        writeln!(f)?;
                    }
                    continue 'outer;
                }
            }
            write!(f, " . ")?;
            if (pos + 1) % 8 == 0 {
                writeln!(f)?;
            }
        }

        writeln!(f)?;
        writeln!(f)?;
        write!(f, "Turn: {}", self.turn)?;
        writeln!(f)?;
        write!(f, "Move Number: {}", self.moves_count)?;

        Ok(())
    }
}

impl Board {
    pub fn new_game() -> Self {
        Self::from_forsyth_edwards("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap()
    }

    /// Parse Board position from Forsyth-Edwards notation:
    ///
    /// Notation Exaple: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    /// # Panics
    /// On inconsistent notation.
    pub fn from_forsyth_edwards(s: &str) -> Result<Self, anyhow::Error> {
        let pieces = s.split(" ").collect::<Vec<&str>>();
        if pieces.len() != 6 {
            return Err(anyhow!("invalid Forsyth-Edwards notation for: {}", s));
        }
        let (position, turn, castling_rights, en_passant, reps_50, moves_count) = (
            pieces[0], pieces[1], pieces[2], pieces[3], pieces[4], pieces[5],
        );

        let position = BBPosition::from_fen_notation(position)?;
        let turn: Color = turn.try_into()?;
        let en_passant_target = match en_passant {
            "-" => Bitboard { bits: 0 },
            s => Bitboard::try_from(s)?,
        };
        let (white_can_castle, black_can_castle) = Castle::from_str(castling_rights)?;
        let reps_50: u8 = reps_50.parse()?;
        let moves_count: u32 = moves_count.parse()?;

        // Compute the Zobrist hash from scratch once at construction time.
        // All subsequent positions update it incrementally in make_unchecked_move.
        let mut h = hash::castle_rights_hash(white_can_castle, black_can_castle);
        if turn == Color::White {
            h ^= hash::side_to_move_hash();
        }
        for (piece, bitboard) in &position {
            for sq in bitboard.single_squares() {
                h ^= hash::piece_square_hash(piece.color, piece.kind, sq);
            }
        }

        Ok(Self {
            position,
            turn,
            en_passant_target,
            white_can_castle,
            black_can_castle,
            hash: h,
            reps_50,
            moves_count,
        })
    }

    pub fn attacked_squares(&self, side: Color) -> Bitboard {
        self.position.attacked_squares(side)
    }

    /// calculates how castling rights get changed by the move being made
    fn calculate_castling_rights(&self, player_move: &Move) -> (Castle, Castle) {
        let white_queen_rook = 7; // a1
        let white_king_rook = 0; // h1
        let black_queen_rook = 63; // a8
        let black_king_rook = 56; // h8

        let white_can_castle = match (player_move.piece.color, player_move.piece.kind) {
            (Color::White, PieceKind::King) => Castle::No,
            (Color::White, PieceKind::Rook) => match (self.white_can_castle, player_move.action) {
                (Castle::No, _) => Castle::No,
                (Castle::King, MoveKind::Standard { from, to: _ }) => {
                    if from == white_king_rook {
                        Castle::No
                    } else {
                        Castle::King
                    }
                }
                (Castle::Queen, MoveKind::Standard { from, to: _ }) => {
                    if from == white_queen_rook {
                        Castle::No
                    } else {
                        Castle::Queen
                    }
                }
                (Castle::Both, MoveKind::Standard { from, to: _ }) => {
                    if from == white_king_rook {
                        Castle::Queen
                    } else if from == white_queen_rook {
                        Castle::King
                    } else {
                        Castle::Both
                    }
                }
                _ => unreachable!(),
            },
            (Color::Black, _) => match (self.white_can_castle, player_move.action) {
                (Castle::No, _) => Castle::No,
                (
                    Castle::King,
                    MoveKind::Standard { from: _, to }
                    | MoveKind::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to == white_king_rook {
                        Castle::No
                    } else {
                        Castle::King
                    }
                }
                (
                    Castle::Queen,
                    MoveKind::Standard { from: _, to }
                    | MoveKind::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to == white_queen_rook {
                        Castle::No
                    } else {
                        Castle::Queen
                    }
                }
                (
                    Castle::Both,
                    MoveKind::Standard { from: _, to }
                    | MoveKind::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to == white_king_rook {
                        Castle::Queen
                    } else if to == white_queen_rook {
                        Castle::King
                    } else {
                        Castle::Both
                    }
                }
                _ => self.white_can_castle,
            },
            _ => self.white_can_castle,
        };

        let black_can_castle = match (player_move.piece.color, player_move.piece.kind) {
            (Color::Black, PieceKind::King) => Castle::No,
            (Color::Black, PieceKind::Rook) => match (self.black_can_castle, player_move.action) {
                (Castle::No, _) => Castle::No,
                (Castle::King, MoveKind::Standard { from, to: _ }) => {
                    if from == black_king_rook {
                        Castle::No
                    } else {
                        Castle::King
                    }
                }
                (Castle::Queen, MoveKind::Standard { from, to: _ }) => {
                    if from == black_queen_rook {
                        Castle::No
                    } else {
                        Castle::Queen
                    }
                }
                (Castle::Both, MoveKind::Standard { from, to: _ }) => {
                    if from == black_king_rook {
                        Castle::Queen
                    } else if from == black_queen_rook {
                        Castle::King
                    } else {
                        Castle::Both
                    }
                }
                _ => unreachable!(),
            },
            (Color::White, _) => match (self.black_can_castle, player_move.action) {
                (Castle::No, _) => Castle::No,
                (
                    Castle::King,
                    MoveKind::Standard { from: _, to }
                    | MoveKind::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to == black_king_rook {
                        Castle::No
                    } else {
                        Castle::King
                    }
                }
                (
                    Castle::Queen,
                    MoveKind::Standard { from: _, to }
                    | MoveKind::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to == black_queen_rook {
                        Castle::No
                    } else {
                        Castle::Queen
                    }
                }
                (
                    Castle::Both,
                    MoveKind::Standard { from: _, to }
                    | MoveKind::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to == black_king_rook {
                        Castle::Queen
                    } else if to == black_queen_rook {
                        Castle::King
                    } else {
                        Castle::Both
                    }
                }
                _ => self.black_can_castle,
            },
            _ => self.black_can_castle,
        };

        (white_can_castle, black_can_castle)
    }

    /// checks if the 50 moves rules counter should be resetted
    pub fn reset_50_moves(&self, player_move: &Move) -> bool {
        match player_move.action {
            MoveKind::EnPassant { .. } => true,
            MoveKind::Standard { from: _, to } => {
                player_move.piece.kind == PieceKind::Pawn
                    || ((1 << to) & self.position.occupied_cells().bits != 0)
            }
            _ => false,
        }
    }

    /// Returns true when total material (both sides, excluding kings) is below
    /// the endgame threshold, signalling that king centralisation is preferred
    /// over king safety on the back rank.
    pub fn is_endgame(&self) -> bool {
        let total_material: i32 = (&self.position)
            .into_iter()
            .filter(|(piece, _)| piece.kind != PieceKind::King)
            .map(|(piece, bitboard)| bitboard.count_bits() * piece.kind.value())
            .sum();

        total_material < constants::ENDGAME_MATERIAL_THRESHOLD
    }

    /// Returns true if the side to move has at least one non-pawn, non-king piece.
    /// Used to guard against null move pruning in pawn-only endgames (zugzwang risk).
    pub fn has_non_pawn_pieces(&self) -> bool {
        match self.turn {
            Color::White => {
                (self.position.get('N').bits
                    | self.position.get('B').bits
                    | self.position.get('R').bits
                    | self.position.get('Q').bits)
                    != 0
            }
            Color::Black => {
                (self.position.get('n').bits
                    | self.position.get('b').bits
                    | self.position.get('r').bits
                    | self.position.get('q').bits)
                    != 0
            }
        }
    }

    /// Passes the turn to the opponent without moving any piece.
    ///
    /// Only valid when the current player is not in check. Used internally by the
    /// search algorithm for null move pruning - never played in an actual game.
    pub fn make_null_move(&self) -> Self {
        Board {
            position: self.position.clone(),
            turn: self.turn.other(),
            en_passant_target: Bitboard::new(0),
            white_can_castle: self.white_can_castle,
            black_can_castle: self.black_can_castle,
            // No piece changes, castling rights unchanged, only side-to-move flips.
            hash: self.hash ^ hash::side_to_move_hash(),
            reps_50: self.reps_50 + 1,
            moves_count: self.moves_count + 1,
        }
    }

    /// Makes a move and updates position, turn, en passant target, castling rights and moves count.
    ///
    /// Does not prevent you to make an illegal move.
    pub fn make_unchecked_move(&self, player_move: &Move) -> Self {
        let position = self.position.inner_make_unchecked_move(player_move);

        let turn = self.turn.other();

        let en_passant_target = self.position.calculate_en_passant_target(player_move);
        let (white_can_castle, black_can_castle) = self.calculate_castling_rights(player_move);
        let reps_50 = if self.reset_50_moves(player_move) {
            0
        } else {
            self.reps_50 + 1
        };
        let moves_count = self.moves_count + 1;

        let hash = self.incremental_hash(player_move, white_can_castle, black_can_castle);

        Board {
            position,
            turn,
            en_passant_target,
            white_can_castle,
            black_can_castle,
            hash,
            reps_50,
            moves_count,
        }
    }

    /// Computes the Zobrist hash for the position that results from applying
    /// a `player_move`, using an incremental XOR update instead of
    /// recomputing from scratch.
    fn incremental_hash(
        &self,
        player_move: &Move,
        new_white_castle: Castle,
        new_black_castle: Castle,
    ) -> u64 {
        let mut h = self.hash;

        // Flip side to move.
        h ^= hash::side_to_move_hash();

        // Transition castling rights: cancel old bits, apply new bits.
        h ^= hash::castle_rights_hash(self.white_can_castle, self.black_can_castle);
        h ^= hash::castle_rights_hash(new_white_castle, new_black_castle);

        match player_move.action {
            MoveKind::Standard { from, to } => {
                h ^= hash::piece_square_hash(player_move.piece.color, player_move.piece.kind, from);
                if let Some(captured) = self.position.piece_at(to) {
                    h ^= hash::piece_square_hash(captured.color, captured.kind, to);
                }
                h ^= hash::piece_square_hash(player_move.piece.color, player_move.piece.kind, to);
            }
            MoveKind::Castle(side) => {
                let (king_from, king_to, rook_from, rook_to) =
                    Self::castle_piece_squares(player_move.piece.color, side);
                h ^= hash::piece_square_hash(player_move.piece.color, PieceKind::King, king_from);
                h ^= hash::piece_square_hash(player_move.piece.color, PieceKind::King, king_to);
                h ^= hash::piece_square_hash(player_move.piece.color, PieceKind::Rook, rook_from);
                h ^= hash::piece_square_hash(player_move.piece.color, PieceKind::Rook, rook_to);
            }
            MoveKind::EnPassant { from, to } => {
                h ^= hash::piece_square_hash(player_move.piece.color, PieceKind::Pawn, from);
                h ^= hash::piece_square_hash(player_move.piece.color, PieceKind::Pawn, to);
                // Captured pawn sits one rank behind the landing square.
                let captured_sq = match player_move.piece.color {
                    Color::White => to - 8,
                    Color::Black => to + 8,
                };
                h ^= hash::piece_square_hash(
                    player_move.piece.color.other(),
                    PieceKind::Pawn,
                    captured_sq,
                );
            }
            MoveKind::Promote { from, to, to_piece } => {
                h ^= hash::piece_square_hash(player_move.piece.color, PieceKind::Pawn, from);
                if let Some(captured) = self.position.piece_at(to) {
                    h ^= hash::piece_square_hash(captured.color, captured.kind, to);
                }
                h ^= hash::piece_square_hash(player_move.piece.color, to_piece, to);
            }
        }

        h
    }

    /// Returns (king_from, king_to, rook_from, rook_to) bit positions for a castling move.
    fn castle_piece_squares(color: Color, side: CastleSide) -> (u8, u8, u8, u8) {
        match (color, side) {
            (Color::White, CastleSide::King) => (3, 1, 0, 2), // e1→g1, h1→f1
            (Color::White, CastleSide::Queen) => (3, 5, 7, 4), // e1→c1, a1→d1
            (Color::Black, CastleSide::King) => (59, 57, 56, 58), // e8→g8, h8→f8
            (Color::Black, CastleSide::Queen) => (59, 61, 63, 60), // e8→c8, a8→d8
        }
    }
}
