//! Module with all needed to play.

use crate::pieces::util::coord::Coord;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

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
pub struct Point<T = MaybeColor> {
    pub coord: Coord,
    pub color: T,
}

pub type Stone = Point<Color>;

impl TryFrom<Point> for Stone {
    type Error = &'static str;

    fn try_from(value: Point) -> Result<Self, Self::Error> {
        if let Some(color) = value.color {
            Ok(Stone {
                coord: value.coord,
                color,
            })
        } else {
            Err("We cannot transform an empty point to a stone")
        }
    }
}

impl From<(Coord, Color)> for Stone {
    fn from((coord, color): (Coord, Color)) -> Self {
        Stone { coord, color }
    }
}

impl From<(Coord, MaybeColor)> for Point {
    fn from((coord, color): (Coord, MaybeColor)) -> Self {
        Point { coord, color }
    }
}

impl<T> From<Point<T>> for Coord {
    fn from(value: Point<T>) -> Self {
        value.coord
    }
}
