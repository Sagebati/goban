//! Module with all needed to play.

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

use crate::pieces::util::coord::Coord;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
#[repr(u8)]
pub enum Color {
    White = 2,
    Black = 1,
}

pub type MaybeColor = Option<Color>;

pub const EMPTY: Option<Color> = None;

impl std::ops::Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

impl From<MaybeColor> for Color {
    fn from(x: MaybeColor) -> Self {
        match x {
            Some(x) => x,
            EMPTY => panic!("Cannot transform an empty point ot a color"),
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
        }
    }
}

/// Stone on a goban.
#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub struct Point {
    pub coord: Coord,
    pub color: MaybeColor,
}

impl Point {
    #[inline]
    pub fn is_empty(self) -> bool {
        self.color.is_none()
    }
}

/// Stone on a goban.
#[derive(PartialEq, Eq, Hash, Clone, Debug, Copy)]
pub struct Stone {
    pub coord: Coord,
    pub color: Color,
}

impl From<Point> for Stone {
    fn from(x: Point) -> Self {
        Stone {
            coord: x.coord,
            color: x
                .color
                .expect("We cannot transform an empty point to a stone"),
        }
    }
}
