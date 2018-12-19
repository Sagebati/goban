use crate::pieces::goban::*;
use crate::pieces::util::Coord;
use crate::pieces::stones::Stones;
use crate::pieces::goban::Point;
use std::collections::HashSet;

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
        let legals = self.pseudo_legals();
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
        self.atari();
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
