use std::fmt;

use anyhow::anyhow;

use crate::moves::moves::{Move, MoveKind};

use super::{
    castle::Castle,
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
    #[expect(dead_code)]
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

        Ok(Self {
            position,
            turn,
            en_passant_target,
            white_can_castle,
            black_can_castle,
            reps_50,
            moves_count,
        })
    }

    pub fn attacked_squares(&self, side: Color) -> Bitboard {
        self.position.attacked_squares(side)
    }

    /// calculates how castling rights get changed by the move being made
    fn calculate_castling_rights(&self, player_move: Move) -> (Castle, Castle) {
        let white_queen_rook = 56;
        let white_king_rook = 63;
        let black_queen_rook = 0;
        let black_king_rook = 7;

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
    pub fn reset_50_moves(&self, player_move: Move) -> bool {
        // suppose that the validity check already happened so a piece can not move on a square occupied by another piece of the same color.
        let occupied_cells = self.position.occupied_cells();
        if let MoveKind::Standard { from: _, to } = player_move.action {
            return player_move.piece.kind == PieceKind::Pawn
                || ((1 << to) & occupied_cells.bits != 0);
        }

        false
    }

    /// Makes a move and updates position, turn, en passant target, castling rights and moves count.
    ///
    /// Does not prevent you to make an illegal move.
    pub fn make_unchecked_move(&self, player_move: Move) -> Self {
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

        Board {
            position,
            turn,
            en_passant_target,
            white_can_castle,
            black_can_castle,
            reps_50,
            moves_count,
        }
    }
}
