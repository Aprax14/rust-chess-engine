use anyhow::anyhow;
use std::{collections::HashMap, fmt};
use strum::IntoEnumIterator;

use crate::{
    moves::generator,
    types::{moves::CastleSide, piece::Piece},
};

use super::{
    constants::{self, EIGHT_ROW, FIRST_ROW},
    moves::{Move, MoveVariant},
    piece::{self, Bitboard, Color, Kind},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Castle {
    No,
    King,
    Queen,
    Both,
}

impl Castle {
    fn from_str(s: &str) -> Result<(Self, Self), anyhow::Error> {
        match s {
            "KQkq" => Ok((Self::Both, Self::Both)),
            "Kkq" => Ok((Self::King, Self::Both)),
            "kq" => Ok((Self::No, Self::Both)),
            "KQk" => Ok((Self::Both, Self::King)),
            "KQq" => Ok((Self::Both, Self::Queen)),
            "KQ" => Ok((Self::Both, Self::No)),
            "-" => Ok((Self::No, Self::No)),
            _ => Err(anyhow!("invalid castling right notation: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bitboards {
    pub by_piece: HashMap<piece::Piece, piece::Bitboard>,
}

impl Bitboards {
    /// Returns an empty Chess board as Bitboards
    fn empty() -> Self {
        let mut pieces = HashMap::new();
        for c in piece::Color::iter() {
            for k in piece::Kind::iter() {
                pieces.insert(
                    piece::Piece { color: c, kind: k },
                    piece::Bitboard { bits: 0 },
                );
            }
        }

        Self { by_piece: pieces }
    }

    fn from_fen_position(s: &str) -> Result<Self, anyhow::Error> {
        let mut bb = Self::empty();
        let mut index: i32 = 63;

        let t = s.replace("/", "");

        for c in t.chars() {
            if let Some(n) = c.to_digit(10) {
                // sequence of empty squares
                index -= n as i32;
                continue;
            } else {
                // found a piece -> update the board
                let piece = piece::Piece::from_char(&c)?;
                bb.by_piece
                    .get_mut(&piece)
                    .expect("board should be completely initialized")
                    .bits |= 1 << index;
                index -= 1;
            }
        }

        Ok(bb)
    }

    pub fn occupied_cells(&self) -> Bitboard {
        Bitboard {
            bits: self.by_piece.values().map(|pos| pos.bits).sum(),
        }
    }

    pub fn empty_cells(&self) -> Bitboard {
        Bitboard {
            bits: !self.occupied_cells().bits,
        }
    }

    pub fn squares_occupied_by_color(&self, c: Color) -> Bitboard {
        Bitboard {
            bits: self
                .by_piece
                .iter()
                .filter(|(piece, _)| piece.color == c)
                .map(|(_, pos)| pos.bits)
                .sum(),
        }
    }

    pub fn pieces_position_array(&self) -> [Option<Piece>; 64] {
        const NONE: Option<Piece> = None;
        let mut pieces: [Option<Piece>; 64] = [NONE; 64];

        for (p, bitboard) in &self.by_piece {
            let mut counter = bitboard.bits;
            while counter != 0 {
                let index = counter.leading_zeros();
                pieces[index as usize] = Some(*p);
                counter &= !(1 << (63 - index));
            }
        }
        pieces
    }

    pub fn bitboard_by_piece(&self, p: piece::Piece) -> Bitboard {
        *self
            .by_piece
            .get(&p)
            .expect("missing piece information inside bitboard")
    }
    /// Return all the squares attacked by a side.
    ///
    /// For the pawns a square is considered attacked even if there is no piece there so the pawn can not capture anything there.
    pub fn attacked_squares(&self, side: Color) -> Bitboard {
        let mut accumulator = Bitboard { bits: 0 };

        let other_side = side.other();
        for (piece, positions) in &self.by_piece {
            if piece.color == other_side {
                continue;
            }
            let single_piece_bitboards = positions.single_squares();
            let move_generator = piece.get_attacks_generator();
            for bitboard in single_piece_bitboards {
                let possible_moves = match piece.kind {
                    piece::Kind::Pawn => {
                        move_generator(bitboard, self.occupied_cells(), Bitboard { bits: u64::MAX })
                    }
                    _ => move_generator(
                        bitboard,
                        self.squares_occupied_by_color(side),
                        self.squares_occupied_by_color(other_side),
                    ),
                };
                accumulator.bits |= possible_moves.bits;
            }
        }
        accumulator
    }

    pub fn side_attacks_square(&self, side: Color, square: Bitboard) -> bool {
        let attacked_squares = self.attacked_squares(side);
        (attacked_squares.bits & square.bits) != 0
    }

    pub fn get_piece_in_square(&self, square: Bitboard) -> Option<Piece> {
        for (piece, bitboard) in self.by_piece.iter() {
            if bitboard.bits & square.bits != 0 {
                return Some(*piece);
            }
        }

        None
    }

    pub fn is_in_check(&self, side: Color) -> bool {
        let other_side = side.other();
        let attacked_squares = self.attacked_squares(other_side);
        let king = Piece {
            kind: piece::Kind::King,
            color: side,
        };
        self.bitboard_by_piece(king).bits & attacked_squares.bits != 0
    }

    pub fn make_unchecked_move(&self, player_move: &Move) -> Self {
        match player_move.action {
            MoveVariant::Standard { from, to } => {
                let mut resulting_bitboards = self.clone();
                let piece_bitboard = resulting_bitboards
                    .by_piece
                    .get_mut(&player_move.piece)
                    .expect("failed to get piece bitboard");

                // remove piece from the old position
                piece_bitboard.bits &= !from.bits;

                // set the piece in the new position
                piece_bitboard.bits |= to.bits;

                let occupator = self.get_piece_in_square(to);
                if let Some(oc) = occupator {
                    resulting_bitboards
                        .by_piece
                        .get_mut(&oc)
                        .expect("missing piece from bitboards")
                        .bits &= !to.bits;
                }

                resulting_bitboards
            }
            MoveVariant::Castle(side) => {
                generator::bitboards_after_castling(&self, player_move.piece.color, side)
            }
            MoveVariant::Promote { from, to, to_piece } => {
                let mut resulting_bitboards = self.clone();
                let pawn_bitboard = resulting_bitboards
                    .by_piece
                    .get_mut(&player_move.piece)
                    .expect("failed to get pawn bitboard");

                // remove pawn from the old position
                pawn_bitboard.bits &= !from.bits;

                let new_piece_bitboard = resulting_bitboards
                    .by_piece
                    .get_mut(&Piece {
                        color: player_move.piece.color,
                        kind: to_piece,
                    })
                    .expect("failed to get piece bitboard");

                // set the piece it is promoting to in the new position
                new_piece_bitboard.bits |= to.bits;

                // remove possible captured pieces
                let occupator = self.get_piece_in_square(to);
                if let Some(oc) = occupator {
                    resulting_bitboards
                        .by_piece
                        .get_mut(&oc)
                        .expect("missing piece from bitboards")
                        .bits &= !to.bits;
                }

                resulting_bitboards
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub position: Bitboards,
    pub turn: piece::Color,
    pub en_passant_target: Bitboard,
    pub white_can_castle: Castle,
    pub black_can_castle: Castle,
    pub reps_50: u8,
    pub moves_count: u32,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        'outer: for pos in 0..64 {
            for (piece, position) in &self.position.by_piece {
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

        let position = Bitboards::from_fen_position(position)?;
        let turn = piece::Color::turn_from_str(turn)?;
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

    pub fn castling_rights(&self, side: Color) -> Castle {
        match side {
            Color::Black => self.black_can_castle,
            Color::White => self.white_can_castle,
        }
    }

    pub fn manual_move_is_valid(
        &self,
        player_move: &Move,
        precalculated_resulting_position: &Bitboards,
    ) -> bool {
        // check if the player is moving an opponent piece
        if player_move.piece.color != self.turn {
            return false;
        }

        match player_move.action {
            MoveVariant::Standard { from, to } => {
                // check if there is the piece in the starting square
                let piece_bitboard = self.position.bitboard_by_piece(player_move.piece);
                if piece_bitboard.bits & from.bits == 0 {
                    return false;
                }

                // check if the move is one of the possible generated moves
                let moves_generator = player_move.piece.get_moves_generator();
                let other_side = player_move.piece.color.other();

                let valid_moves = match player_move.piece.kind {
                    piece::Kind::Pawn => moves_generator(
                        from,
                        self.position.occupied_cells(),
                        self.position.squares_occupied_by_color(other_side),
                    ),
                    _ => moves_generator(
                        from,
                        self.position
                            .squares_occupied_by_color(player_move.piece.color),
                        self.position.squares_occupied_by_color(other_side),
                    ),
                };
                if valid_moves.bits & to.bits == 0 {
                    return false;
                }
            }
            MoveVariant::Castle(side) => match (side, self.castling_rights(self.turn)) {
                (CastleSide::King, Castle::Both) | (CastleSide::King, Castle::King) => {
                    return true;
                }
                (CastleSide::Queen, Castle::Both) | (CastleSide::Queen, Castle::Queen) => {
                    return true;
                }
                _ => {
                    return false;
                }
            },
            MoveVariant::Promote { from, to, to_piece } => todo!(),
        }

        // check if we already reached the draw
        if self.reps_50 > 49 {
            return false;
        }

        // check if in the resulting position the player is suiciding the king
        if precalculated_resulting_position.is_in_check(player_move.piece.color) {
            return false;
        }

        true
    }

    /// calculates possibile en passant target generated by the move being made
    pub fn calculate_en_passant_target(&self, player_move: &Move) -> Bitboard {
        match player_move.action {
            MoveVariant::Standard { from, to } => {
                if player_move.piece.kind != piece::Kind::Pawn {
                    return Bitboard { bits: 0 };
                }

                if from.bits << 16 != to.bits && from.bits >> 16 != to.bits {
                    // pawn is not making a 2 squares move
                    return Bitboard { bits: 0 };
                }

                let possible_en_passant_doer =
                    (to.bits << 1 & constants::NOT_H_RANK) | (to.bits >> 1 & constants::NOT_A_RANK);
                match player_move.piece.color {
                    Color::White => {
                        let black_pawns = self
                            .position
                            .bitboard_by_piece(Piece {
                                kind: piece::Kind::Pawn,
                                color: Color::Black,
                            })
                            .bits;

                        if possible_en_passant_doer & black_pawns != 0 {
                            return Bitboard { bits: to.bits >> 8 };
                        }
                    }
                    Color::Black => {
                        let white_pawns = self
                            .position
                            .bitboard_by_piece(Piece {
                                kind: piece::Kind::Pawn,
                                color: Color::White,
                            })
                            .bits;

                        if possible_en_passant_doer & white_pawns != 0 {
                            return Bitboard { bits: to.bits << 8 };
                        }
                    }
                }
                Bitboard { bits: 0 }
            }
            _ => Bitboard { bits: 0 },
        }
    }
    /// calculates how castling rights get changed by the move being made
    fn calculate_castling_rights(&self, player_move: &Move) -> (Castle, Castle) {
        let white_queen_rook: u64 =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000000;
        let white_king_rook: u64 =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001;
        let black_queen_rook: u64 =
            0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
        let black_king_rook: u64 =
            0b00000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

        let white_can_castle = match (player_move.piece.color, player_move.piece.kind) {
            (Color::White, Kind::King) => Castle::No,
            (Color::White, Kind::Rook) => {
                match (self.white_can_castle, player_move.action.clone()) {
                    (Castle::No, _) => Castle::No,
                    (Castle::King, MoveVariant::Standard { from, to: _ }) => {
                        if from.bits & white_king_rook != 0 {
                            Castle::No
                        } else {
                            Castle::King
                        }
                    }
                    (Castle::Queen, MoveVariant::Standard { from, to: _ }) => {
                        if from.bits & white_queen_rook != 0 {
                            Castle::No
                        } else {
                            Castle::Queen
                        }
                    }
                    (Castle::Both, MoveVariant::Standard { from, to: _ }) => {
                        if from.bits & white_king_rook != 0 {
                            Castle::Queen
                        } else if from.bits & white_queen_rook != 0 {
                            Castle::King
                        } else {
                            Castle::Both
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (Color::Black, _) => match (self.white_can_castle, player_move.action.clone()) {
                (Castle::No, _) => Castle::No,
                (
                    Castle::King,
                    MoveVariant::Standard { from: _, to }
                    | MoveVariant::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to.bits & white_king_rook != 0 {
                        Castle::No
                    } else {
                        Castle::King
                    }
                }
                (
                    Castle::Queen,
                    MoveVariant::Standard { from: _, to }
                    | MoveVariant::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to.bits & white_queen_rook != 0 {
                        Castle::No
                    } else {
                        Castle::Queen
                    }
                }
                (
                    Castle::Both,
                    MoveVariant::Standard { from: _, to }
                    | MoveVariant::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to.bits & white_king_rook != 0 {
                        Castle::Queen
                    } else if to.bits & white_queen_rook != 0 {
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
            (Color::Black, Kind::King) => Castle::No,
            (Color::Black, Kind::Rook) => {
                match (self.black_can_castle, player_move.action.clone()) {
                    (Castle::No, _) => Castle::No,
                    (Castle::King, MoveVariant::Standard { from, to: _ }) => {
                        if from.bits & black_king_rook != 0 {
                            Castle::No
                        } else {
                            Castle::King
                        }
                    }
                    (Castle::Queen, MoveVariant::Standard { from, to: _ }) => {
                        if from.bits & black_queen_rook != 0 {
                            Castle::No
                        } else {
                            Castle::Queen
                        }
                    }
                    (Castle::Both, MoveVariant::Standard { from, to: _ }) => {
                        if from.bits & black_king_rook != 0 {
                            Castle::Queen
                        } else if from.bits & black_queen_rook != 0 {
                            Castle::King
                        } else {
                            Castle::Both
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (Color::White, _) => match (self.black_can_castle, player_move.action.clone()) {
                (Castle::No, _) => Castle::No,
                (
                    Castle::King,
                    MoveVariant::Standard { from: _, to }
                    | MoveVariant::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to.bits & black_king_rook != 0 {
                        Castle::No
                    } else {
                        Castle::King
                    }
                }
                (
                    Castle::Queen,
                    MoveVariant::Standard { from: _, to }
                    | MoveVariant::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to.bits & black_queen_rook != 0 {
                        Castle::No
                    } else {
                        Castle::Queen
                    }
                }
                (
                    Castle::Both,
                    MoveVariant::Standard { from: _, to }
                    | MoveVariant::Promote {
                        from: _,
                        to,
                        to_piece: _,
                    },
                ) => {
                    if to.bits & black_king_rook != 0 {
                        Castle::Queen
                    } else if to.bits & black_queen_rook != 0 {
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

    pub fn reset_50_moves(&self, player_move: &Move) -> bool {
        // suppose that the validity check already happened so a piece can not move on a square occupied by another piece of the same color.
        let occupied_cells = self.position.occupied_cells();
        if let MoveVariant::Standard { from: _, to } = player_move.action {
            return player_move.piece.kind == piece::Kind::Pawn
                || (to.bits & occupied_cells.bits != 0);
        } else {
            return false;
        }
    }

    /// Makes a move and updates position, turn, en passant target, castling rights and moves count.
    ///
    /// Does not prevent you to make an illegal move.
    pub fn make_unchecked_move(&self, player_move: &Move) -> Self {
        let position = self.position.make_unchecked_move(player_move);

        let turn = self.turn.other();

        let en_passant_target = self.calculate_en_passant_target(player_move);
        let (white_can_castle, black_can_castle) = self.calculate_castling_rights(&player_move);
        let reps_50 = if self.reset_50_moves(&player_move) {
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

    pub fn there_is_promotion(&self) -> bool {
        self.position
            .bitboard_by_piece(Piece {
                color: Color::White,
                kind: piece::Kind::Pawn,
            })
            .bits
            & EIGHT_ROW
            != 0
            || self
                .position
                .bitboard_by_piece(Piece {
                    color: Color::Black,
                    kind: piece::Kind::Pawn,
                })
                .bits
                & FIRST_ROW
                != 0
    }

    pub fn make_checked_manual_move<T: TryInto<Bitboard>>(
        &self,
        piece: Piece,
        from: T,
        to: T,
    ) -> Result<Self, anyhow::Error>
    where
        anyhow::Error: From<T::Error>,
    {
        let from = from.try_into()?;
        let to = to.try_into()?;

        let player_move = Move {
            piece,
            action: MoveVariant::Standard { from, to },
        };

        let next_board = self.make_unchecked_move(&player_move);
        if !self.manual_move_is_valid(&player_move, &next_board.position) {
            return Err(anyhow!("Illegal move!"));
        }
        Ok(next_board)
    }

    pub fn move_is_capture(&self, player_move: &Move) -> bool {
        let other_side = player_move.piece.color.other();
        if let MoveVariant::Standard { from: _, to } = player_move.action {
            return self.position.squares_occupied_by_color(other_side).bits & to.bits != 0;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_to_square() {
        let cell = "C7";
        let square = Bitboard::try_from(cell).unwrap();
        assert_eq!(
            square.bits,
            0b00000000_00100000_00000000_00000000_00000000_00000000_00000000_00000000
        );

        let cell = "H6";
        let square = Bitboard::try_from(cell).unwrap();
        assert_eq!(
            square.bits,
            0b00000000_00000000_00000001_00000000_00000000_00000000_00000000_00000000
        );
    }

    #[test]
    fn legal_move() {
        let board = Board::from_forsyth_edwards(
            "r1b1kbnr/pppp1ppp/2n2q2/4p3/2BPP3/5N2/PPP2PPP/RNBQK2R b KQkq - 2 4",
        )
        .unwrap();

        let board_after = board.make_checked_manual_move(
            Piece {
                kind: piece::Kind::Knight,
                color: Color::Black,
            },
            "c6",
            "d4",
        );

        assert!(board_after.is_ok());

        let board_after = board.make_checked_manual_move(
            Piece {
                kind: piece::Kind::Pawn,
                color: Color::Black,
            },
            "a7",
            "a5",
        );

        assert!(board_after.is_ok());
    }

    fn illegal_moves() {
        let board = Board::from_forsyth_edwards(
            "r1b1kbnr/pppp1ppp/2n2q2/4p3/2BPP3/5N2/PPP2PPP/RNBQK2R b KQkq - 2 4",
        )
        .unwrap();

        let board_after = board.make_checked_manual_move(
            Piece {
                kind: piece::Kind::Pawn,
                color: Color::White,
            },
            "e4",
            "e5",
        );

        assert!(board_after.is_err());

        let board_after = board.make_checked_manual_move(
            Piece {
                kind: piece::Kind::Knight,
                color: Color::White,
            },
            "f3",
            "g4",
        );

        assert!(board_after.is_err());
    }
}
