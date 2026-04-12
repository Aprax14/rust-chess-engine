use super::{
    castle, constants, en_passant,
    pieces::{Bitboard, Color, Piece, PieceKind},
};
use crate::moves::{
    generators,
    move_type::{Move, MoveKind},
};

#[derive(Debug, Clone)]
pub struct BBPosition {
    white_pawn: Bitboard,
    white_knight: Bitboard,
    white_bishop: Bitboard,
    white_rook: Bitboard,
    white_queen: Bitboard,
    white_king: Bitboard,
    black_pawn: Bitboard,
    black_knight: Bitboard,
    black_bishop: Bitboard,
    black_rook: Bitboard,
    black_queen: Bitboard,
    black_king: Bitboard,
}

impl<'a> IntoIterator for &'a BBPosition {
    type Item = (&'a Piece, &'a Bitboard);
    type IntoIter = std::array::IntoIter<Self::Item, 12>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (
                &Piece {
                    color: Color::White,
                    kind: PieceKind::Pawn,
                },
                &self.white_pawn,
            ),
            (
                &Piece {
                    color: Color::White,
                    kind: PieceKind::Knight,
                },
                &self.white_knight,
            ),
            (
                &Piece {
                    color: Color::White,
                    kind: PieceKind::Bishop,
                },
                &self.white_bishop,
            ),
            (
                &Piece {
                    color: Color::White,
                    kind: PieceKind::Rook,
                },
                &self.white_rook,
            ),
            (
                &Piece {
                    color: Color::White,
                    kind: PieceKind::Queen,
                },
                &self.white_queen,
            ),
            (
                &Piece {
                    color: Color::White,
                    kind: PieceKind::King,
                },
                &self.white_king,
            ),
            (
                &Piece {
                    color: Color::Black,
                    kind: PieceKind::Pawn,
                },
                &self.black_pawn,
            ),
            (
                &Piece {
                    color: Color::Black,
                    kind: PieceKind::Knight,
                },
                &self.black_knight,
            ),
            (
                &Piece {
                    color: Color::Black,
                    kind: PieceKind::Bishop,
                },
                &self.black_bishop,
            ),
            (
                &Piece {
                    color: Color::Black,
                    kind: PieceKind::Rook,
                },
                &self.black_rook,
            ),
            (
                &Piece {
                    color: Color::Black,
                    kind: PieceKind::Queen,
                },
                &self.black_queen,
            ),
            (
                &Piece {
                    color: Color::Black,
                    kind: PieceKind::King,
                },
                &self.black_king,
            ),
        ]
        .into_iter()
    }
}

impl BBPosition {
    pub fn empty() -> Self {
        Self {
            white_pawn: Bitboard::new(0),
            white_knight: Bitboard::new(0),
            white_bishop: Bitboard::new(0),
            white_rook: Bitboard::new(0),
            white_queen: Bitboard::new(0),
            white_king: Bitboard::new(0),
            black_pawn: Bitboard::new(0),
            black_knight: Bitboard::new(0),
            black_bishop: Bitboard::new(0),
            black_rook: Bitboard::new(0),
            black_queen: Bitboard::new(0),
            black_king: Bitboard::new(0),
        }
    }

    pub fn get<T>(&self, piece: T) -> Bitboard
    where
        T: TryInto<Piece>,
        T::Error: std::fmt::Debug,
    {
        let piece = piece.try_into().expect("piece conversion failed");
        match (piece.color, piece.kind) {
            (Color::White, PieceKind::Pawn) => self.white_pawn,
            (Color::White, PieceKind::Knight) => self.white_knight,
            (Color::White, PieceKind::Bishop) => self.white_bishop,
            (Color::White, PieceKind::Rook) => self.white_rook,
            (Color::White, PieceKind::Queen) => self.white_queen,
            (Color::White, PieceKind::King) => self.white_king,
            (Color::Black, PieceKind::Pawn) => self.black_pawn,
            (Color::Black, PieceKind::Knight) => self.black_knight,
            (Color::Black, PieceKind::Bishop) => self.black_bishop,
            (Color::Black, PieceKind::Rook) => self.black_rook,
            (Color::Black, PieceKind::Queen) => self.black_queen,
            (Color::Black, PieceKind::King) => self.black_king,
        }
    }

