use crate::pieces::goban::*;
use crate::pieces::stones::Stones;
use crate::pieces::goban::Point;
use std::collections::HashSet;

pub enum GobanSizes {
    Nineteen = 19,
    Nine = 9,
    Thirteen = 13,
}

#[derive(Copy, Clone)]
pub enum Rules {
    Japanese,
    Chinese,
}

#[derive(Copy, Clone)]
pub enum Conflicts {
    Ko,
}

#[derive(Copy, Clone)]
pub enum Move {
    Pass,
    Play(usize, usize),
}

#[derive(Copy, Clone)]
pub enum EndGame {
    WhiteW,
    BlackW,
    Equality,
}

struct Passes {
    first: bool,
    second: bool,
}

impl Passes {
    pub fn new() -> Self {
        Passes { first: false, second: false }
    }

    pub fn two_passes(&self) -> bool {
        self.first && self.second
    }

    pub fn pass(&mut self) {
        if self.first {
            self.second = true;
        } else {
            self.first = false;
        }
    }
    pub fn no_pass(&mut self) {
        self.first = false;
    }
}

pub struct Game {
    goban: Goban,
    passes: Passes,
    turn: bool,
    komi: f32,
    rules: Rules,
}

impl Game {
    pub fn new(size: GobanSizes) -> Game {
        let goban = Goban::new(size as usize);
        let komi = 5.5;
        let pass = Passes::new();
        Game { goban, turn: false, komi, passes: pass, rules: Rules::Japanese }
    }

    pub const fn get_goban(&self) -> &Goban {
        &self.goban
    }

    pub const fn get_turn(&self) -> bool {
        self.turn
    }

    pub fn set_komi(&mut self, komi: f32) {
        self.komi = komi;
    }

    pub fn set_rules(&mut self, rule: Rules) {
        self.rules = rule;
    }

    pub fn get_rules(&self) -> Rules {
        self.rules
    }
}

impl Game {
    ///
    /// Reset the game.
    ///
    pub fn new_game(&mut self) {
        self.goban.clear();
    }

    ///
    /// True when the game is over (two passes, or no more legals moves)
    ///
    pub fn gameover(&self) -> bool {
        self.legals().is_empty() || self.passes.two_passes()
    }

    ///
    /// Returns a list with legals moves,
    /// In the list will appear suicides moves, and ko moves.
    /// Ko moves are analysed when a play occurs.
    ///
    pub fn legals(&self) -> Vec<Move> {
        let mut legals = self.pseudo_legals();
        if !legals.is_empty() {
            legals.push(Move::Pass);
        }
        legals
    }

    ///
    /// Method to play on the goban or pass,
    /// Return a conflict (Ko) if the move cannot be performed
    ///
    pub fn play(&mut self, play: &Move) -> Option<Conflicts> {
        let mut res = None;
        match *play {
            Move::Pass => {
                self.passes.pass();
            }
            Move::Play(x, y) => {
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

    pub fn end_game(&self) -> Option<EndGame> {
        if self.gameover() {
            None
        } else {
            Some(EndGame::BlackW)
        }
    }

    ///
    /// Generate all moves on all intersections.
    ///
    fn pseudo_legals(&self) -> Vec<Move> {
        let mut res = Vec::new();
        for i in 0..self.goban.get_size() {
            for j in 0..self.goban.get_size() {
                if self.goban.get(&(i, j)) == Stones::Empty {
                    res.push(Move::Play(i, j));
                }
            }
        }
        res
    }


    ///
    /// Removes stones in atari from the goban.
    ///
    fn atari(&mut self) {
        let atari_stones: Vec<Point> = self.goban
            .get_stones().into_iter()
            // get all stones without liberties
            .filter(|point| !self.goban.has_liberties(point))
            .collect();

        let mut list_of_groups_stones: Vec<HashSet<Point>> = Vec::new();
        for atari_stone in atari_stones {

            // if the stone is already in a group of stones
            let is_handled = list_of_groups_stones.iter()
                .any(|set| set.contains(&atari_stone));

            if !is_handled{
                list_of_groups_stones.push(self.bfs(&atari_stone))
            }
        }

        for groups_of_stones in list_of_groups_stones {
            let mut is_atari = true;
            for stone in &groups_of_stones {
                if self.goban.has_liberties(&stone) {
                    is_atari = false;
                    break;
                }
            }
            if is_atari {
                for stone in groups_of_stones {
                    self.goban.set(&stone.coord, Stones::Empty);
                }
            }
        }
    }

    ///
    /// Can get a group of stones and his neigboors with a bfs,
    /// works for Empty stones too.
    ///
    fn bfs(&self, point: &Point) -> HashSet<Point> {
        let mut explored: HashSet<Point> = HashSet::new();
        explored.insert(point.clone());

        let mut to_explore: Vec<Point> = self.goban.get_neighbors(&point.coord)
            .into_iter()
            .filter(|p| p.stone == point.stone)
            .collect(); // Acquiring all the neighbors

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
