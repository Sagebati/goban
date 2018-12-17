use bitvec::{BitVec, bitvec, BigEndian};
use std::ops::Index;
use core::borrow::Borrow;

const ORDER: Order = Order::RowMajor;

type Coord = (usize, usize);

pub mod stone {
    pub const WHITE: char = '⚫';
    pub const BLACK: char = '⚪';
    pub const EMPTY: char = '.';
}

pub enum SizeGoban {
    Nineteen = 19,
    Nine = 9,
}

pub enum Order {
    RowMajor,
    ColumnMajor,
}

pub struct CoordUtil {
    n_rows: usize,
    n_cols: usize,
}

impl CoordUtil {
    pub fn new(n_rows: usize, n_cols: usize) -> CoordUtil {
        CoordUtil { n_cols, n_rows }
    }

    pub fn to(&self, coord: Coord) -> usize {
        match ORDER {
            Order::ColumnMajor => {
                (coord.0 * self.n_cols + coord.1 % self.n_rows)
            }
            Order::RowMajor => {
                (coord.0 * self.n_rows + coord.1 % self.n_cols)
            }
        }
    }

    pub fn from(&self, index: usize) -> Coord {
        match ORDER {
            Order::ColumnMajor => {
                (index / self.n_cols, index % self.n_rows)
            }
            Order::RowMajor => {
                (index / self.n_rows, index % self.n_cols)
            }
        }
    }
}

pub struct Goban {
    tab: BitVec,
    history: Vec<CoordUtil>,
    size: usize,
}

impl Index<Coord> for Goban {
    type Output = bool;

    fn index(&self, index: Coord) -> &<Self as Index<Coord>>::Output {
        let c = CoordUtil::new(self.size, self.size);

        self.tab.get(c.to(index)).borrow()
    }
}

impl Goban {
    pub fn new(size: usize) -> Goban {
        Goban {
            tab: bitvec![0;size*size],
            history: Vec::new(),
            size,
        }
    }

    pub fn clear(&mut self) {
        self.tab = bitvec![0;self.size*self.size];
        self.history = Vec::new();
    }

    fn play(&mut self, coord: Coord) {
        if coord.0 > self.size || coord.1 > self.size {
            panic!("Play outside the goban")
        }
        let c = CoordUtil { n_cols: self.size, n_rows: self.size };
        self.tab.set(c.to(coord), true);
    }

    fn pretty_string(&self) -> &str {
        let mut buff = String::new();
        for i in 0..self.size {
            for j in 0..self.size {
                buff.push(
                    if *self.index((i, j)) {
                        stone::BLACK
                    } else {
                        stone::EMPTY
                    }
                )
            }
            buff.push('\n');
        }

        buff.as_str()
    }
}



