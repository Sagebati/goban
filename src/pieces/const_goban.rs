//! Module with the goban and his implementations.

use crate::pieces::stones::*;
use crate::pieces::util::coord::{neighbors_coords, Coord, CoordUtil, Order};
use crate::pieces::zobrist::*;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::ops::{Index, IndexMut};

///
/// Represents a Goban. With an array with the stones encoded in u8. and the size.
/// only square boards are possible for the moment.
///
#[derive(Clone, Getters, Setters, Debug)]
pub struct Goban<const L: usize, const N: usize> {
    ///
    /// The values are stored in a one dimension vector.
    /// Using the RowMajor Policy.
    ///
    #[get = "pub"]
    #[set]
    tab: [Color; L * N],

    ///
    /// For future repr
    ///
    #[get = "pub"]
    b_stones: [bool; L * N],
    #[get = "pub"]
    w_stones: [bool; L * N],

    #[get = "pub"]
    #[set]
    size: usize,

    #[get]
    coord_util: CoordUtil,

    zobrist: &'static ZobristTable,

    #[get = "pub"]
    hash: u64,
}

impl<const L: usize, const N: usize> Goban<{L},{N}> {
    pub fn new() -> Self {
        Goban {
            tab: [Color::None; L * N],
            size: L*N,
            coord_util: CoordUtil::new(size, size),
            b_stones: [false; L * N],
            w_stones: [false; L * N],
            zobrist: &ZOBRIST19,
            hash: 0,
        }
    }

    ///
    /// Creates a goban from an array of stones.
    ///
    pub fn from_array(stones: &[Color], order: Order) -> Self {
        let size = ((stones.len() as f32).sqrt()) as usize;
        let mut g = Goban::new(size);
        let coord_util = CoordUtil::new_order(size, size, order);
        stones
            .iter()
            .enumerate()
            .map(|k| {
                // k.0 is the index of the coord
                // k.1 is the color
                (coord_util.from(k.0), k.1)
            })
            .filter(|s| *(*s).1 != Color::None)
            .for_each(|coord_color| {
                g.push(coord_color.0, *coord_color.1)
                    .expect("Play the stone");
            });
        g
    }

    ///
    /// Put a stones in the goban. The point depends on the order choose.
    /// default (line, column)
    /// the (0,0) point is in the top left.
    ///
    pub fn push(&mut self, point: Coord, color: Color) -> Result<&mut Goban, String> {
        if self.coord_valid(point) {
            let i = self.coord_util.to(point);
            match color {
                Color::Black => {
                    self.b_stones[i] = true;
                }
                Color::White => {
                    self.w_stones[i] = true;
                }
                Color::None => {
                    self.b_stones[i] = false;
                    self.w_stones[i] = false;
                }
            }
            if color == Color::None {
                self.hash ^= self.zobrist[(point, self[point])];
            } else {
                self.hash ^= self.zobrist[(point, color)];
            }
            self[point] = color;
            Ok(self)
        } else {
            Err(format!(
                "the point :({},{}) are outside the goban",
                point.0, point.1
            ))
        }
    }

    ///
    /// Put many stones.
    ///
    #[inline]
    pub fn push_many(&mut self, coords: impl Iterator<Item=Coord>, value: Color) {
        coords.for_each(|c| {
            self.push(c, value)
                .expect("Add one of the stones to the goban.");
        })
    }

    #[inline]
    pub fn push_stone(&mut self, stone: Stone) -> Result<&mut Goban, String> {
        self.push(stone.coordinates, stone.color)
    }

