use crate::pieces::goban::*;
use crate::pieces::stones::StoneColor;
use std::collections::HashSet;
use crate::pieces::stones::Stone;

pub enum GobanSizes {
    Nineteen,
    Nine,
    Thirteen,
    Custom(usize),
}

impl Into<usize> for GobanSizes {
    fn into(self) -> usize {
        match self {
            GobanSizes::Nine => 9,
            GobanSizes::Custom(size) => size,
            GobanSizes::Nineteen => 19,
            GobanSizes::Thirteen => 13,
        }
    }
}


#[derive(Copy, Clone)]
pub enum Rules {
    Japanese,
    Chinese,
    Aga,
}

#[derive(Copy, Clone)]
pub enum Conflicts {
    Ko,
    Suicide,
}

#[derive(Copy, Clone)]
pub enum Move {
    Pass,
    Play(usize, usize),
}

#[derive(Copy, Clone)]
pub enum EndGame {
    Equality,
    Score(f32, f32),
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
    pub fn reset(&mut self) {
        self.first = false;
        self.second = false;
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
        let goban = Goban::new(size.into());
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

    pub fn get_komi(&self)-> f32{
        self.komi
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
    /// resume the game when to players have passed, and want to continue.
    ///
    pub fn resume(&mut self) {
        self.passes.reset();
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

    pub fn display(&self) {
        println!("{}", self.goban.pretty_string());
    }

    ///
    /// Method to play on the goban or pass,
    /// Return a conflict (Ko,Suicide) if the move cannot be performed
    ///
    pub fn play(&mut self, play: &Move) -> Option<Conflicts> {
        let mut possible_conflict = None;
        match *play {
            Move::Pass => {
                self.passes.pass();
            }
            Move::Play(x, y) => {
                let stone = Stone { coord: (x, y), color: self.turn.into() };
                possible_conflict =
                    if self.is_ko(&stone) {
                        Some(Conflicts::Ko)
                    } else if self.is_suicide(&stone) {
                        Some(Conflicts::Suicide)
                    } else {
                        self.goban.play(&(x, y), self.turn);
                        self.turn = !self.turn;
                        self.passes.reset();
                        self.remove_atari_stones();
                        None
                    };
            }
        }
        possible_conflict
    }

    ///
    /// Returns the endgame.
    /// None if the game is not finish
    ///
    pub fn end_game(&self) -> Option<EndGame> {
        if !self.gameover() {
            None
        } else {
            let scores = self.calculate_pseudo_score();
            Some(EndGame::Score(scores.0, scores.1))
        }
    }

    ///
    /// Calculates a score for the endgame. It's a naive implementation, it counts only
    /// territories with the same color surrounding them.
    ///
    /// Doesn't handle dead stones.
    ///
    pub fn calculate_pseudo_score(&self) -> (f32, f32) {
        let mut scores: (f32, f32) = (0., 0.); // White & Black
        let empty_groups =
            self.get_strongly_connected_stones(self.goban.get_stones_by_color(&StoneColor::Empty));
        for group in empty_groups {
            let mut neutral = (false, false);
            for empty_intersection in &group {
                for stone in self.goban.get_neighbors(&empty_intersection.coord) {
                    if stone.color == StoneColor::White {
                        neutral.0 = true; // found white stone
                    }
                    if stone.color == StoneColor::Black {
                        neutral.1 = true; // found black stone
                    }
                }
            }
            if neutral.0 && !neutral.1 {
                scores.0 += group.len() as f32;
            } else if !neutral.0 && neutral.1 {
                scores.1 += group.len() as f32;
            }
        }
        scores.0 += self.goban.get_stones_by_color(&StoneColor::White).len() as f32 + self.komi;
        scores.1 += self.goban.get_stones_by_color(&StoneColor::Black).len() as f32;

        (scores.0, scores.1)
    }

    ///
    /// Generate all moves on all intersections.
    ///
    fn pseudo_legals(&self) -> Vec<Move> {
        let mut res = Vec::new();
        for i in 0..self.goban.get_size() {
            for j in 0..self.goban.get_size() {
                if self.goban.get(&(i, j)) == StoneColor::Empty {
                    res.push(Move::Play(i, j));
                }
            }
        }
        res
    }
    ///
    /// Add a stone to the board an then test if the stone or stone groupe is
    /// atari.
    ///
    pub fn is_suicide(&self, stone: &Stone) -> bool {
        let mut goban_tmp = self.goban.clone();
        goban_tmp.play(&stone.coord, self.turn);
        if !goban_tmp.has_liberties(stone) {
            self.are_atari(&self.bfs(&stone))
        } else {
            false
        }
    }

    ///
    /// Returns true if the goban is the same that 2 plays ago, handles passes.
    ///
    pub fn is_ko(&self, stone: &Stone) -> bool {
        if { self.goban.get_history().len() < 2 } {
            false
        } else if self.goban.clone().play(&stone.coord, self.turn) == self.goban.clone().pop_play() {
            true
        } else {
            false
        }
    }

    ///
    /// Test if a groupe of stones is atari.
    ///
    pub fn are_atari(&self, stones: &HashSet<Stone>) -> bool {
        !stones // If there is one stone connected who has liberties it's not atari
            .iter()
            .any(|s| self.goban.has_liberties(s))
    }

    ///
    /// Removes stones in atari from the goban.
    ///
    fn remove_atari_stones(&mut self) {
        let atari_stones: Vec<Stone> = self.goban
            .get_stones().into_iter()
            // get all stones without liberties
            .filter(|point| !self.goban.has_liberties(point))
            .collect();

        let list_of_groups_stones = self.get_strongly_connected_stones
        (atari_stones);

        for groups_of_stones in list_of_groups_stones {
            if self.are_atari(&groups_of_stones) {
                self.goban.set_many(
                    groups_of_stones
                        .iter()
                        .map(|point| &point.coord), StoneColor::Empty)
            }
        }
    }

    ///
    /// Get stones connected. [[x,y,z],[a,e,r]] exemple of return.
    ///
    fn get_strongly_connected_stones(&self, stones: Vec<Stone>) -> Vec<HashSet<Stone>> {
        let mut strongly_connected_stones: Vec<HashSet<Stone>> = Vec::new();
        for atari_stone in stones {
            // if the stone is already in a group of stones
            let is_handled = strongly_connected_stones
                .iter()
                .any(|set| set.contains(&atari_stone));

            if !is_handled {
                strongly_connected_stones.push(self.bfs(&atari_stone))
            }
        }
        strongly_connected_stones
    }

    ///
    /// Can get a group of stones and his neigboors with a bfs,
    /// works for Empty stones too.
    ///
    fn bfs(&self, point: &Stone) -> HashSet<Stone> {
        let mut explored: HashSet<Stone> = HashSet::new();
        explored.insert(point.clone());

        let mut to_explore: Vec<Stone> = self.goban.get_neighbors(&point.coord)
            .into_iter()
            .filter(|p| p.color == point.color)
            .collect(); // Acquiring all the neighbors

        while let Some(point_to_explore) = to_explore.pop() { // exploring the graph
            explored.insert(point_to_explore);
            let neighbors: Vec<Stone> = self.goban.get_neighbors(&point_to_explore.coord)
                .into_iter()
                .filter(|p| p.color == point.color && !explored.contains(p))
                .collect();
            for p in neighbors {
                to_explore.push(p);
            }
        }
        explored
    }
}
