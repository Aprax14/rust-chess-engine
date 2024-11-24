use std::fmt;

use anyhow::{anyhow, Context};
use strum_macros::EnumIter;

use crate::moves::attack;

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

impl Color {
    pub fn turn_from_str(s: &str) -> Result<Self, anyhow::Error> {
        match s.to_lowercase().as_str() {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(anyhow!("Invalid color kind: {}", s)),
        }
    }

    pub fn other(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Kind {
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Piece {
    pub color: Color,
    pub kind: Kind,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.kind, &self.color) {
            (Kind::Pawn, Color::White) => write!(f, "♙"),
            (Kind::Knight, Color::White) => write!(f, "♘"),
            (Kind::Bishop, Color::White) => write!(f, "♗"),
            (Kind::Rook, Color::White) => write!(f, "♖"),
            (Kind::Queen, Color::White) => write!(f, "♕"),
            (Kind::King, Color::White) => write!(f, "♔"),
            (Kind::Pawn, Color::Black) => write!(f, "♟"),
            (Kind::Knight, Color::Black) => write!(f, "♞"),
            (Kind::Bishop, Color::Black) => write!(f, "♝"),
            (Kind::Rook, Color::Black) => write!(f, "♜"),
            (Kind::Queen, Color::Black) => write!(f, "♛"),
            (Kind::King, Color::Black) => write!(f, "♚"),
        }
    }
}

impl Piece {
    pub fn from_char(c: &char) -> Result<Self, anyhow::Error> {
        match c {
            'p' => Ok(Piece {
                color: Color::Black,
                kind: Kind::Pawn,
            }),
            'n' => Ok(Piece {
                color: Color::Black,
                kind: Kind::Knight,
            }),
            'b' => Ok(Piece {
                color: Color::Black,
                kind: Kind::Bishop,
            }),
            'r' => Ok(Piece {
                color: Color::Black,
                kind: Kind::Rook,
            }),
            'q' => Ok(Piece {
                color: Color::Black,
                kind: Kind::Queen,
            }),
            'k' => Ok(Piece {
                color: Color::Black,
                kind: Kind::King,
            }),
            'P' => Ok(Piece {
                color: Color::White,
                kind: Kind::Pawn,
            }),
            'N' => Ok(Piece {
                color: Color::White,
                kind: Kind::Knight,
            }),
            'B' => Ok(Piece {
                color: Color::White,
                kind: Kind::Bishop,
            }),
            'R' => Ok(Piece {
                color: Color::White,
                kind: Kind::Rook,
            }),
            'Q' => Ok(Piece {
                color: Color::White,
                kind: Kind::Queen,
            }),
            'K' => Ok(Piece {
                color: Color::White,
                kind: Kind::King,
            }),
            _ => Err(anyhow!("invalid piece: {}", c)),
        }
    }

    pub fn get_attacks_generator(&self) -> impl Fn(Bitboard, Bitboard, Bitboard) -> Bitboard {
        match (&self.kind, &self.color) {
            (Kind::Pawn, Color::White) => attack::white_pawn_attack,
            (Kind::Pawn, Color::Black) => attack::black_pawn_attack,
            (Kind::Knight, _) => attack::knight,
            (Kind::Bishop, _) => attack::bishop,
            (Kind::Rook, _) => attack::rook,
            (Kind::Queen, _) => attack::queen,
            (Kind::King, _) => attack::king,
        }
    }

    pub fn get_moves_generator(&self) -> impl Fn(Bitboard, Bitboard, Bitboard) -> Bitboard {
        match (&self.kind, &self.color) {
            (Kind::Pawn, Color::White) => attack::white_pawn,
            (Kind::Pawn, Color::Black) => attack::black_pawn,
            (Kind::Knight, _) => attack::knight,
            (Kind::Bishop, _) => attack::bishop,
            (Kind::Rook, _) => attack::rook,
            (Kind::Queen, _) => attack::queen,
            (Kind::King, _) => attack::king,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bitboard {
    pub bits: u64,
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
    /// parse from (x, y) coordinates
    /// starting from (0, 0)
    fn from((x, y): (u8, u8)) -> Self {
        Self {
            bits: 1 << ((7 - x) + (y - 1) * 8),
        }
    }
}

impl TryFrom<&str> for Bitboard {
    type Error = anyhow::Error;
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
    pub fn single_squares(&self) -> Vec<Self> {
        let mut counter = self.bits;
        let mut accumulator = Vec::new();
        while counter != 0 {
            accumulator.push(Self {
                bits: 1 << counter.trailing_zeros(),
            });
            counter &= counter - 1;
        }
        accumulator
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
