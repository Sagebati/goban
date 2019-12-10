use crate::pieces::goban::*;
use crate::pieces::stones::Color;
use crate::pieces::stones::Stone;
use crate::pieces::util::coord::Point;
use crate::rules::EndGame::{Draw, WinnerByScore};
use crate::rules::PlayError;
use crate::rules::Player;
use crate::rules::Player::{Black, White};
use crate::rules::Rule;
use crate::rules::Rule::Chinese;
use crate::rules::{EndGame, GobanSizes, Move};
use std::collections::HashSet;
use sloth::Lazy;

#[derive(Clone, Getters, CopyGetters, Setters, Debug)]
pub struct Game {
    #[get = "pub"]
    goban: Goban,

    passes: u8,

    #[get_copy = "pub"]
    prisoners: (u32, u32),

    /// None if hte game is not finished
    /// the player in the option is the player who resigned.
    outcome: Option<EndGame>,

    #[get_copy = "pub"]
    turn: Player,

    #[get_copy = "pub"]
    #[set = "pub"]
    komi: f32,

    #[get_copy = "pub"]
    #[set = "pub"]
    rule: Rule,

    #[get_copy = "pub"]
    #[set]
    handicap: u8,

    #[get = "pub"]
    plays: Vec<u64>,

    hashes: HashSet<u64>,
}

