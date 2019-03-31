//! Module with all needed to play.

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;
use crate::pieces::util::coord::Coord;

pub const WHITE_STONE: char = '⚫';
pub const BLACK_STONE: char = '⚪';
pub const EMPTY_STONE: char = '.';

/// Color on the goban.
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Color {
    White = 2,
    Black = 1,
    None = 0,
}

/// Stone on a goban.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Stone {
    pub coord: Coord,
    pub color: Color,
}

impl From<u8> for Color {
    fn from(x: u8) -> Self {
        match x {
            2 => Color::White,
            1 => Color::Black,
            0 => Color::None,
            _ => panic!("Error int the conversion from u8 to Stone")
        }
    }
}

impl From<bool> for Color {
    fn from(x: bool) -> Self {
        match x {
            true => Color::White,
            false => Color::Black,
        }
    }
}

///
///  "true" turn belongs to White
///  "false" turn belongs to Black
///
impl Into<bool> for Color {
    fn into(self) -> bool {
        match self {
            Color::White => true,
            Color::Black => false,
            Color::None => panic!("Tried to convert Empty to a turn")
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<>) -> Result<(), Error> {
        let color_str = match self {
            Color::White => "White",
            Color::Black => "Black",
            Color::None => "Empty"
        };
        write!(f, "{}", color_str)
    }
}
