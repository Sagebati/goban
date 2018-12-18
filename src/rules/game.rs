use crate::pieces::goban::*;
use crate::pieces::util::Coord;
use crate::pieces::stones::EMPTY_ID;

pub enum SizeGoban {
    Nineteen = 19,
    Nine = 9,
}

pub enum Errors {
    Atari,
    Ko,
}

pub struct Game {
    goban: Goban,
    legals : Vec<Coord>
}

impl Game {
    fn new(size: SizeGoban) -> Game {
        Game { goban: Goban::new(size as usize) }
    }

    fn new_game(&mut self) {
        self.goban.clear();
    }

    pub fn gameover() {}

    pub fn legals(&self) -> Vec<Coord> {
        self.pseudo_legals()
    }

    fn pseudo_legals(&self) -> Vec<Coord> {
        let mut res = Vec::new();
        for i in 0..self.goban.get_size() {
            for j in 0..self.goban.get_size() {
                if self.goban.get(&(i, j)) == EMPTY_ID {
                    res.push((i, j));
                }
            }
        }
        res
    }
}
