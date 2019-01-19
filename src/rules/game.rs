use crate::pieces::goban::*;
use crate::pieces::stones::StoneColor;
use std::collections::HashSet;
use crate::pieces::stones::Stone;
use crate::rules::Rule;
use crate::rules::JapRule;

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

#[derive(Copy, Clone)]
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

#[derive(Clone)]
pub struct Game<T: Rule> {
    goban: Goban,
    passes: Passes,
    prisoners: (u32, u32),
    turn: bool,
    komi: f32,
    plays: Vec<Goban>,
    rule: T,
}

impl<T: Rule> Game<T> {
    pub fn new(size: GobanSizes) -> Game<T> {
        let goban = Goban::new(size.into());
        let komi = 5.5;
        let pass = Passes::new();
        let plays = Vec::new();
        let prisoners = (0, 0);
        Game<JapRule> { goban, turn: false, komi, prisoners, passes: pass, rule: JapRule {}, plays }
    }

    pub const fn goban(&self) -> &Goban {
        &self.goban
    }

    pub const fn turn(&self) -> bool {
        self.turn
    }

    pub const fn prisoners(&self) -> &(u32, u32) { &self.prisoners }

    pub fn komi(&self) -> f32 { self.komi }

    pub fn set_komi(&mut self, komi: f32) {
        self.komi = komi;
    }

    pub fn set_rules(&mut self, rule: impl Rule) {
        self.rule = rule;
    }

    pub fn rules(&self) -> &impl Rule {
        &self.rule
    }

    pub fn plays(&self) -> &Vec<Goban> { &self.plays }
}

impl<T:Rule> Game<T>{
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
    /// Removes the last move.
    ///
    pub fn pop(&mut self) {
        self.goban.pop();
        self.plays.pop();
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
    /// Prints the goban.
    ///
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
                        self.plays.push(self.goban.clone());
                        self.goban.push(&(x, y), stone.color)
                            .expect(&format!("Put the stone in ({},{}) of color {}", x, y, stone
                                .color));
                        self.turn = !self.turn;
                        self.passes.reset();
                        self.remove_dead_stones();
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
            let scores = self.calculate_territories();
            Some(EndGame::Score(scores.0, scores.1))
        }
    }

    ///
    /// Calculates a score for the endgame. It's a naive implementation, it counts only
    /// territories with the same color surrounding them.
    ///
    /// Returns (black territory,  white territory)
    ///
    pub fn calculate_territories(&self) -> (f32, f32) {
        let mut scores: (f32, f32) = (0., 0.); // Black & White
        let empty_groups =
            self.goban.get_strongly_connected_stones(self.goban.get_stones_by_color
            (&StoneColor::Empty));
        for group in empty_groups {
            let mut neutral = (false, false);
            for empty_intersection in &group {
                for stone in self.goban.get_neighbors(&empty_intersection.coord) {
                    if stone.color == StoneColor::White {
                        neutral.1 = true; // found white stone
                    }
                    if stone.color == StoneColor::Black {
                        neutral.0 = true; // found black stone
                    }
                }
            }
            if neutral.0 && !neutral.1 {
                scores.0 += group.len() as f32;
            } else if !neutral.0 && neutral.1 {
                scores.1 += group.len() as f32;
            }
        }
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
    /// Add a stone to the board an then test if the stone or stone group is
    /// atari.
    /// Returns true if the move is a suicide
    ///
    pub fn is_suicide(&self, stone: &Stone) -> bool {
        let mut goban_test = self.goban().clone();
        goban_test.push_stone(stone).expect("Play the stone");

        // If there is no atari
        if goban_test.has_liberties(stone) {
            false
        } else {
            // Search if the opponent has atari stones because of the play
            if Goban::get_atari_stones_color(&goban_test, (!self.turn).into()).len() == 0 {
                true
            } else {
                // Search for connections
                self.are_dead(&self.goban.bfs(&stone))
            }
        }
    }

    ///
    /// If the goban is in the configuration of the two plays ago returns true
    ///
    pub fn is_ko(&self, stone: &Stone) -> bool {
        if self.plays.len() <= 2 {
            false
        } else {
            if self.goban.clone().push_stone(stone) == self.plays[self.plays.len() - 2] {
                true
            } else {
                false
            }
        }
    }

    ///
    /// Rule of the super Ko, if any before configuration was already played then the move is
    /// illegal
    ///
    pub fn super_ko(&self, stone: &Stone) -> bool {
        let mut goban_test = self.goban.clone();
        goban_test.push_stone(stone).expect("Put the stone");

        self.plays.iter().rev().any(|g| *g == goban_test)
    }

    ///
    /// Test if a group of stones is dead.
    ///
    /// "a group of stones is dead if it doesn't have liberties"
    ///
    pub fn are_dead(&self, stones: &HashSet<Stone>) -> bool {
        !stones // If there is one stone connected who has liberties it's not atari
            .iter()
            .any(|s| self.goban.has_liberties(s))
    }

    ///
    /// Removes dead stones from the goban.
    ///
    fn remove_dead_stones(&mut self) {
        for groups_of_stones in self.goban.get_atari_stones() {
            if self.are_dead(&groups_of_stones) {
                self.goban.push_many(
                    groups_of_stones
                        .iter()
                        .map(|point| &point.coord), StoneColor::Empty)
            }
        }
    }

    ///
    /// Removes the dead stones from the goban by specifying a color stone.
    ///
    fn remove_dead_stones_color(&mut self, color: StoneColor) {
        for groups_of_stones in self.goban.get_atari_stones_color(color) {
            if self.are_dead(&groups_of_stones) {
                self.goban.push_many(
                    groups_of_stones
                        .iter()
                        .map(|point| &point.coord), StoneColor::Empty)
            }
        }
    }
}
