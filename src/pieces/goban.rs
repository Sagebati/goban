//! Module with the goban and his implementations.

use crate::pieces::stones::*;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;
use crate::pieces::util::coord::{CoordUtil, Coord, neighbors_coords, Order};
use std::ops::{Index, IndexMut};

///
/// Represents a Goban. With an array with the stones encoded in u8. and the size.
///
#[derive(Clone, Eq, Getters, Setters, Debug)]
pub struct Goban {
    ///
    /// The values are stored in a one dimension vector.
    /// Using the RowMajor Policy.
    ///
    #[get = "pub"]
    #[set]
    tab: Vec<Color>,

    ///
    /// For future repr
    ///
    #[get = "pub"]
    b_stones: Vec<bool>,
    #[get = "pub"]
    w_stones: Vec<bool>,

    #[get = "pub"]
    #[set]
    size: usize,

    #[get]
    #[set]
    coord_util: CoordUtil,
}


impl Goban {
    pub fn new(size: usize) -> Self {
        Goban {
            tab: vec![Color::None; size * size],
            size,
            coord_util: CoordUtil::new(size, size),
            b_stones: vec![false; size * size],
            w_stones: vec![false; size * size],
        }
    }

    pub fn new_with_order(size: usize, order: Order) -> Self {
        Goban {
            tab: vec![Color::None; size * size],
            size,
            coord_util: CoordUtil::new_order(size, size, order),
            b_stones: vec![false; size * size],
            w_stones: vec![false; size * size],
        }
    }

    ///
    /// Creates a goban from an array of stones.
    ///
    pub fn from_array(stones: &[Color], order: Order) -> Self {
        let size = ((stones.len() as f32).sqrt()) as usize;
        let mut g = Goban::new_with_order(size, order);
        let coord_util = CoordUtil::new_order(size, size, order);
        stones.iter().enumerate().map(|k|
            {
                (coord_util.from(k.0), *k.1)
            }).for_each(|coord_color| {
            g.push(&coord_color.0, coord_color.1).expect("Play the stone");
        });
        g
    }

    ///
    /// Removes all the stones from the goban.
    ///
    pub fn clear(&mut self) {
        self.tab = vec![Color::None; self.size * self.size];
    }

    ///
    /// Put a stones in the goban. The coord depends on the order choose.
    /// default (line, colomn)
    /// the (0,0) coord is in the top left.
    ///
    pub fn push(&mut self, coord: &Coord, color: Color) -> Result<&mut Goban, String> {
        if self.coord_valid(coord) {
            let i = self.coord_util.to(coord);
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
            self[*coord] = color;
            Ok(self)
        } else {
            Err(format!("the coord :({},{}) are outside the goban", coord.0, coord.1))
        }
    }

    ///
    /// Put many stones.
    ///
    #[inline]
    pub fn push_many<'a>(&'a mut self, coords: impl Iterator<Item=&'a Coord>, value: Color) {
        coords.for_each(|c| {
            self.push(c, value).expect("Add one of the stones to the goban.");
        })
    }

    #[inline]
    pub fn push_stone(&mut self, stone: &Stone) -> Result<&mut Goban, String> {
        self.push(&stone.coord, stone.color)
    }

    ///
    /// Get all the neighbors to the coordinate
    ///
    #[inline]
    pub fn get_neighbors(&self, coord: &Coord) -> impl Iterator<Item=Stone> + '_ {
        neighbors_coords(coord)
            .into_iter()
            .filter(move |x| self.coord_valid(x))
            .map(move |x| Stone { coord: x, color: self[x] })
    }

    ///
    /// Get all the stones that are neighbor to the coord except empty intersections
    ///
    #[inline]
    pub fn get_neighbors_stones(&self, coord: &Coord) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(coord)
            .filter(|s| s.color != Color::None)
    }

    ///
    /// Get all the stones except "Empty stones"
    ///
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item=Stone> + '_ {
        let coord_util = CoordUtil::new(self.size, self.size);
        self.tab.iter()
            .enumerate()
            .filter(|(_index, t)| **t != Color::None)
            .map(move |(index, t)|
                Stone { coord: coord_util.from(index), color: *t })
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
            .map(move |(index, t)|
                Stone { coord: self.coord_util.from(index), color: *t })
    }

    ///
    /// Returns the empty stones connected to the point
    ///
    #[inline]
    pub fn get_liberties(&self, point: &Stone) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(&point.coord)
            .filter(|s| s.color == Color::None)
    }

    ///
    /// Returns the number of liberties. of the stone
    ///
    #[inline]
    pub fn get_nb_liberties(&self, point: &Stone) -> u8 {
        self.get_liberties(point).count() as u8
    }

    ///
    /// Returns true if the stone has liberties.
    ///
    #[inline]
    pub fn has_liberties(&self, point: &Stone) -> bool {
        self.get_liberties(point).any(|s| Color::None == s.color)
    }

    ///
    /// Get a string for printing the goban in the memory shape (0,0) right top
    ///
    pub fn raw_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size {
            for j in 0..self.size {
                buff.push(
                    match self[(i, j)] {
                        Color::White => WHITE_STONE,
                        Color::Black => BLACK_STONE,
                        Color::None => EMPTY_STONE,
                    }
                );
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
                buff.push(
                    match self[(i, j)] {
                        Color::White => WHITE_STONE,
                        Color::Black => BLACK_STONE,
                        Color::None => EMPTY_STONE,
                    }
                );
            }
            buff.push('\n');
        }
        buff
    }

    ///
    /// Return true if the coord is in the goban.
    ///
    #[inline]
    fn coord_valid(&self, coord: &Coord) -> bool {
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
        other.tab == self.tab
    }
}

impl Index<Coord> for Goban {
    type Output = Color;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.tab[self.coord_util.to(&index)]
    }
}

impl IndexMut<Coord> for Goban {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.tab[self.coord_util.to(&index)]
    }
}