impl Game {
    pub fn new(size: GobanSizes, rule: Rule) -> Self {
        let goban = Goban::new(size.into());
        let komi = 5.5;
        let pass = 0;
        let plays = Vec::with_capacity(300);
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
            outcome: None,
            rule,
            handicap,
            hashes,
        }
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
        if self.outcome.is_some() {
            true
        } else {
            self.passes == 2
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
        } else if self.outcome.is_some() {
            self.outcome
        } else {
            let scores = self.rule.count_points(&self);
            if (scores.0 - scores.1).abs() < std::f32::EPSILON {
                Some(Draw)
            } else if scores.0 > scores.1 {
                Some(WinnerByScore(Black, scores.0 - scores.1))
            } else {
                Some(WinnerByScore(White, scores.1 - scores.0))
            }
        }
    }

    ///
    /// Generate all moves on all intersections.
    ///
    #[inline]
    fn pseudo_legals(&self) -> impl Iterator<Item=Point> + '_ {
        self.goban.get_points_by_color(Color::None)
    }

    ///
    /// Returns a list with legals moves,
    /// In the list will appear suicides moves, and ko moves.
    ///
    #[inline]
    pub fn legals(&self) -> impl Iterator<Item=Point> + '_ {
        let mut test_game = Lazy::new(move || self.clone());
        self.pseudo_legals()
            .map(move |s| Stone {
                color: self.turn.get_stone_color(),
                coordinates: s,
            })
            .filter(move |&s| self.rule.move_validation(&mut test_game, s).is_none())
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
                let hash = self.goban.hash();
                self.plays.push(hash);
                self.hashes.insert(hash);
                self.goban.push((x, y), self.turn.get_stone_color());
                self.prisoners = self.remove_captured_stones();
                self.turn = !self.turn;
                self.passes = 0;
                self
            }
            Move::Resign(player) => {
                self.outcome = Some(EndGame::WinnerByResign(player));
                self
            }
        }
    }

    fn play_for_verification(&mut self, (x, y): Point) -> u64 {
        let actual_goban = self.goban.clone();
        self.goban.push((x, y), self.turn.get_stone_color());
        self.remove_captured_stones();
        let new_goban_hash = self.goban.hash();
        self.goban = actual_goban;
        new_goban_hash
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
                        },
                    ) {
                        Err(c)
                    } else {
                        Ok(self.play(play))
                    }
                }
            }
        }
    }

    ///
    /// Put the handicap stones on the goban.
    /// Does not override previous setting ! .
    ///
    pub fn put_handicap(&mut self, points: &[Point]) {
        self.handicap = points.len() as u8;
        points.iter().for_each(|&coord| {
            self.goban.push(coord, Color::Black);
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
        if self.goban.has_liberties(stone) {
            false
        } else {
            let mut friendly_strings = vec![];
            for neighbor_go_string in self.goban.get_neighbors_strings(stone.coordinates) {
                if neighbor_go_string.borrow().color == stone.color {
                    friendly_strings.push(neighbor_go_string)
                } else {
                    // capture move so not suicide
                    if neighbor_go_string.borrow().number_of_liberties() == 1 {
                        return false;
                    }
                }
            }
            // If all of the same color go strings have only one liberty then
            // it's self capture
            friendly_strings
                .into_iter()
                .all(|go_str_ptr| go_str_ptr.borrow().number_of_liberties() == 1)
        }
    }

    ///
    /// Returns true if the stone played in that point will capture another
    /// string.
    ///
    pub fn will_capture(&self, point: Point) -> bool {
        self.goban
            .get_neighbors_strings(point)
            .filter(|go_str_ptr| go_str_ptr.borrow().color != self.turn.get_stone_color())
            // if an enemy string has only liberty it's a capture move
            .any(|go_str_ptr| go_str_ptr.borrow().number_of_liberties() == 1)
    }

    ///
    /// Test if a play is ko.
    /// If the goban is in the configuration of the two plays ago returns true
    ///
    pub fn ko(&mut self, stone: Stone) -> bool {
        if self.plays.len() <= 2 || !self.will_capture(stone.coordinates) {
            false
        } else {
            self.play_for_verification(stone.coordinates) == self.plays[self.plays.len() - 1]
        }
    }

    ///
    /// Rule of the super Ko, if any before configuration was already played then return true.
    ///
    pub fn super_ko(&mut self, stone: Stone) -> bool {
        if !self.will_capture(stone.coordinates) {
            false
        } else {
            let hash_test_goban = self.play_for_verification(stone.coordinates);
            self.hashes
                .contains(&hash_test_goban)
        }
    }

    ///
    /// Displays the internal board.
    ///
    pub fn display_goban(&self) {
        println!("{}", self.goban)
    }

    ///
    /// Remove captured stones, and add it to the count of prisoners
    /// returns new captured stones.
    ///
    #[inline]
    fn remove_captured_stones(&mut self) -> (u32, u32) {
        let mut new_prisoners = self.prisoners;
        match self.turn {
            Black => {
                new_prisoners.0 += self.remove_captured_stones_turn(White);
                if self.rule.is_suicide_valid() {
                    new_prisoners.1 += self.remove_captured_stones_turn(Black);
                }
                new_prisoners
            }
            White => {
                new_prisoners.1 += self.remove_captured_stones_turn(Black);
                if self.rule.is_suicide_valid() {
                    new_prisoners.0 += self.remove_captured_stones_turn(White);
                }
                new_prisoners
            }
        }
    }

    ///
    /// Removes the dead stones from the goban by specifying a color stone.
    /// Returns the number of stones removed from the goban.
    ///
    fn remove_captured_stones_turn(&mut self, player: Player) -> u32 {
        let mut number_of_stones_captured = 0u32;
        let string_without_liberties = self
            .goban
            .get_strings_of_stones_without_liberties_wth_color(player.get_stone_color())
            .collect::<HashSet<_>>();
        for group_of_stones in string_without_liberties
            {
                number_of_stones_captured += group_of_stones.borrow().stones().len() as u32;
                self.goban.remove_string(group_of_stones);
            }
        number_of_stones_captured
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new(GobanSizes::Nineteen, Rule::Japanese)
    }
}

pub struct GameBuilder {
    size: (u32, u32),
    komi: f32,
    black_player: String,
    white_player: String,
    rule: Rule,
    handicap_points: Vec<Point>,
    turn: Player,
    moves: Vec<Move>,
    outcome: Option<EndGame>,
}

impl GameBuilder {
    fn new() -> GameBuilder {
        GameBuilder {
            size: (19, 19),
            komi: 0.,
            black_player: "".to_string(),
            white_player: "".to_string(),
            handicap_points: vec![],
            rule: Chinese,
            turn: Player::Black,
            moves: vec![],
            outcome: None,
        }
    }

    pub fn moves(&mut self, moves: &[Move]) -> &mut Self {
        self.moves = moves.to_vec();
        self
    }

    pub fn outcome(&mut self, outcome: EndGame) -> &mut Self {
        self.outcome = Some(outcome);
        self
    }

    pub fn handicap(&mut self, points: &[Point]) -> &mut Self {
        self.handicap_points = points.to_vec();
        self.turn = !self.turn;
        self
    }

    pub fn size(&mut self, size: (u32, u32)) -> &mut Self {
        self.size = size;
        self
    }

    pub fn komi(&mut self, komi: f32) -> &mut Self {
        self.komi = komi;
        self
    }

    pub fn black_player(&mut self, black_player_name: &str) -> &mut Self {
        self.black_player = black_player_name.to_string();
        self
    }

    pub fn white_player(&mut self, white_player_name: &str) -> &mut Self {
        self.white_player = white_player_name.to_string();
        self
    }

    pub fn build(&mut self) -> Result<Game, String> {
        let mut goban: Goban = Goban::new(self.size.0 as usize);
        if !self.handicap_points.is_empty() {
            goban.push_many(&self.handicap_points, Color::Black);
        }
        let mut g = Game {
            goban,
            passes: 0,
            prisoners: (0, 0),
            outcome: self.outcome,
            turn: self.turn,
            komi: self.komi,
            rule: self.rule,
            handicap: self.handicap_points.len() as u8,
            plays: vec![],
            hashes: Default::default(),
        };

        for &m in &self.moves {
            g.play(m); // without verifications of Ko
        }

        Ok(g)
    }
}

impl Default for GameBuilder {
    fn default() -> Self {
        Self::new()
    }
}
