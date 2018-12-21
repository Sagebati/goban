use crate::pieces::util::*;
use crate::pieces::stones::*;


#[derive(Clone)]
pub struct Goban {
    turn: bool,
    tab: Vec<u8>,
    history: Vec<Coord>,
    size: usize,
}


impl Goban {
    pub fn new(size: usize) -> Goban {
        Goban {
            turn: true,
            tab: vec![StoneColor::Empty as u8; size * size],
            history: Vec::new(),
            size,
        }
    }

    pub fn clear(&mut self) {
        self.tab = vec![StoneColor::Empty as u8; self.size * self.size];
        self.history = Vec::new();
    }

    pub fn play(&mut self, coord: &Coord, turn: bool) -> &mut Goban {
        if !self.coord_valid(coord) {
            panic!("Play outside the pieces")
        }
        let c = CoordUtil::new(self.size, self.size);
        let y = if turn {
            StoneColor::White as u8
        } else {
            StoneColor::Black as u8
        };
        self.tab[c.to(coord)] = y;
        self
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

    pub fn set(&mut self, coord: &Coord, value: StoneColor) {
        self.tab[CoordUtil::new(self.size, self.size).to(coord)] = value as u8;
    }

    pub fn set_many<'a>(&'a mut self, coords: impl Iterator<Item=&'a Coord>, value: StoneColor) {
        coords.for_each(|c| self.set(c, value))
    }

    pub const fn get_size(&self) -> usize {
        self.size
    }

    ///
    /// Get all the neighbors to the coordinate
    ///
    pub fn get_neighbors(&self, coord: &Coord) -> Vec<Stone> {
        let mut res = Vec::new();
        for c in neighbors_connected(coord) {
            if c.0 < self.size && c.1 < self.size {
                res.push(Stone { coord: c.clone(), color: self.get(&c) })
            }
        }
        res
    }
    ///
    /// Get all the stones that are neighbor to the coord except empty intersections
    ///
    pub fn get_stones_neghboors(&self, coord: &Coord) -> Vec<Stone> {
        let mut res = Vec::new();
        for c in neighbors_connected(coord) {
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
                    res.push(Stone { coord: (i, j), color:*color })
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

    pub fn get_history(&self) -> &Vec<Coord> {
        &self.history
    }

    pub fn pop_play(&mut self) -> &mut Self {
        match self.history.pop() {
            Some(coord) => self.set(&coord, StoneColor::Empty),
            None => eprintln!("The goban has not plays to pop")
        }
        self
    }
}

impl PartialEq for Goban {
    fn eq(&self, other: &Goban) -> bool {
        other.tab == self.tab
    }
}

impl Eq for Goban {}

