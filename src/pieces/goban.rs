use crate::pieces::util::*;
use crate::pieces::stones::*;


#[derive(Clone)]
pub struct Goban {
    turn: bool,
    tab: Vec<u8>,
    history: Vec<Coord>,
    size: usize,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Point {
    pub coord: Coord,
    pub stone: Stones,
}

impl Goban {
    pub fn new(size: usize) -> Goban {
        Goban {
            turn: true,
            tab: vec![Stones::Empty as u8; size * size],
            history: Vec::new(),
            size,
        }
    }

    pub fn clear(&mut self) {
        self.tab = vec![Stones::Empty as u8; self.size * self.size];
        self.history = Vec::new();
    }

    pub fn play(&mut self, coord: &Coord, turn: bool) {
        if !self.coord_valid(coord) {
            panic!("Play outside the pieces")
        }
        let c = CoordUtil::new(self.size, self.size);
        let y = if turn {
            Stones::White as u8
        } else {
            Stones::Black as u8
        };
        self.tab[c.to(coord)] = y;
    }

    pub fn get(&self, coord: &Coord) -> Stones {
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

    pub fn set(&mut self, coord: &Coord, value: Stones) {
        self.tab[CoordUtil::new(self.size, self.size).to(coord)] = value as u8;
    }

    pub const fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_neighbors(&self, coord: &Coord) -> Vec<Point> {
        let mut res = Vec::new();
        for c in neighbors_connected(coord) {
            if c.0 < self.size && c.1 < self.size {
                res.push(Point { coord: c.clone(), stone: self.get(&c) })
            }
        }
        res
    }

    pub fn get_neighbors_stones(&self, coord: &Coord) -> Vec<Point> {
        let mut res = Vec::new();
        for c in neighbors_connected(coord) {
            if c.0 < self.size && c.1 < self.size {
                let s = self.get(&c);
                if s != Stones::Empty {
                    res.push(Point { coord: c.clone(), stone: self.get(&c) })
                }
            }
        }
        res
    }

    pub fn get_neighbors_bulk(&self, coords: &Vec<Coord>) -> Vec<Vec<Point>> {
        coords.iter().map(|c| self.get_neighbors(c)).collect()
    }

    pub fn get_stones(&self) -> Vec<Point> {
        let mut res = Vec::new();
        for i in 0..self.size {
            for j in 0..self.size {
                let x = self.get(&(i, j));
                if x != Stones::Empty {
                    res.push(Point { coord: (i, j), stone: x })
                }
            }
        }
        res
    }

    pub fn get_stones_by_color(&self, color: &Stones) -> Vec<Coord> {
        let mut res = Vec::new();
        for i in 0..self.size {
            for j in 0..self.size {
                if self.get(&(i, j)) == *color {
                    res.push((i, j))
                }
            }
        }
        res
    }

    pub fn get_liberties(&self, point: &Point) -> u8 {
        4 - self.get_neighbors_stones(&point.coord).len() as u8
    }

    pub fn has_liberties(&self, point: &Point) -> bool {
        self.get_liberties(point) != 0
    }

    pub fn pretty_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size {
            for j in 0..self.size {
                buff.push(
                    match self.get(&(i, j)) {
                        Stones::White => WHITE_STONE,
                        Stones::Black => BLACK_STONE,
                        Stones::Empty => EMPTY_STONE,
                    }
                );
            }
            buff.push('\n');
        }
        buff
    }
}

impl PartialEq for Goban {
    fn eq(&self, other: &Goban) -> bool {
        other.tab == self.tab
    }
}

impl Eq for Goban {}

