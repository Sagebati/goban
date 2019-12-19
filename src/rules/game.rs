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
use hash_hasher::{HashedSet, HashBuildHasher};

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

    #[cfg(feature = "history")]
    #[get = "pub"]
    plays: Vec<Goban>,

    #[get = "pub"]
    last_hash: u64,

    hashes: HashedSet<u64>,
}

impl Game {
    pub fn new(size: GobanSizes, rule: Rule) -> Self {
        let goban = Goban::new(size.into());
        let komi = 5.5;
        let pass = 0;
        #[cfg(feature = "history")]
            let plays = Vec::with_capacity(300);
        let prisoners = (0, 0);
        let handicap = 0;
        let hashes = HashedSet::with_capacity_and_hasher(300, HashBuildHasher::default());
        let last_hash = 0;
        Game {
            goban,
            turn: Player::Black,
            komi,
            prisoners,
            passes: pass,
            #[cfg(feature = "history")]
            plays,
            outcome: None,
            rule,
            handicap,
            hashes,
            last_hash,
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
            // two passes
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
    /// Generate all moves on all intersections.
    ///
    #[inline]
    fn pseudo_legals_shuffle(&self, rng: &mut impl rand::Rng) -> Vec<Point> {
        use rand::prelude::SliceRandom;
        let mut legals = self
            .goban
            .get_points_by_color(Color::None)
            .collect::<Vec<_>>();
        legals.shuffle(rng);
        legals
    }

    ///
    /// Returns a list with legals moves,
    /// In the list will appear suicides moves, and ko moves.
    ///
    #[inline]
    pub fn legals(&self) -> impl Iterator<Item=Point> + '_ {
        self.pseudo_legals()
            .map(move |s| Stone {
                color: self.turn.stone_color(),
                coordinates: s,
            })
            .filter(move |&s| self.rule.move_validation(&self, s).is_none())
            .map(|s| s.coordinates)
    }

    ///
    /// Returns a list with legals moves,
    /// In the list will appear suicides moves, and ko moves.
    ///
    #[inline]
    pub fn legals_shuffle(&self, rng: &mut impl rand::Rng) -> impl Iterator<Item=Point> + '_ {
        self.pseudo_legals_shuffle(rng)
            .into_iter()
            .map(move |s| Stone {
                color: self.turn.stone_color(),
                coordinates: s,
            })
            .filter(move |&s| self.rule.move_validation(&self, s).is_none())
            .map(|s| s.coordinates)
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
                let hash = self.goban.zobrist_hash();
                self.last_hash = hash;
                self.hashes.insert(hash);
                #[cfg(feature = "history")]
                    self.plays.push(self.goban.clone());
                self.goban.push((x, y), self.turn.stone_color());
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

    /// This methods plays a move then return the hash of the goban simulated,
    /// used in legals for fast move simulation in Ko ans Super Ko situations.
    pub fn play_for_verification(&self, (x, y): Point) -> u64 {
        let mut test_goban = self.goban.clone();
        test_goban.push((x, y), self.turn.stone_color());
        test_goban.remove_captured_stones_turn((!self.turn).stone_color());
        if self.rule.is_suicide_valid() {
            test_goban.remove_captured_stones_turn(self.turn.stone_color());
        }
        test_goban.zobrist_hash()
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
                            color: self.turn.stone_color(),
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
            for neighbor_go_string in self.goban.get_neighbors_strings(stone.coordinates) {
                if neighbor_go_string.color == stone.color {
                    if neighbor_go_string.number_of_liberties() != 1 {
                        return false;
                    }
                } else {
                    // capture move so not suicide
                    if neighbor_go_string.number_of_liberties() == 1 {
                        return false;
                    }
                }
            }
            true
        }
    }

    ///
    /// Returns true if the stone played in that point will capture another
    /// string.
    ///
    pub fn will_capture(&self, point: Point) -> bool {
        self.goban
            .get_neighbors_strings(point)
            .filter(|go_str_ptr| go_str_ptr.color != self.turn.stone_color())
            // if an enemy string has only liberty it's a capture move
            .any(|go_str_ptr| go_str_ptr.number_of_liberties() == 1)
    }

    ///
    /// Test if a play is ko.
    /// If the goban is in the configuration of the two plays ago returns true
    ///
    pub fn ko(&self, stone: Stone) -> bool {
        if self.last_hash == 0 || self.hashes.len() <= 2 || !self.will_capture(stone.coordinates) {
            false
        } else {
            self.play_for_verification(stone.coordinates) == self.last_hash
        }
    }

    ///
    /// Rule of the super Ko, if any before configuration was already played then return true.
    ///
    pub fn super_ko(&self, stone: Stone) -> bool {
        if !self.will_capture(stone.coordinates) {
            false
        } else {
            let hash_test_goban = self.play_for_verification(stone.coordinates);
            self.hashes.contains(&hash_test_goban)
        }
    }

    /// Displays the internal board.
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
        let pris = self
            .goban
            .remove_captured_stones_turn((!self.turn).stone_color());
        match self.turn {
            Black => new_prisoners.0 += pris,
            White => new_prisoners.1 += pris,
        };
        if self.rule.is_suicide_valid() {
            let pris = self
                .goban
                .remove_captured_stones_turn(self.turn.stone_color());
            match self.turn {
                Black => new_prisoners.1 += pris,
                White => new_prisoners.0 += pris,
            };
        }
        new_prisoners
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
        let mut goban: Goban = Goban::new((self.size.0 as usize, self.size.1 as usize ));
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
            #[cfg(feature = "history")]
            plays: vec![],
            hashes: Default::default(),
            last_hash: 0,
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