    ///
    /// Get all the neighbors to the coordinate
    ///
    #[inline]
    pub fn get_neighbors(&self, coord: Coord) -> impl Iterator<Item=Stone> + '_ {
        neighbors_coords(coord)
            .into_iter()
            .filter(move |x| self.coord_valid(*x))
            .map(move |x| Stone {
                coordinates: x,
                color: self[x],
            })
    }

    ///
    /// Get all the stones that are neighbor to the coord except empty intersections
    ///
    #[inline]
    pub fn get_neighbors_stones(&self, coord: Coord) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(coord).filter(|s| s.color != Color::None)
    }

    ///
    /// Get all the stones except "Empty stones"
    ///
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item=Stone> + '_ {
        let coord_util = CoordUtil::new(self.size, self.size);
        self.tab
            .iter()
            .enumerate()
            .filter(|(_index, t)| **t != Color::None)
            .map(move |(index, t)| Stone {
                coordinates: coord_util.from(index),
                color: *t,
            })
    }

    ///
    /// Get stones by their color.
    ///
    #[inline]
    pub fn get_stones_by_color(&self, color: Color) -> impl Iterator<Item=Stone> + '_ {
        self.tab
            .iter()
            .enumerate()
            .filter(move |(_index, t)| **t == color)
            .map(move |(index, t)| Stone {
                coordinates: self.coord_util.from(index),
                color: *t,
            })
    }

    ///
    /// Returns the empty stones connected to the point
    ///
    #[inline]
    pub fn get_liberties(&self, point: Stone) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(point.coordinates)
            .filter(|s| s.color == Color::None)
    }

    ///
    /// Returns the number of liberties of the stone.
    ///
    #[inline]
    pub fn get_nb_liberties(&self, point: Stone) -> u8 {
        self.get_liberties(point).count() as u8
    }

    ///
    /// Returns true if the stone has liberties.
    ///
    #[inline]
    pub fn has_liberties(&self, point: Stone) -> bool {
        self.get_liberties(point).any(|s| Color::None == s.color)
    }

    ///
    /// Get a string for printing the goban in the memory shape (0,0) right top
    ///
    pub fn raw_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size {
            for j in 0..self.size {
                buff.push(match self[(i, j)] {
                    Color::White => WHITE_STONE,
                    Color::Black => BLACK_STONE,
                    Color::None => EMPTY_STONE,
                });
            }
            buff.push('\n');
        }
        buff
    }

    ///
    /// Get a string for printing the goban in normal shape (0,0 ) left bottom
    ///
    pub fn pretty_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size {
            for j in 0..self.size {
                buff.push(match self[(i, j)] {
                    Color::White => WHITE_STONE,
                    Color::Black => BLACK_STONE,
                    Color::None => EMPTY_STONE,
                });
            }
            buff.push('\n');
        }
        buff
    }

    ///
    /// Get number of stones on the goban.
    /// (number of black stones, number of white stones)
    ///
    pub fn number_of_stones(&self) -> (u32, u32) {
        let mut res: (u32, u32) = (0, 0);
        self.get_stones().for_each(|stone| match stone.color {
            Color::Black => {
                res.0 += 1;
            }
            Color::White => {
                res.1 += 1;
            }
            _ => unreachable!(),
        });
        res
    }

    ///
    /// Calculates a score for the endgame. It's a naive implementation, it counts only
    /// territories with the same color surrounding them.
    ///
    /// Returns (black territory,  white territory)
    ///
    pub fn calculate_territories(&self) -> (f32, f32) {
        let mut scores: (f32, f32) = (0., 0.); // Black & White
        let empty_strings = self.get_strings_from_stones(self.get_stones_by_color(Color::None));
        for group in empty_strings {
            let mut neutral = (false, false);
            for empty_intersection in &group {
                for stone in self.get_neighbors(empty_intersection.coordinates) {
                    if stone.color == Color::White {
                        neutral.1 = true; // found white stone
                    }
                    if stone.color == Color::Black {
                        neutral.0 = true; // found black stone
                    }
                }
            }
            if neutral.0 && !neutral.1 {
                scores.0 += group.len() as f32;
            } else if !neutral.0 && neutral.1 {
                scores.1 += group.len() as f32;
            }
        }
        (scores.0, scores.1)
    }

    ///
    /// Return true if the coord is in the goban.
    ///
    #[inline]
    fn coord_valid(&self, coord: Coord) -> bool {
        coord.0 < self.size && coord.1 < self.size
    }
}

impl Display for Goban {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.pretty_string())
    }
}

impl PartialEq for Goban {
    fn eq(&self, other: &Goban) -> bool {
        other.hash == self.hash
    }
}

impl Eq for Goban {}

impl Index<Coord> for Goban {
    type Output = Color;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.tab[self.coord_util.to(index)]
    }
}

impl IndexMut<Coord> for Goban {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.tab[self.coord_util.to(index)]
    }
}

impl Default for Goban {
    fn default() -> Self {
        Goban::new(19)
    }
}
