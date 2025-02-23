use std::{fmt, ops};

use anyhow::{Context, anyhow};
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Black => write!(f, "Black"),
            Color::White => write!(f, "White"),
        }
    }
}

impl TryFrom<&str> for Color {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(anyhow!("Invalid color kind: {}", s)),
        }
    }
}

impl Color {
    pub fn other(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub fn value(&self) -> i32 {
        match self {
            Self::Pawn => 1000,
            Self::Knight => 3000,
            Self::Bishop => 3100,
            Self::Rook => 5000,
            Self::Queen => 9000,
            Self::King => 1_000_000_000,
        }
    }

    pub fn attacked_value(&self) -> i32 {
        match self {
            Self::Pawn => 100,
            Self::Knight => 300,
            Self::Bishop => 310,
            Self::Rook => 500,
            Self::Queen => 900,
            Self::King => 1000,
        }
    }
}

// make it Copy cause a reference to Piece (usize) is 64 bits while Piece itself is 16 bits.
#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub fn new(color: Color, kind: PieceKind) -> Self {
        Piece { color, kind }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.kind, &self.color) {
            (PieceKind::Pawn, Color::White) => write!(f, "♙"),
            (PieceKind::Knight, Color::White) => write!(f, "♘"),
            (PieceKind::Bishop, Color::White) => write!(f, "♗"),
            (PieceKind::Rook, Color::White) => write!(f, "♖"),
            (PieceKind::Queen, Color::White) => write!(f, "♕"),
            (PieceKind::King, Color::White) => write!(f, "♔"),
            (PieceKind::Pawn, Color::Black) => write!(f, "♟"),
            (PieceKind::Knight, Color::Black) => write!(f, "♞"),
            (PieceKind::Bishop, Color::Black) => write!(f, "♝"),
            (PieceKind::Rook, Color::Black) => write!(f, "♜"),
            (PieceKind::Queen, Color::Black) => write!(f, "♛"),
            (PieceKind::King, Color::Black) => write!(f, "♚"),
        }
    }
}

impl TryFrom<char> for Piece {
    type Error = anyhow::Error;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'p' => Ok(Piece {
                color: Color::Black,
                kind: PieceKind::Pawn,
            }),
            'n' => Ok(Piece {
                color: Color::Black,
                kind: PieceKind::Knight,
            }),
            'b' => Ok(Piece {
                color: Color::Black,
                kind: PieceKind::Bishop,
            }),
            'r' => Ok(Piece {
                color: Color::Black,
                kind: PieceKind::Rook,
            }),
            'q' => Ok(Piece {
                color: Color::Black,
                kind: PieceKind::Queen,
            }),
            'k' => Ok(Piece {
                color: Color::Black,
                kind: PieceKind::King,
            }),
            'P' => Ok(Piece {
                color: Color::White,
                kind: PieceKind::Pawn,
            }),
            'N' => Ok(Piece {
                color: Color::White,
                kind: PieceKind::Knight,
            }),
            'B' => Ok(Piece {
                color: Color::White,
                kind: PieceKind::Bishop,
            }),
            'R' => Ok(Piece {
                color: Color::White,
                kind: PieceKind::Rook,
            }),
            'Q' => Ok(Piece {
                color: Color::White,
                kind: PieceKind::Queen,
            }),
            'K' => Ok(Piece {
                color: Color::White,
                kind: PieceKind::King,
            }),
            _ => Err(anyhow!("invalid piece: {}", c)),
        }
    }
}

pub struct SingleSquareIterator {
    bits: u64,
}

impl Iterator for SingleSquareIterator {
    type Item = Bitboard;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }

        let piece = self.bits & self.bits.wrapping_neg();
        self.bits &= self.bits - 1;

        Some(Bitboard::new(piece))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bitboard {
    pub bits: u64,
}

impl ops::Shl<u32> for Bitboard {
    type Output = Self;

    fn shl(self, shift: u32) -> Self::Output {
        Self {
            bits: self.bits << shift,
        }
    }
}

impl ops::Shr<u32> for Bitboard {
    type Output = Self;

    fn shr(self, shift: u32) -> Self::Output {
        Self {
            bits: self.bits >> shift,
        }
    }
}

impl ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits & rhs.bits,
        }
    }
}

impl ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

impl ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits ^ rhs.bits,
        }
    }
}

impl ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { bits: !self.bits }
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for pos in 0..64 {
            if (self.bits << pos).leading_zeros() == 0_u32 {
                write!(f, " 1 ")?;
            } else {
                write!(f, " 0 ")?;
            }
            if (pos + 1) % 8 == 0 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl From<u8> for Bitboard {
    /// parse from offset
    fn from(n: u8) -> Self {
        Self { bits: 1 << n }
    }
}

impl From<(u8, u8)> for Bitboard {
    /// parse Bitboard from (x, y) coordinates
    /// starting from (0, 0)
    fn from((x, y): (u8, u8)) -> Self {
        Self {
            bits: 1 << ((7 - x) + (y - 1) * 8),
        }
    }
}

impl TryFrom<&str> for Bitboard {
    type Error = anyhow::Error;
    /// parse a square as square from his coordinates.
    /// the returned Bitboard is all 0s exept for the parsed square.
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut chars = s.chars();
        let column_number = chars
            .next()
            .context("empty coordinates")?
            .to_ascii_uppercase() as u8
            - 64;
        let row_number = chars
            .next()
            .context("invalid coordinates")?
            .to_digit(10)
            .context("invalid row digit")? as u8;

        Ok(Self {
            bits: 1 << (8 - column_number + (row_number - 1) * 8),
        })
    }
}

impl Bitboard {
    pub fn new(bits: u64) -> Self {
        Self { bits }
    }

    pub fn single_squares(&self) -> SingleSquareIterator {
        SingleSquareIterator { bits: self.bits }
    }

    pub fn count_bits(&self) -> i32 {
        let mut count = 0;
        let mut counter = self.bits;
        while counter != 0 {
            count += 1;
            counter &= counter - 1;
        }
        count
    }
}
