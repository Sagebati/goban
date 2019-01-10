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
            Err("Play outside the goban".into())
        }
    }

    pub fn push_stone(&mut self, stone: &Stone) -> Result<&mut Goban, String> {
        self.push(&stone.coord, stone.color)
    }

    pub fn get(&self, coord: &Coord) -> StoneColor {
        if !self.coord_valid(coord) {
            panic!("Coord out of bouds")
        }
        let c = CoordUtil::new(self.size, self.size);

        self.tab[c.to(coord)].into()
    }

    fn coord_valid(&self, coord: &Coord) -> bool {
        if coord.0 < self.size || coord.1 < self.size {
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
    pub fn get_neighbors(&self, coord: &Coord) -> Vec<Stone> {
        let mut res = Vec::new();
        for c in neighbors_coords(coord) {
            if c.0 < self.size && c.1 < self.size {
                res.push(Stone { coord: c.clone(), color: self.get(&c) })
            }
        }
        res
    }
    ///
    /// Get all the stones that are neighbor to the coord except empty intersections
    ///
    pub fn get_neighbors_stones(&self, coord: &Coord) -> Vec<Stone> {
        let mut res = Vec::new();
        for c in neighbors_coords(coord) {
            if c.0 < self.size && c.1 < self.size {
                let s = self.get(&c);
                if s != StoneColor::Empty {
                    res.push(Stone { coord: c.clone(), color: self.get(&c) })
                }
            }
        }
        res
    }

    pub fn get_stones(&self) -> Vec<Stone> {
        let mut res = Vec::new();
        for i in 0..self.size {
            for j in 0..self.size {
                let x = self.get(&(i, j));
                if x != StoneColor::Empty {
                    res.push(Stone { coord: (i, j), color: x })
                }
            }
        }
        res
    }

    pub fn get_stones_by_color(&self, color: &StoneColor) -> Vec<Stone> {
        let mut res = Vec::new();
        for i in 0..self.size {
            for j in 0..self.size {
                if self.get(&(i, j)) == *color {
                    res.push(Stone { coord: (i, j), color: *color })
                }
            }
        }
        res
    }

    pub fn get_liberties(&self, point: &Stone) -> u8 {
        let liberties: Vec<Stone> = self.get_neighbors(&point.coord).into_iter()
            .filter(|p| p.color == StoneColor::Empty)
            .collect();
        liberties.len() as u8
    }

    pub fn has_liberties(&self, point: &Stone) -> bool {
        self.get_liberties(point) != 0
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

