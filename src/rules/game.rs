use crate::pieces::goban::*;
use crate::pieces::stones::Color;
use crate::pieces::stones::Stone;
use crate::rules::Rule;
use crate::rules::PlayError;
use crate::rules::turn::BLACK;
use crate::rules::turn::WHITE;
use crate::rules::Player;
use crate::rules::EndGame;
use crate::pieces::util::coord::Coord;

#[derive(Debug, Clone, PartialEq, Eq)]
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


/// Enum for playing in the Goban.
#[derive(Clone, Debug, PartialEq, Eq)]
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


#[derive(Clone, Getters, Setters, Debug)]
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

    /// Bool true when is white turn
    /// false when is black turn
    turn: bool,

    #[get = "pub"]
    #[set = "pub"]
    komi: f32,

    #[get = "pub"]
    #[set]
    rule: Rule,

    #[get = "pub"]
    #[set]
    handicap: u8,

    #[get = "pub"]
    #[set = "pub"]
    plays: Vec<Goban>,
}


impl Game {
    pub fn new(size: GobanSizes, rule: Rule) -> Game {
        let goban = Goban::new(size.into());
        let komi = 5.5;
        let pass = 0;
        let plays = Vec::new();
        let prisoners = (0, 0);
        let handicap = 0;
        Game {
            goban,
            turn: BLACK,
            komi,
            prisoners,
            passes: pass,
            plays,
            resigned: None,
            rule,
            handicap,
        }
    }

    pub fn turn(&self) -> Player {
        if self.turn {
            Player::White
        } else {
            Player::Black
        }
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
    pub fn over(&self) -> bool {
        if let Some(_x) = self.resigned {
            true
        } else {
            self.passes == 2 || self.legals().count() == 0
        }
    }


    ///
    /// Returns the endgame.
    /// None if the game is not finished
    ///
    pub fn outcome(&self) -> Option<EndGame> {
        if !self.over() {
            None
        } else {
            if let Some(x) = self.resigned {
                if x == WHITE {
                    Some(EndGame::WinnerByResign(Player::Black))
                } else {
                    Some(EndGame::WinnerByResign(Player::White))
                }
            } else {
                let scores = self.rule.count_points(&self);
                Some(EndGame::Score(scores.0, scores.1))
            }
        }
    }


    ///
    /// Generate all moves on all intersections.
    ///
    fn pseudo_legals(&self) -> impl Iterator<Item=Coord> + '_ {
        self.goban.get_stones_by_color(Color::None)
            .map(|s| s.coord)
    }

    ///
    /// Returns a list with legals moves,
    /// In the list will appear suicides moves, and ko moves.
    ///
    pub fn legals(&self) -> impl Iterator<Item=Coord> + '_ {
        self.pseudo_legals()
            .map(move |s| Stone {
                color: self.turn.into(),
                coord: s,
            })
            .filter(move |s| {
                if let Some(_x) = self.rule.move_validation(self, s) {
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
    /// (0,0) is in the top left corner of the goban.
    ///
    pub fn play(&mut self, play: &Move) {
        match play {
            Move::Pass => {
                self.turn = !self.turn;
                self.passes += 1;
            }
            Move::Play(x, y) => {
                self.plays.push(self.goban.clone());
                self.goban.push(&(*x, *y), self.turn.into())
                    .expect(&format!("Put the stone in ({},{}) of color {}", x, y, self.turn));
                self.turn = !self.turn;
                self.passes = 0;
                self.remove_captured_stones();
            }
            Move::Resign => {
                self.resigned = self.turn.into();
            }
        }
    }

    ///
    /// Method to play but it verifies if the play is legal or not.
    ///
    pub fn play_with_verifications(&mut self, play: &Move) -> Result<(), PlayError> {
        if self.passes == 2 {
            Err(PlayError::GamePaused)
        } else {
            match play {
                Move::Pass => {
                    self.passes += 1;
                    Ok(())
                }
                Move::Play(x, y) => {
                    let stone = Stone { coord: (*x, *y), color: self.turn.into() };
                    if let Some(c) = self.rule.move_validation(self, &stone) {
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
    /// Removes the last move.
    ///
    pub fn pop(&mut self) -> &Self {
        let x = self.plays.pop();
        match x {
            Some(goban) => {
                self.goban = goban;
            }
            _ => {}
        }
        self
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
    /// Get number of stones on the goban.
    /// (number of black stones, number of white stones)
    ///
    pub fn number_of_stones(&self) -> (u32, u32) {
        let mut res: (u32, u32) = (0, 0);
        self.goban.get_stones().for_each(|stone| {
            match stone.color {
                Color::Black => { res.0 += 1; }
                Color::White => { res.1 += 1; }
                _ => unreachable!()
            }
        });
        res
    }

    ///
    /// Put the handicap stones on the goban.
    /// Does not override previous setting ! .
    ///
    pub fn put_handicap(&mut self, coords: &[Coord]) {
        self.handicap = coords.len() as u8;
        coords.iter().for_each(|coord| {
            self.goban.push(coord, Color::Black)
                .expect(&format!("Putting the handicap stone ({},{})", coord.0, coord.1));
        });
        self.turn = !self.turn;
    }

    ///
    /// Calculates score. with prisoners and komi.
    /// Dependant of the rule.
    ///
    pub fn calculate_score(&self) -> (f32, f32) {
        self.rule.count_points(self)
    }


    ///
    /// Add a stone to the board an then test if the stone or stone group is
    /// dead.
    /// Returns true if the move is a suicide
    ///
    pub fn is_suicide(&self, stone: &Stone) -> bool {
        let mut goban_test: Goban = self.goban().clone();
        goban_test.push_stone(stone).expect("Play the stone");

        if goban_test.has_liberties(stone) {
            false
        } else {
            let opponent_color: Color = (!self.turn).into();
            // Search if the opponent has captured stones because of the play
            if goban_test.get_neighbors(&stone.coord)
                .filter(|s| s.color == opponent_color)
                .map(|s| goban_test.bfs(&s))
                .any(|bfs| goban_test.are_dead(&bfs))
            {
                true
            } else {
                // Search for connections
                goban_test.are_dead(&goban_test.bfs(&stone))
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
    /// Removes dead stones from the goban.
    ///
    fn remove_captured_stones(&mut self) {
        for groups_of_stones in self.goban.get_captured_stones() {
            if self.goban.are_dead(&groups_of_stones) {
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
    #[allow(dead_code)]
    fn remove_captured_stones_color(&mut self, color: Color) {
        for groups_of_stones in self.goban.get_dead_stones_color(color) {
            if self.goban.are_dead(&groups_of_stones) {
                self.goban.push_many(
                    groups_of_stones
                        .iter()
                        .map(|point| &point.coord), Color::None)
            }
        }
    }
}



