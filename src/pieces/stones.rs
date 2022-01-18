//! Module with all needed to play.

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

use crate::pieces::util::coord::Point;

pub const fn u8_to_color (x: u8) -> Color{
    match x {
        2 => Color::White,
        1 => Color::Black,
        0 => Color::None,
        _ => panic!("IS not a valid u8 for color")
    }
}

/// Color on the goban.
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Color {
    White = 2,
    Black = 1,
    None = 0,
}

/// Stone on a goban.
#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub struct Stone {
    pub coordinates: Point,
    pub color: Color,
}

impl From<u8> for Color {
    fn from(x: u8) -> Self {
        debug_assert!(x <= 2, "Error in the conversion from u8 to stone");
        u8_to_color(x)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let color_str = match self {
            Color::White => "White",
            Color::Black => "Black",
            Color::None => "Empty",
        };
        write!(f, "{}", color_str)
    }
}
