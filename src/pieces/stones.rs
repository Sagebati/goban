use crate::pieces::util::Coord;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;

pub const WHITE_STONE: char = '⚫';
pub const BLACK_STONE: char = '⚪';
pub const EMPTY_STONE: char = '.';

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum StoneColor {
    White = 2,
    Black = 1,
    Empty = 0,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Stone {
    pub coord: Coord,
    pub color: StoneColor,
}

impl From<u8> for StoneColor {
    fn from(x: u8) -> Self {
        match x {
            2 => StoneColor::White,
            1 => StoneColor::Black,
            0 => StoneColor::Empty,
            _ => panic!("Error int the conversion from u8 to Stone")
        }
    }
}

impl Into<u8> for StoneColor{
    fn into(self) -> u8 {
        match self {
            StoneColor::Black => 1,
            StoneColor::Empty => 0,
            StoneColor::White => 2,
        }
    }
}

impl From<bool> for StoneColor {
    fn from(x: bool) -> Self {
        match x {
            true => StoneColor::White,
            false => StoneColor::Black,
        }
    }
}

impl Into<bool> for StoneColor {
    fn into(self) -> bool {
        match self {
            StoneColor::Black => false,
            StoneColor::White => true,
            StoneColor::Empty => panic!("Tried to convert Empty to a turn")
        }
    }
}

impl Display for StoneColor {
    fn fmt(&self, f: &mut Formatter<>) -> Result<(), Error> {
        let color_str = match self {
            StoneColor::White => "White",
            StoneColor::Black => "Black",
            StoneColor::Empty => "Empty"
        };
        write!(f, "{}", color_str)
    }
}
