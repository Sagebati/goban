use crate::pieces::util::*;
use crate::pieces::stones::*;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Error;


#[derive(Clone, Eq)]
pub struct Goban {
    tab: Vec<u8>,
    size: usize,
}


impl Goban {
    pub fn new(size: usize) -> Goban {
        Goban {
            tab: vec![StoneColor::Empty as u8; size * size],
            size,
        }
    }

    pub fn clear(&mut self) {
        self.tab = vec![StoneColor::Empty as u8; self.size * self.size];
    }

    pub fn push(&mut self, coord: &Coord, color: StoneColor) -> Result<&mut Goban, String> {
        if self.coord_valid(coord) {
            self.tab[CoordUtil::new(self.size, self.size).to(coord)] = color as u8;
            Ok(self)
        } else {
            Err(format!("the coord :({},{}) are outside the goban", coord.0, coord.1))
        }
    }

    pub fn push_stone(&mut self, stone: &Stone) -> Result<&mut Goban, String> {
        self.push(&stone.coord, stone.color)
    }

    pub fn get(&self, coord: &Coord) -> StoneColor {
        if !self.coord_valid(coord) {
            panic!("Coord out of bounds")
        }
        let c = CoordUtil::new(self.size, self.size);

        self.tab[c.to(coord)].into()
    }

    fn coord_valid(&self, coord: &Coord) -> bool {
        if coord.0 < self.size && coord.1 < self.size {
            return true;
        }
        false
    }

    ///
    /// Put many stones.
    ///
    pub fn push_many<'a>(&'a mut self, coords: impl Iterator<Item=&'a Coord>, value: StoneColor) {
        coords.for_each(|c| {
            self.push(c, value).expect("Add one\
        of the stones to the goban.");
        })
    }

    /// Removes the last
    pub fn pop(&mut self) -> &mut Self {
        self.tab.pop();
        self
    }

    pub const fn get_size(&self) -> usize {
        self.size
    }

    ///
    /// Get all the neighbors to the coordinate
    ///
    pub fn get_neighbors(&self, coord: &Coord) -> impl Iterator<Item=Stone> + '_ {
        neighbors_coords(coord)
            .into_iter()
            .filter(move |x| self.coord_valid(x))
            .map(move |x| Stone { coord: x.clone(), color: self.get(&x) })
    }
    ///
    /// Get all the stones that are neighbor to the coord except empty intersections
    ///
    pub fn get_neighbors_stones(&self, coord: &Coord) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(coord)
            .filter(|s| s.color != StoneColor::Empty)
    }

    ///
    /// Get all the stones except "Empty stones"
    ///
    pub fn get_stones(&self) -> impl Iterator<Item=Stone> + '_ {
        let coord_util = CoordUtil::new(self.size, self.size);
        self.tab.iter()
            .enumerate()
            .filter(|(_index, t)| **t != StoneColor::Empty.into())
            .map(move |(index, t)|
                Stone { coord: coord_util.from(index), color: (*t).into() })
    }

    ///
    /// Get stones by their color.
    ///
    pub fn get_stones_by_color(&self, color: StoneColor) -> impl Iterator<Item=Stone> + '_ {
        let coord_util = CoordUtil::new(self.size, self.size);
        self.tab
            .iter()
            .enumerate()
            .filter(move |(_index, t)| (**t) ==  color.into())
            .map(move |(index, t)|
                Stone { coord: coord_util.from(index), color: (*t).into() })
    }

    ///
    /// Returns the empty stones connected to the point
    ///
    pub fn get_liberties(&self, point: &Stone) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(&point.coord)
            .filter(|s| s.color == StoneColor::Empty)
    }

    ///
    /// Returns the number of liberties. of the stone
    ///
    pub fn get_nb_liberties(&self, point: &Stone) -> u8 {
        self.get_liberties(point).count() as u8
    }

    ///
    /// Returns true if the stone has liberties.
    ///
    pub fn has_liberties(&self, point: &Stone) -> bool {
        self.get_liberties(point).any(|s| StoneColor::Empty == s.color)
    }

    pub fn pretty_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size {
            for j in 0..self.size {
                buff.push(
                    match self.get(&(i, j)) {
                        StoneColor::White => WHITE_STONE,
                        StoneColor::Black => BLACK_STONE,
                        StoneColor::Empty => EMPTY_STONE,
                    }
                );
            }
            buff.push('\n');
        }
        buff
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

