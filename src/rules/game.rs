use crate::pieces::goban::*;
use crate::pieces::stones::Color;
use std::collections::HashSet;
use crate::pieces::stones::Stone;
use crate::rules::Rule;
use crate::rules::turn::BLACK;
use crate::rules::PlayError;
use crate::pieces::util::Coord;

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
pub enum Move {
    Pass,
    Resign,
    Play(usize, usize),
}

impl From<Coord> for Move {
    fn from(x: (usize, usize)) -> Self {
        Move::Play(x.0, x.1)
    }
}

#[derive(Copy, Clone)]
pub enum EndGame {
    Score(f32, f32),
    WinnerByResign(bool),
}

#[derive(Clone, Getters, Setters)]
pub struct Game {
    #[get = "pub"]
    #[set = "pub"]
    goban: Goban,
    passes: u8,

    #[get = "pub"]
    #[set = "pub"]
    prisoners: (u32, u32),

    /// None if none resigned
    /// true if the white resigned
    /// false if the black resigned
    resigned: Option<bool>,

    #[get = "pub"]
    #[set = "pub"]
    turn: bool,

    #[get = "pub"]
    #[set = "pub"]
    komi: f32,

    #[get = "pub"]
    #[set = "pub"]
    plays: Vec<Goban>,
}

impl Game {
    pub fn new(size: GobanSizes) -> Game {
        let goban = Goban::new(size.into());
        let komi = 5.5;
        let pass = 0;
        let plays = Vec::new();
        let prisoners = (0, 0);
        Game { goban, turn: BLACK, komi, prisoners, passes: pass, plays, resigned: None }
    }
}

impl Game {
    ///
    /// Resume the game when to players have passed, and want to continue.
    ///
    pub fn resume(&mut self) {
        self.passes = 0;
    }

    ///
    /// True when the game is over (two passes, or no more legals moves, Resign)
    ///
    pub fn over<T: Rule>(&self) -> bool {
        if let Some(_x) = self.resigned {
            true
        } else {
            self.passes == 2 || self.legals::<T>().count() == 0
        }
    }

    ///
    /// Returns the endgame.
    /// None if the game is not finished
    ///
    pub fn end_game<T: Rule>(&self) -> Option<EndGame> {
        if !self.over::<T>() {
            None
        } else {
            if let Some(x) = self.resigned {
                Some(EndGame::WinnerByResign(!x))
            } else {
                let scores = T::count_points(&self);
                Some(EndGame::Score(scores.0, scores.1))
            }
        }
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
    ///
    pub fn legals<T: Rule>(&self) -> impl Iterator<Item=Coord> + '_ {
        self.pseudo_legals()
            .map(move |s| Stone {
                color: self.turn.into(),
                coord: s,
            })
            .filter(move |s| {
                if let Some(_x) = T::move_validation(self, s) {
                    false
                } else {
                    true
                }
            })
            .map(|s| (s.coord.0, s.coord.1))
    }

    ///
    /// Prints the goban.
    ///
    pub fn display(&self) {
        println!("{}", self.goban.pretty_string());
    }

    ///
    /// Method to play on the goban or pass.
    ///
    pub fn play(&mut self, play: &Move) {
        match *play {
            Move::Pass => {
                self.passes += 1;
            }
            Move::Play(x, y) => {
                self.plays.push(self.goban.clone());
                self.goban.push(&(x, y), self.turn.into())
                    .expect(&format!("Put the stone in ({},{}) of color {}", x, y, self.turn));
                self.turn = !self.turn;
                self.passes = 0;
                self.remove_dead_stones();
            }
            Move::Resign => {
                self.resigned = self.turn.into();
            }
        }
    }

    ///
    /// Method to play but it verifies if the play is legal or not.
    ///
    pub fn play_with_verifications<R: Rule>(&mut self, play: &Move) -> Result<(), PlayError> {
        if self.passes != 2 {
            Err(PlayError::GamePaused)
        } else {
            match *play {
                Move::Pass => {
                    self.passes += 1;
                    Ok(())
                }
                Move::Play(x, y) => {
                    let stone = Stone { coord: (x, y), color: self.turn.into() };
                    if let Some(c) = R::move_validation(self, &stone) {
                        Err(c)
                    } else {
                        self.play(play);
                        Ok(())
                    }
                }
                Move::Resign => {
                    Ok(self.resigned = self.turn.into())
                }
            }
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
            (Color::None));
        for group in empty_groups {
            let mut neutral = (false, false);
            for empty_intersection in &group {
                for stone in self.goban.get_neighbors(&empty_intersection.coord) {
                    if stone.color == Color::White {
                        neutral.1 = true; // found white stone
                    }
                    if stone.color == Color::Black {
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
    fn pseudo_legals(&self) -> impl Iterator<Item=Coord> + '_ {
        self.goban.get_stones_by_color(Color::None)
            .map(|s| s.coord)
    }

    ///
    /// Add a stone to the board an then test if the stone or stone group is
    /// dead.
    /// Returns true if the move is a suicide
    ///
    pub fn is_suicide(&self, stone: &Stone) -> bool {
        let mut goban_test = self.goban().clone();
        goban_test.push_stone(stone).expect("Play the stone");

        if goban_test.has_liberties(stone) {
            false
        } else {
            // Search if the opponent has dead stones because of the play
            if Goban::get_dead_stones_color(&goban_test, (!self.turn).into()).len() == 0 {
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
            if *self.goban.clone().push_stone(stone).expect("Put the stone")
                == self.plays[self.plays.len() - 2] {
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
        for groups_of_stones in self.goban.get_dead_stones() {
            if self.are_dead(&groups_of_stones) {
                self.goban.push_many(
                    groups_of_stones
                        .iter()
                        .map(|point| &point.coord), Color::None)
            }
        }
    }

    ///
    /// Removes the dead stones from the goban by specifying a color stone.
    ///
    fn remove_dead_stones_color(&mut self, color: Color) {
        for groups_of_stones in self.goban.get_dead_stones_color(color) {
            if self.are_dead(&groups_of_stones) {
                self.goban.push_many(
                    groups_of_stones
                        .iter()
                        .map(|point| &point.coord), Color::None)
            }
        }
    }
}
