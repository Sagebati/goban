use bitvec::{BitVec, bitvec, BigEndian};
use std::collections::VecDeque;
use std::ops::Shl;
use std::ops::ShlAssign;

const ORDER: Order = Order::RowMajor;

pub enum SizeGoban {
    Nineteen = 19,
    Nine = 9,
}


pub enum Order {
    RowMajor,
    ColumnMajor,
}

pub struct Coord {
    pub x: usize,
    pub y: usize,
    n_rows: usize,
    n_cols: usize,
}

impl Coord {
    pub fn set(mut self, coord: (usize, usize)) -> Coord {
        self.x = coord.0;
        self.y = coord.1;
        self
    }

    pub fn from(mut self, index: usize) -> Coord {
        match ORDER {
            Order::ColumnMajor => {
                self.x = index / self.n_cols;
                self.y = index % self.n_rows;
            }
            Order::RowMajor => {
                self.x = index / self.n_rows;
                self.y = index % self.n_cols;
            }
        };
        self
    }
}


pub struct Goban {
    tab: BitVec,
    history: Vec<Coord>,
}

impl Goban {
    pub fn new(size: usize) -> Goban {
        Goban {
            tab: bitvec![0;size*usize],
            history: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.tab = bitvec![0;size];
        self.history = Vec::new();
    }

    fn play(&mut self, coord: Coord) {
        if coord.x > size || coord.y > size {
            panic!("Play outside the goban")
        }
        self.tab.set(coord.into(), true);
    }
}

impl Into<usize> for Coord {
    fn into(self) -> usize {
        match ORDER {
            Order::RowMajor => self.x * self.n_rows + self.y % self.n_cols,
            Order::ColumnMajor => self.x * self.n_cols + self.y % self.n_rows
        }
    }
}

impl Into<(usize, usize)> for Coord {
    fn into(self) -> (usize, usize) {
        (self.x, self.y)
    }
}

