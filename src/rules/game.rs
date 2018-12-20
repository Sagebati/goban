use crate::pieces::goban::*;
use crate::pieces::util::Coord;
use crate::pieces::stones::Stones;
use crate::pieces::goban::Point;
use std::collections::HashSet;

pub enum GobanSizes {
    Nineteen = 19,
    Nine = 9,
    Thirteen = 13,
}

pub enum Rules {
    Japanese,
    Chinese,
}

pub enum Conflicts {
    Ko,
}

pub enum Plays {
    Pass,
    Play(usize, usize),
}

pub enum EndGames {
    WhiteW,
    BlackW,
}

struct Passes {
    firt_pass: bool,
    second_pass: bool,
}

impl Passes {
    pub fn new() -> Self {
        Passes { firt_pass: false, second_pass: false }
    }

    pub fn two_passes(&self) -> bool {
        self.firt_pass && self.second_pass
    }

    pub fn pass(&mut self) {
        if self.firt_pass {
            self.second_pass = true;
        } else {
            self.firt_pass = false;
        }
    }
    pub fn no_pass(&mut self) {
        self.firt_pass = false;
    }
}

pub struct Game {
    goban: Goban,
    turn: bool,
    komi: f32,
    passes: Passes,
}

impl Game {
    pub fn new(size: GobanSizes) -> Game {
        let goban = Goban::new(size as usize);
        let komi = 5.5;
        let pass = Passes::new();
        Game { goban, turn: false, komi, passes: pass }
    }

    fn new_game(&mut self) {
        self.goban.clear();
    }

    pub fn gameover(&self) -> bool {
        self.legals().is_empty() || self.passes.two_passes()
    }

    pub fn legals(&self) -> Vec<Plays> {
        let mut legals = self.pseudo_legals();
        if !legals.is_empty() {
            legals.push(Plays::Pass);
        }
        legals
    }


    pub fn play(&mut self, play: &Plays) -> Option<Conflicts> {
        let mut res = None;
        match *play {
            Plays::Pass => {
                self.passes.pass();
            }
            Plays::Play(x, y) => {
                let mut tmp_goban = self.goban.clone();
                tmp_goban.play(&(x, y), self.turn);
                res = if tmp_goban == self.goban {
                    Some(Conflicts::Ko)
                } else {
                    self.goban = tmp_goban;
                    self.turn = !self.turn;
                    self.passes.no_pass();
                    None
                };
                self.atari();
            }
        }
        res
    }

    fn pseudo_legals(&self) -> Vec<Plays> {
        let mut res = Vec::new();
        for i in 0..self.goban.get_size() {
            for j in 0..self.goban.get_size() {
                if self.goban.get(&(i, j)) == Stones::Empty {
                    res.push(Plays::Play(i, j));
                }
            }
        }
        res
    }

    pub const fn get_goban(&self) -> &Goban {
        &self.goban
    }

    pub fn atari(&mut self) {
        let atari_stones: Vec<HashSet<Point>> = self.goban
            .get_stones().into_iter()
            .filter(|point| !self.goban.has_liberties(point))
            .map(|p| self.bfs(&p))
            .collect();

        for strong_connex in atari_stones {
            let mut is_atari = true;
            for point in &strong_connex {
                if self.goban.has_liberties(&point) {
                    is_atari = false;
                    break;
                }
            }
            if is_atari {
                for point in strong_connex {
                    self.goban.set(&point.coord, Stones::Empty);
                }
            }
        }
    }

    fn bfs(&self, point: &Point) -> HashSet<Point> {
        let mut explored: HashSet<Point> = HashSet::new();
        explored.insert(point.clone());

        let mut to_explore: Vec<Point> = self.goban.get_neighbors(&point.coord)
            .into_iter()
            .filter(|p| p.stone == point.stone)
            .collect(); // Aquiring all the neigbors

        while let Some(point_to_explore) = to_explore.pop() { // exploring the graph
            explored.insert(point_to_explore);
            let neighbors: Vec<Point> = self.goban.get_neighbors(&point.coord)
                .into_iter()
                .filter(|p| p.stone == point.stone && !explored.contains(p))
                .collect();
            for p in neighbors {
                to_explore.push(p);
            }
        }
        explored
    }
}
