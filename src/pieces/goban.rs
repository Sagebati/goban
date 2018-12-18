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
            tab: vec![EMPTY_ID; size * size],
            history: Vec::new(),
            size,
        }
    }

    pub fn clear(&mut self) {
        self.tab = vec![EMPTY_ID; self.size * self.size];
        self.history = Vec::new();
    }

    pub fn play(&mut self, coord: &Coord, turn: bool) {
        if !self.coord_valid(coord) {
            panic!("Play outside the pieces")
        }
        let c = CoordUtil::new(self.size,  self.size);
        let y = if turn {
            WHITE_ID
        } else {
            BLACK_ID
        };
        self.tab[c.to(coord)] = y;
    }

    pub fn get(&self, coord: &Coord) -> u8 {
        if !self.coord_valid(coord) {
            panic!("Coord out of bouds")
        }
        let c = CoordUtil::new(self.size, self.size);
        return *self.tab.get(c.to(coord)).unwrap();
    }

    fn coord_valid(&self, coord: &Coord) -> bool {
        if coord.0 > self.size || coord.1 > self.size {
            return false;
        }
        true
    }

    pub const fn get_size(&self)->usize{
        self.size
    }

    pub fn pretty_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size {
            for j in 0..self.size {
                buff.push(
                    match self.get(&(i, j)) {
                        WHITE_ID => WHITE_STONE,
                        BLACK_ID => BLACK_STONE,
                        EMPTY_ID => EMPTY_STONE,
                        _ => panic!("Error this is not possible")
                    }
                );
            }
            buff.push('\n');
        }
        buff
    }
}