    pub fn get_mut<T>(&mut self, piece: T) -> &mut Bitboard
    where
        T: TryInto<Piece>,
        T::Error: std::fmt::Debug,
    {
        let piece = piece.try_into().expect("piece conversion failed");
        match (piece.color, piece.kind) {
            (Color::White, PieceKind::Pawn) => &mut self.white_pawn,
            (Color::White, PieceKind::Knight) => &mut self.white_knight,
            (Color::White, PieceKind::Bishop) => &mut self.white_bishop,
            (Color::White, PieceKind::Rook) => &mut self.white_rook,
            (Color::White, PieceKind::Queen) => &mut self.white_queen,
            (Color::White, PieceKind::King) => &mut self.white_king,
            (Color::Black, PieceKind::Pawn) => &mut self.black_pawn,
            (Color::Black, PieceKind::Knight) => &mut self.black_knight,
            (Color::Black, PieceKind::Bishop) => &mut self.black_bishop,
            (Color::Black, PieceKind::Rook) => &mut self.black_rook,
            (Color::Black, PieceKind::Queen) => &mut self.black_queen,
            (Color::Black, PieceKind::King) => &mut self.black_king,
        }
    }

    pub fn from_fen_notation(s: &str) -> Result<Self, anyhow::Error> {
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
                let piece: Piece = c.try_into()?;
                bb.get_mut(piece).bits |= 1 << index;
                index -= 1;
            }
        }

        Ok(bb)
    }

    pub fn occupied_cells(&self) -> Bitboard {
        Bitboard::new(self.into_iter().fold(0u64, |acc, (_, pos)| acc | pos.bits))
    }

    pub fn empty_cells(&self) -> Bitboard {
        Bitboard::new(!self.occupied_cells().bits)
    }

    pub fn occupied_by(&self, c: Color) -> Bitboard {
        Bitboard::new(
            self.into_iter()
                .filter(|(piece, _)| piece.color == c)
                .fold(0u64, |acc, (_, pos)| acc | pos.bits),
        )
    }

    /// Returns `(ours, enemies)` in a single pass over the 12 bitboards.
    pub fn occupied_by_both(&self, color: Color) -> (Bitboard, Bitboard) {
        let (ours, enemies) =
            self.into_iter()
                .fold((0u64, 0u64), |(ours, enemies), (piece, bb)| {
                    if piece.color == color {
                        (ours | bb.bits, enemies)
                    } else {
                        (ours, enemies | bb.bits)
                    }
                });

        (Bitboard::new(ours), Bitboard::new(enemies))
    }

    pub fn piece_at(&self, left_shift: u8) -> Option<Piece> {
        for (piece, bitboard) in self.into_iter() {
            if bitboard.bits & (1 << left_shift) != 0 {
                return Some(*piece);
            }
        }

        None
    }

    /// Generates all possible captures by piece.
    /// This can be called with a single piece Bitboard (a Bitboard with just one single 1 inside its u64)
    /// or with a multi-pieces Bitboard.
    pub fn captures(&self, piece: Piece, piece_position: Bitboard) -> Bitboard {
        let (our_squares, enemies) = self.occupied_by_both(piece.color);
        let occupied = Bitboard::new(our_squares.bits | enemies.bits);
        match (piece.kind, piece.color) {
            (PieceKind::Pawn, Color::White) => {
                generators::white_pawn_attack(piece_position, Bitboard::new(0), enemies)
            }
            (PieceKind::Pawn, Color::Black) => {
                generators::black_pawn_attack(piece_position, Bitboard::new(0), enemies)
            }
            (PieceKind::Knight, _) => {
                generators::knight(piece_position, occupied, enemies) & enemies
            }
            (PieceKind::Bishop, _) => {
                generators::bishop(piece_position, occupied, enemies) & enemies
            }
            (PieceKind::Rook, _) => generators::rook(piece_position, occupied, enemies) & enemies,
            (PieceKind::Queen, _) => generators::queen(piece_position, occupied, enemies) & enemies,
            (PieceKind::King, _) => generators::king(piece_position, occupied, enemies) & enemies,
        }
    }

    /// Returns all attacked squares.
    /// Not all the squares are legal moves.
    ///
    /// For example, a pawn can attack a square but, if that squares does not contain an enemy piece,
    /// the pawn can't actually go there. Can be usefull when evaluating the king's moves, to be sure
    /// you are not suiciding the king.
    ///
    /// Can be called with Bitboards of multiple pieces of a kind.
    pub fn attacks(&self, piece: Piece, piece_position: Bitboard) -> Bitboard {
        let (our_squares, enemies) = self.occupied_by_both(piece.color);
        match (piece.kind, piece.color) {
            (PieceKind::Pawn, Color::White) => generators::white_pawn_attack(
                piece_position,
                Bitboard::new(0),
                Bitboard::new(u64::MAX),
            ),
            (PieceKind::Pawn, Color::Black) => generators::black_pawn_attack(
                piece_position,
                Bitboard::new(0),
                Bitboard::new(u64::MAX),
            ),
            (PieceKind::Knight, _) => generators::knight(piece_position, our_squares, enemies),
            (PieceKind::Bishop, _) => generators::bishop(piece_position, our_squares, enemies),
            (PieceKind::Rook, _) => generators::rook(piece_position, our_squares, enemies),
            (PieceKind::Queen, _) => generators::queen(piece_position, our_squares, enemies),
            (PieceKind::King, _) => generators::king(piece_position, our_squares, enemies),
        }
    }

    /// Returns all defended squares.
    ///
    /// Can be called with Bitboards of multiple pieces of a kind.
    pub fn defenses(&self, piece: Piece, piece_position: Bitboard) -> Bitboard {
        let (our_squares, enemies) = self.occupied_by_both(piece.color);
        let attacked_and_defended = match (piece.kind, piece.color) {
            (PieceKind::Pawn, Color::White) => generators::white_pawn_attack(
                piece_position,
                Bitboard::new(0),
                Bitboard::new(u64::MAX),
            ),
            (PieceKind::Pawn, Color::Black) => generators::black_pawn_attack(
                piece_position,
                Bitboard::new(0),
                Bitboard::new(u64::MAX),
            ),
            (PieceKind::Knight, _) => generators::knight(piece_position, Bitboard::new(0), enemies),
            (PieceKind::Bishop, _) => {
                generators::bishop(piece_position, Bitboard::new(0), our_squares | enemies)
            }
            (PieceKind::Rook, _) => {
                generators::rook(piece_position, Bitboard::new(0), our_squares | enemies)
            }
            (PieceKind::Queen, _) => {
                generators::queen(piece_position, Bitboard::new(0), our_squares | enemies)
            }
            (PieceKind::King, _) => {
                generators::king(piece_position, Bitboard::new(0), our_squares | enemies)
            }
        };

        attacked_and_defended & our_squares
    }

    /// Returns all the possible moves for a piece.
    /// Can be called with Bitboards containing more than 1 piece of a kind.
    pub fn available_moves(&self, piece: Piece, piece_position_left_shift: u8) -> Bitboard {
        let (our_squares, enemies) = self.occupied_by_both(piece.color);
        let occupied = Bitboard::new(our_squares.bits | enemies.bits);
        let piece_position = Bitboard::new(1 << piece_position_left_shift);

        match (piece.kind, piece.color) {
            (PieceKind::Pawn, Color::White) => {
                generators::white_pawn(piece_position, occupied | enemies, enemies)
            }
            (PieceKind::Pawn, Color::Black) => {
                generators::black_pawn(piece_position, occupied | enemies, enemies)
            }
            (PieceKind::Knight, _) => generators::knight(piece_position, our_squares, enemies),
            (PieceKind::Bishop, _) => generators::bishop(piece_position, our_squares, enemies),
            (PieceKind::Rook, _) => generators::rook(piece_position, our_squares, enemies),
            (PieceKind::Queen, _) => generators::queen(piece_position, our_squares, enemies),
            (PieceKind::King, _) => generators::king(piece_position, our_squares, enemies),
        }
    }
    /// Returns a Bitboard where the 1s rapresent the squares attacked by a side
    pub fn attacked_squares(&self, color: Color) -> Bitboard {
        self.into_iter()
            .filter(|(piece, _)| piece.color == color)
            .map(|(piece, position)| self.attacks(*piece, *position))
            .fold(Bitboard::new(0), |acc, x| acc | x)
    }

    /// Returns a Bitboard where the 1s represent the defended squares
    pub fn defended_squares(&self, color: Color) -> Bitboard {
        self.into_iter()
            .filter(|(piece, _)| piece.color == color)
            .map(|(piece, positions)| self.defenses(*piece, *positions))
            .fold(Bitboard::new(0), |acc, x| acc | x)
    }

    /// Returns true if at least one of the squares in the Bitboard is defended
    pub fn square_is_defended_by(&self, square: u8, color: Color) -> bool {
        self.defended_squares(color).bits & (1 << square) != 0
    }

    pub fn is_in_check(&self, side: Color) -> bool {
        self.get(match side {
            Color::White => 'K',
            Color::Black => 'k',
        }) & self.attacked_squares(side.other())
            != Bitboard::new(0)
    }

    /// Returns true if the moving side's king is in check after a standard (or promotion) move.
    /// Avoids cloning the full position by doing reverse ray-casting from the king's square.
    pub fn is_in_check_after_standard_move(&self, from: u8, to: u8, moving_piece: Piece) -> bool {
        self.is_in_check_after_move_internal(
            from,
            to,
            to,
            moving_piece.color,
            moving_piece.kind == PieceKind::King,
        )
    }

    /// Returns true if the moving side's king is in check after an en-passant capture.
    pub fn is_in_check_after_en_passant(&self, from: u8, to: u8, moving_color: Color) -> bool {
        let captured_sq = match moving_color {
            Color::White => to - 8,
            Color::Black => to + 8,
        };
        self.is_in_check_after_move_internal(from, to, captured_sq, moving_color, false)
    }

    fn is_in_check_after_move_internal(
        &self,
        from: u8,
        to: u8,
        captured_sq: u8,
        moving_color: Color,
        king_moved: bool,
    ) -> bool {
        let enemy_color = moving_color.other();

        let king_sq = if king_moved {
            to
        } else {
            self.get(Piece::new(moving_color, PieceKind::King))
                .bits
                .trailing_zeros() as u8
        };
        let king_bb = Bitboard::new(1u64 << king_sq);

        let (our_orig, enemy_orig) = self.occupied_by_both(moving_color);

        // compute the new occupation bitboard after the move
        let our_new = (our_orig.bits & !(1u64 << from)) | (1u64 << to);
        let enemy_new = enemy_orig.bits & !(1u64 << captured_sq);

        // All occupied squares except the king.
        let occ_no_king = Bitboard::new((our_new | enemy_new) & !(1u64 << king_sq));

        // Get diagonal mover (bishop or queen)
        let enemy_diag = (self.get(Piece::new(enemy_color, PieceKind::Bishop)).bits
            | self.get(Piece::new(enemy_color, PieceKind::Queen)).bits)
            & enemy_new;
        // Check if an enemy diagonal mover hits the king
        if enemy_diag != 0 {
            let diag_blockers = Bitboard::new(occ_no_king.bits & !enemy_diag);
            let enemy_diag_bb = Bitboard::new(enemy_diag);
            // The idea here is to simulate a king moving like a bishop.
            // By symmetry, if the ray reaches an enemy bishop/queen it means that
            // that piece can also reach the king. One cast from the king is enough making this
            // more efficient that one cast for every enemy bishop and queen.
            if generators::bishop(king_bb, diag_blockers, enemy_diag_bb).bits & enemy_diag != 0 {
                return true;
            }
        }

        // Orthogonal moving pieces (rook + queen)
        let enemy_ortho = (self.get(Piece::new(enemy_color, PieceKind::Rook)).bits
            | self.get(Piece::new(enemy_color, PieceKind::Queen)).bits)
            & enemy_new;
        if enemy_ortho != 0 {
            let ortho_blockers = Bitboard::new(occ_no_king.bits & !enemy_ortho);
            let enemy_ortho_bb = Bitboard::new(enemy_ortho);
            if generators::rook(king_bb, ortho_blockers, enemy_ortho_bb).bits & enemy_ortho != 0 {
                return true;
            }
        }

        // Knights
        let enemy_knights = self.get(Piece::new(enemy_color, PieceKind::Knight)).bits & enemy_new;
        if enemy_knights != 0
            && generators::knight(king_bb, Bitboard::new(0), Bitboard::new(0)).bits & enemy_knights
                != 0
        {
            return true;
        }

        // Pawns
        let enemy_pawns = self.get(Piece::new(enemy_color, PieceKind::Pawn)).bits & enemy_new;
        if enemy_pawns != 0 {
            // Cast from king's square as if it were a pawn of the moving color.
            // The attacked squares are exactly the squares from which an enemy pawn would attack the king.
            let pawn_threats = match moving_color {
                Color::White => generators::white_pawn_attack(
                    king_bb,
                    Bitboard::new(0),
                    Bitboard::new(u64::MAX),
                ),
                Color::Black => generators::black_pawn_attack(
                    king_bb,
                    Bitboard::new(0),
                    Bitboard::new(u64::MAX),
                ),
            };
            if pawn_threats.bits & enemy_pawns != 0 {
                return true;
            }
        }

        // Enemy king (two kings can not be adjacent)
        let enemy_king = self.get(Piece::new(enemy_color, PieceKind::King)).bits;
        if generators::king(king_bb, Bitboard::new(0), Bitboard::new(0)).bits & enemy_king != 0 {
            return true;
        }

        false
    }

    /// Applies a move to the position in place.
    /// Only modifies piece bitboards; all other Board fields must be updated separately.
    pub fn apply_move(&mut self, player_move: &Move) {
        match player_move.action {
            MoveKind::Standard { from, to } => {
                let from_bb = 1u64 << from;
                let to_bb = 1u64 << to;
                if let Some(cap) = self.piece_at(to) {
                    self.get_mut(cap).bits &= !to_bb;
                }
                let piece_bb = self.get_mut(player_move.piece);
                piece_bb.bits = (piece_bb.bits & !from_bb) | to_bb;
            }
            MoveKind::Castle(side) => {
                castle::apply_castling_in_place(self, player_move.piece.color, side);
            }
            MoveKind::EnPassant { .. } => {
                en_passant::apply_en_passant_in_place(self, player_move);
            }
            MoveKind::Promote { from, to, to_piece } => {
                let from_bb = 1u64 << from;
                let to_bb = 1u64 << to;
                if let Some(cap) = self.piece_at(to) {
                    self.get_mut(cap).bits &= !to_bb;
                }
                self.get_mut(player_move.piece).bits &= !from_bb;
                self.get_mut(Piece::new(player_move.piece.color, to_piece))
                    .bits |= to_bb;
            }
        }
    }

    /// Reverses a previously applied move, restoring the captured piece if any.
    pub fn unapply_move(&mut self, player_move: &Move, captured: Option<Piece>) {
        match player_move.action {
            MoveKind::Standard { from, to } => {
                let from_bb = 1u64 << from;
                let to_bb = 1u64 << to;
                let piece_bb = self.get_mut(player_move.piece);
                piece_bb.bits = (piece_bb.bits & !to_bb) | from_bb;
                if let Some(cap) = captured {
                    self.get_mut(cap).bits |= to_bb;
                }
            }
            MoveKind::Castle(side) => {
                castle::unapply_castling_in_place(self, player_move.piece.color, side);
            }
            MoveKind::EnPassant { .. } => {
                en_passant::unapply_en_passant_in_place(self, player_move);
            }
            MoveKind::Promote { from, to, to_piece } => {
                let from_bb = 1u64 << from;
                let to_bb = 1u64 << to;
                self.get_mut(Piece::new(player_move.piece.color, to_piece))
                    .bits &= !to_bb;
                self.get_mut(player_move.piece).bits |= from_bb;
                if let Some(cap) = captured {
                    self.get_mut(cap).bits |= to_bb;
                }
            }
        }
    }

    /// Updates the position after a move is made. This should not be called manually cause
    /// it does not updates all the other fields of a chess board
    pub fn inner_make_unchecked_move(&self, player_move: &Move) -> Self {
        match player_move.action {
            MoveKind::Standard { from, to } => {
                let from_bb = Bitboard::new(1 << from);
                let to_bb = Bitboard::new(1 << to);

                let mut resulting_bitboards = self.clone();
                let piece_bitboard = resulting_bitboards.get_mut(player_move.piece);

                // remove piece from the old position
                *piece_bitboard = *piece_bitboard & !from_bb;

                // set the piece in the new position
                piece_bitboard.bits |= to_bb.bits;

                if let Some(oc) = self.piece_at(to) {
                    resulting_bitboards.get_mut(oc).bits &= !to_bb.bits;
                }

                resulting_bitboards
            }
            MoveKind::Castle(side) => {
                castle::bitboards_after_castling(self, player_move.piece.color, side)
            }
            MoveKind::EnPassant { .. } => en_passant::bitboards_after_en_passant(self, player_move),
            MoveKind::Promote { from, to, to_piece } => {
                let from_bb = Bitboard::new(1 << from);
                let to_bb = Bitboard::new(1 << to);

                let mut resulting_bitboards = self.clone();
                let pawn_bitboard = resulting_bitboards.get_mut(player_move.piece);

                // remove pawn from the old position
                pawn_bitboard.bits &= !from_bb.bits;

                let new_piece_bitboard =
                    resulting_bitboards.get_mut(Piece::new(player_move.piece.color, to_piece));

                // set the piece it is promoting to in the new position
                new_piece_bitboard.bits |= to_bb.bits;

                // remove possible captured pieces
                if let Some(oc) = self.piece_at(to) {
                    resulting_bitboards.get_mut(oc).bits &= !to_bb.bits;
                }

                resulting_bitboards
            }
        }
    }

    /// calculates possibile en passant target generated by the move being made
    pub fn calculate_en_passant_target(&self, player_move: &Move) -> Bitboard {
        match player_move.action {
            MoveKind::Standard { from, to } => {
                let from = Bitboard::new(1 << from);
                let to = Bitboard::new(1 << to);

                if player_move.piece.kind != PieceKind::Pawn {
                    return Bitboard::new(0);
                }

                if from.bits << 16 != to.bits && from.bits >> 16 != to.bits {
                    // pawn is not making a 2 squares move
                    return Bitboard::new(0);
                }

                let possible_en_passant_doer =
                    (to.bits << 1 & constants::NOT_H_RANK) | (to.bits >> 1 & constants::NOT_A_RANK);
                match player_move.piece.color {
                    Color::White => {
                        let black_pawns = self.get('p').bits;
                        if possible_en_passant_doer & black_pawns != 0 {
                            return Bitboard::new(to.bits >> 8);
                        }
                    }
                    Color::Black => {
                        let white_pawns = self.get('P').bits;

                        if possible_en_passant_doer & white_pawns != 0 {
                            return Bitboard::new(to.bits << 8);
                        }
                    }
                }
                Bitboard::new(0)
            }
            _ => Bitboard::new(0),
        }
    }
}
