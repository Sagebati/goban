use crate::pieces::goban::*;
use crate::pieces::stones::Color;
use crate::pieces::stones::Stone;
use crate::pieces::util::coord::Coord;
use crate::rules::game_builder::GameBuilder;
use crate::rules::EndGame;
use crate::rules::PlayError;
use crate::rules::Player;
use crate::rules::Rule;
use sgf_parser::{SgfError, SgfToken, Action};
use std::collections::HashSet;

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
            GobanSizes::Thirteen => 13,
            GobanSizes::Nineteen => 19,
        }
    }
}

impl From<usize> for GobanSizes {
    fn from(x: usize) -> Self {
        match x {
            9 => GobanSizes::Nine,
            13 => GobanSizes::Thirteen,
            19 => GobanSizes::Nineteen,
            _ => panic!("Not implemented for others size than 9,13,19"),
        }
    }
}

/// Enum for playing in the Goban.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Move {
    Pass,
    Resign(Player),
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
    pub fn new(size: GobanSizes, rule: Rule) -> Self {
        let goban = Goban::new(size.into());
        let komi = 5.5;
        let pass = 0;
        let plays = Vec::new();
        let prisoners = (0, 0);
        let handicap = 0;
        let hashes = HashSet::with_capacity(300);
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

    pub fn from_sgf(sgf_str: &str) -> Result<Self, SgfError> {
        let game_tree = sgf_parser::parse(sgf_str)?;
        let mut gamebuilder = GameBuilder::new();
        let mut first = true;
        let mut game: Option<Game> = None;

        for node in game_tree.iter() {
            if first {
                // Game information
                for token in &node.tokens {
                    if token.is_root_token() {
                        match token {
                            SgfToken::Komi(komi) => {
                                gamebuilder.komi(*komi);
                            }
                            SgfToken::Size(x, y) => {
                                gamebuilder.size((*x, *y));
                            }
                            //TODO another options
                            _ => (),
                        }
                    }
                }
                game = Some(gamebuilder.build().expect("Build the game"));
                first = false;
            } else if let Some(g) = &mut game {
                if !node.tokens.is_empty() {
                    let token = node.tokens.first().unwrap();
                    if let SgfToken::Move {
                        action, ..
                    } = token
                    {
                        g.play_with_verifications(match *action {
                            Action::Move(col, line) => {
                                Move::Play((line - 1) as usize, (col - 1) as usize)
                            }
                            Action::Pass => Move::Pass,
                        }).expect(&format!("Play the move read from the sgf"));
                    }
                }
            } else {
                panic!("Game not constructed")
            }
        }
        Ok(game.expect("The game to be initialised from the sgf"))
    }
}

impl Game {
    ///
    /// Resume the game when to players have passed, and want to continue.
    ///
    #[inline]
    pub fn resume(&mut self) {
        self.passes = 0;
    }

    ///
    /// True when the game is over (two passes, or no more legals moves, Resign)
    ///
    #[inline]
    pub fn is_over(&self) -> bool {
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
    #[inline]
    pub fn outcome(&self) -> Option<EndGame> {
        if !self.is_over() {
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
    #[inline]
    fn pseudo_legals(&self) -> impl Iterator<Item=Coord> + '_ {
        self.goban
            .get_stones_by_color(Color::None)
            .map(|s| s.coordinates)
    }

    ///
    /// Returns a list with legals moves,
    /// In the list will appear suicides moves, and ko moves.
    ///
    #[inline]
    pub fn legals(&self) -> impl Iterator<Item=Coord> + '_ {
        self.pseudo_legals()
            .map(move |s| Stone {
                color: self.turn.get_stone_color(),
                coordinates: s,
            })
            .filter(move |s| {
                if let Some(_) = self.rule.move_validation(self, *s) {
                    false
                } else {
                    true
                }
            })
            .map(|s| (s.coordinates.0, s.coordinates.1))
    }

    ///
    /// Method to play on the goban or pass.
    /// (0,0) is in the top left corner of the goban.
    ///
    pub fn play(&mut self, play: Move) -> &mut Self {
        match play {
            Move::Pass => {
                self.turn = !self.turn;
                self.passes += 1;
                self
            }
            Move::Play(x, y) => {
                let stone_color: Color = self.turn.get_stone_color();
                self.goban.push((x, y), stone_color).expect(&format!(
                    "Put the stone in ({},{}) of color {}",
                    x, y, stone_color
                ));
                self.remove_captured_stones();
                self.plays.push(self.goban.clone());
                self.hashes.insert(*self.goban.hash());
                self.turn = !self.turn;
                self.passes = 0;
                self
            }
            Move::Resign(player) => {
                self.resigned = Some(player);
                self
            }
        }
    }

    ///
    /// Method to play but it verifies if the play is legal or not.
    ///
    pub fn play_with_verifications(&mut self, play: Move) -> Result<&mut Game, PlayError> {
        if self.passes == 2 {
            Err(PlayError::GamePaused)
        } else {
            match play {
                Move::Pass | Move::Resign(_) => {
                    self.play(play);
                    Ok(self)
                }
                Move::Play(x, y) => {
                    if let Some(c) = self.rule.move_validation(
                        self,
                        Stone {
                            coordinates: (x, y),
                            color: self.turn.get_stone_color(),
                        })
                    {
                        Err(c)
                    } else {
                        self.play(play);
                        Ok(self)
                    }
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

    pub fn will_capture(&self, point: Coord) -> bool {
        for stone in self
            .goban
            .get_neighbors(point)
            .filter(|s| s.color != Color::None && s.color != self.turn.get_stone_color())
            {
                if self
                    .goban
                    .count_string_liberties(&self.goban.get_string_from_stone(stone))
                    == 1
                {
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
        goban_test
            .push_stone(stone)
            .expect("Play the stone for verification if it's suicide");

        if goban_test.has_liberties(stone) {
            false
        } else {
            // Test if the connected stones are also without liberties.
            if goban_test.is_string_dead(&goban_test.get_string_from_stone(stone)) {
                // if the chain has no liberties then look if enemy stones are captured
                !goban_test
                    .get_neighbors(stone.coordinates)
                    .filter(|neigbor_stone| neigbor_stone.color == (!self.turn).get_stone_color())
                    .map(|s| goban_test.get_string_from_stone(s))
                    .any(|string_of_stones| goban_test.is_string_dead(&string_of_stones))
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
        if self.plays.len() <= 2 || !self.will_capture(stone.coordinates) {
            false
        } else {
            let mut game = self.clone();
            game.play(stone.coordinates.into());
            game.goban == self.plays[self.plays.len() - 2]
        }
    }

    ///
    /// Rule of the super Ko, if any before configuration was already played then return true.
    ///
    pub fn super_ko(&self, stone: Stone) -> bool {
        self.hashes.contains(self.clone().play(stone.coordinates.into()).goban.hash())
    }

    ///
    /// Displays the internal board.
    ///
    pub fn display_goban(&self) {
        println!("{}", self.goban)
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
