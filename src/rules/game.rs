use crate::pieces::goban::*;
use crate::pieces::stones::Color;
use crate::pieces::stones::Stone;
use crate::pieces::util::coord::Coord;
use crate::rules::EndGame;
use crate::rules::PlayError;
use crate::rules::Player;
use crate::rules::Rule;
use std::collections::HashSet;
use crate::rules::Rule::Japanese;
use sgf_parser::{SgfError, SgfToken};
use crate::rules::game_builder::GameBuilder;
use std::fmt::{Display, Error, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GobanSizes {
    Nineteen,
    Nine,
    Thirteen,
}

impl Into<usize> for GobanSizes {
    fn into(self) -> usize {
        match self {
            GobanSizes::Nine => 9,
            GobanSizes::Nineteen => 19,
            GobanSizes::Thirteen => 13,
        }
    }
}

impl From<usize> for GobanSizes {
    fn from(x: usize) -> Self {
        match x {
            9 => GobanSizes::Nine,
            13 => GobanSizes::Thirteen,
            19 => GobanSizes::Nineteen,
            _ => panic!("Not implemented for others size than 9,13,19")
        }
    }
}

/// Enum for playing in the Goban.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    goban: Goban,

    passes: u8,

    #[get = "pub"]
    prisoners: (u32, u32),

    /// None if none resigned
    /// the player in the option is the player who resigned.
    resigned: Option<Player>,

    #[get = "pub"]
    turn: Player,

    #[get = "pub"]
    #[set = "pub"]
    komi: f32,

    #[get = "pub"]
    #[set = "pub"]
    rule: Rule,

    #[get = "pub"]
    #[set]
    handicap: u8,

    #[get = "pub"]
    plays: Vec<Goban>,

    hashes: HashSet<u64>,
}

impl Game {
    pub fn new(size: GobanSizes, rule: Rule) -> Game {
        let goban = Goban::new(size.into());
        let komi = 5.5;
        let pass = 0;
        let plays = Vec::new();
        let prisoners = (0, 0);
        let handicap = 0;
        let hashes = HashSet::default();
        Game {
            goban,
            turn: Player::Black,
            komi,
            prisoners,
            passes: pass,
            plays,
            resigned: None,
            rule,
            handicap,
            hashes,
        }
    }

