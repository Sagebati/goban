use crate::pieces::goban::*;
use crate::pieces::util::Coord;
use crate::pieces::stones::Stones;

pub enum SizeGoban {
    Nineteen = 19,
    Nine = 9,
}

pub enum Errors {
    Ko,
}

pub enum EndGames {
    WhiteW,
    BlackW,
}

pub struct Game {
    goban: Goban,
    turn: bool,
    tmp: Goban,
    komi: f32,
}

impl Game {
    pub fn new(size: SizeGoban) -> Game {
        let goban = Goban::new(size as usize);
        let tmp = goban.clone();
        let komi = 5.5;
        Game { goban, turn: false, tmp, komi }
    }

    fn new_game(&mut self) {
        self.goban.clear();
    }

    pub fn gameover(&self) -> bool {
        self.legals().is_empty()
    }

    pub fn legals(&self) -> Vec<Coord> {
        let mut legals = self.pseudo_legals();
        legals
    }



    pub fn play(&mut self, coord: &Coord) -> Option<Errors> {
        self.goban.play(coord, self.turn);
        self.turn = !self.turn;
        let res = if self.tmp == self.goban {
            Some(Errors::Ko)
        } else {
            None
        };
        self.tmp = self.goban.clone();
        res
    }

    fn pseudo_legals(&self) -> Vec<Coord> {
        let mut res = Vec::new();
        for i in 0..self.goban.get_size() {
            for j in 0..self.goban.get_size() {
                if self.goban.get(&(i, j)) == Stones::Empty {
                    res.push((i, j));
                }
            }
        }
        res
    }

    pub fn get_goban(&self) -> &Goban {
        &self.goban
    }

    pub fn atari(&self){
        let stones = if self.turn{
            self.goban.get_stones_by_color(Stones::White)
        }else {
            self.goban.get_stones_by_color(Stones::Black)
        };



    }
}