    pub fn from_sgf(sgf_str: &str) -> Result<Game, SgfError> {
        let game_tree = sgf_parser::parse(sgf_str)?;
        let mut gamebuilder = GameBuilder::new();
        let mut first = true;
        let mut game: Option<Game> = None;

        for node in game_tree.iter() {
            dbg!(node);
            if first {
                // Game information
                for token in &node.tokens {
                    if token.is_root_token() {
                        match token {
                            SgfToken::Komi(komi) => { gamebuilder.komi(*komi); }
                            SgfToken::Size(x, _y) => { gamebuilder.size(*x as usize); }
                            //TODO another options
                            _ => ()
                        }
                    }
                }
                game = Some(gamebuilder.build().expect("Build the game"));
                first = false;
            } else {
                if let Some(g) = &mut game {
                    if !node.tokens.is_empty() {
                        let token = node.tokens.first().unwrap();
                        if let SgfToken::Move { color: _, coordinate } = token {
                            g.play(Move::Play((coordinate.0 - 1) as usize, (coordinate.1 - 1) as
                                usize));
                            g.display();
                        }
                    }
                } else {
                    panic!("Game not init")
                }
            }
        }
        Ok(Game::new(GobanSizes::Nineteen, Japanese))
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
        } else if let Some(x) = self.resigned {
            if x == Player::White {
                Some(EndGame::WinnerByResign(Player::Black))
            } else {
                Some(EndGame::WinnerByResign(Player::White))
            }
        } else {
            let scores = self.rule.count_points(&self);
            Some(EndGame::Score(scores.0, scores.1))
        }
    }

    ///
    /// Generate all moves on all intersections.
    ///
    fn pseudo_legals(&self) -> impl Iterator<Item=Coord> + '_ {
        self.goban.get_stones_by_color(Color::None).map(|s| s.coordinates)
    }

    ///
    /// Returns a list with legals moves,
    /// In the list will appear suicides moves, and ko moves.
    ///
    pub fn legals(&self) -> impl Iterator<Item=Coord> + '_ {
        self.pseudo_legals()
            .map(move |s| Stone {
                color: self.turn.get_stone_color(),
                coordinates: s,
            })
            .filter(move |s| {
                if let Some(_x) = self.rule.move_validation(self, *s) {
                    false
                } else {
                    true
                }
            })
            .map(|s| (s.coordinates.0, s.coordinates.1))
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
    pub fn play(&mut self, play: Move) {
        match play {
            Move::Pass => {
                self.turn = !self.turn;
                self.passes += 1;
            }
            Move::Play(x, y) => {
                let stone_color: Color = self.turn.get_stone_color();
                self.goban
                    .push((x, y), stone_color)
                    .expect(&format!(
                        "Put the stone in ({},{}) of color {}",
                        x, y, stone_color
                    ));
                self.remove_captured_stones();
                self.plays.push(self.goban.clone()); // Keep the history of the game
                self.hashes.insert(*self.goban.hash());
                self.turn = !self.turn;
                self.passes = 0;
            }
            Move::Resign => {
                self.resigned = Some(self.turn);
            }
        }
    }

    ///
    /// Method to play but it verifies if the play is legal or not.
    ///
    pub fn play_with_verifications(&mut self, play: Move) -> Result<(), PlayError> {
        if self.passes == 2 {
            Err(PlayError::GamePaused)
        } else {
            match play {
                Move::Pass => {
                    self.passes += 1;
                    Ok(())
                }
                Move::Play(x, y) => {
                    let stone = Stone {
                        coordinates: (x, y),
                        color: self.turn.get_stone_color(),
                    };
                    if let Some(c) = self.rule.move_validation(self, stone) {
                        Err(c)
                    } else {
                        self.play(play);
                        Ok(())
                    }
                }
                Move::Resign => {
                    self.resigned = Some(self.turn);
                    Ok(())
                }
            }
        }
    }

    ///
    /// Removes the last move.
    ///
    pub fn pop(&mut self) -> &mut Self {
        if let Some(goban) = self.plays.pop() {
            self.hashes.remove(self.goban.hash());
            self.turn = !self.turn;
            self.goban = goban;
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
        let empty_groups = self
            .goban
            .get_strings_from_stones(self.goban.get_stones_by_color(Color::None));
        for group in empty_groups {
            let mut neutral = (false, false);
            for empty_intersection in &group {
                for stone in self.goban.get_neighbors(empty_intersection.coordinates) {
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
        self.goban.get_stones().for_each(|stone| match stone.color {
            Color::Black => {
                res.0 += 1;
            }
            Color::White => {
                res.1 += 1;
            }
            _ => unreachable!(),
        });
        res
    }

    pub fn will_capture(&self, point: Coord) -> bool {
        for stone in self.goban.get_neighbors(point)
            .filter(|s|s.color != Color::None && s.color != self.turn.get_stone_color()) {
            if self.goban.count_string_liberties(&self.goban.get_string_from_stone(stone)) ==
                1 {
                return true;
            }
        }
        false
    }

    ///
    /// Put the handicap stones on the goban.
    /// Does not override previous setting ! .
    ///
    pub fn put_handicap(&mut self, coords: &[Coord]) {
        self.handicap = coords.len() as u8;
        coords.iter().for_each(|coord| {
            self.goban.push(*coord, Color::Black).expect(&format!(
                "Putting the handicap stone ({},{})",
                coord.0, coord.1
            ));
        });
        self.turn = Player::White;
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
    pub fn is_suicide(&self, stone: Stone) -> bool {
        let mut goban_test: Goban = self.goban().clone();
        goban_test.push_stone(stone).expect("Play the stone");

        if goban_test.has_liberties(stone) {
            false
        } else {
            // Test if the connected stones are also without liberties.
            if goban_test.is_string_dead(&goban_test.get_string_from_stone(stone)) {
                // if the chain has no liberties then look if enemy stones are captured
                !goban_test
                    .get_neighbors(stone.coordinates)
                    .filter(|s| s.color == (!self.turn).get_stone_color())
                    .map(|s| goban_test.get_string_from_stone(s))
                    .any(|chain_of_stones| goban_test.is_string_dead(&chain_of_stones))
            } else {
                false
            }
        }
    }

    ///
    /// Test if a play is ko.
    /// If the goban is in the configuration of the two plays ago returns true
    ///
    pub fn ko(&self, stone: Stone) -> bool {
        if self.plays.len() <= 2 {
            false
        } else if !self.will_capture(stone.coordinates) {
            false
        } else {
            let mut game = self.clone();
            game.play(Move::Play(stone.coordinates.0, stone.coordinates.1));
            game.goban == self.plays[self.plays.len() - 2]
        }
    }

    ///
    /// Rule of the super Ko, if any before configuration was already played then return true.
    ///
    pub fn super_ko(&self, stone: Stone) -> bool {
        let mut game = self.clone();
        game.play(Move::Play(stone.coordinates.0, stone.coordinates.1));

        self.hashes.contains(game.goban.hash())
    }

    ///
    /// Removes captured stones from the goban.
    ///
    fn remove_captured_stones(&mut self) {
        if self.turn == Player::Black {
            self.prisoners.0 += self.remove_captured_stones_color(Color::White) as u32;
        } else {
            self.prisoners.1 += self.remove_captured_stones_color(Color::Black) as u32;
        }
    }

    ///
    /// Removes the dead stones from the goban by specifying a color stone.
    /// Returns the number of stones removed from the goban.
    ///
    fn remove_captured_stones_color(&mut self, color: Color) -> usize {
        let mut number_of_stones_captured = 0;
        for groups_of_stones in self
            .goban
            .get_strings_of_stones_without_liberties_wth_color(color)
            {
                if self.goban.is_string_dead(&groups_of_stones) {
                    self.goban.push_many(
                        groups_of_stones.iter().map(|point| point.coordinates),
                        Color::None,
                    );
                    number_of_stones_captured += groups_of_stones.len();
                }
            }
        number_of_stones_captured
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new(GobanSizes::Nineteen, Rule::Japanese)
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        self.goban.fmt(f)
    }
}
