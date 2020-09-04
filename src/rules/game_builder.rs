//! # GameBuilder Helper
//! Utility for creating complex games with non standard komi, handicap etc...
//! # Exemple
//! ```
//! use crate::goban::rules::game_builder::GameBuilder;
//! use crate::goban::rules::Rule;
//! use goban::rules::game::Game;
//!
//! let mut builder = GameBuilder::default();
//! // or
//! let mut builder = Game::builder();
//! let game = builder
//!     .rule(Rule::Japanese)
//!     .size((19,19))
//!     .handicap(&[(3,3), (4,4)])
//!     .komi(10.)
//!     .build();
//! ```

use crate::pieces::goban::Goban;
use crate::pieces::Nat;
use crate::pieces::stones::Color;
use crate::pieces::util::coord::Point;
use crate::rules::{EndGame, Move, Player, Rule};
use crate::rules::game::Game;
use crate::rules::Rule::Chinese;

pub struct GameBuilder {
    size: (u32, u32),
    komi: f32,
    manual_komi: bool,
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
            komi: Chinese.komi(),
            manual_komi: false,
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

    /// Overrides the turn because it's a game with handicap. So White begins.
    pub fn handicap(&mut self, points: &[Point]) -> &mut Self {
        self.handicap_points = points.to_vec();
        self.turn = Player::White;
        self
    }

    pub fn size(&mut self, size: (u32, u32)) -> &mut Self {
        self.size = size;
        self
    }

    pub fn komi(&mut self, komi: f32) -> &mut Self {
        self.komi = komi;
        self.manual_komi = true;
        self
    }

    pub fn black_player(&mut self, black_player_name: &str) -> &mut Self {
        self.black_player = black_player_name.to_string();
        self
    }

    pub fn rule(&mut self, rule: Rule) -> &mut Self {
        self.rule = rule;
        if !self.manual_komi {
            self.komi = rule.komi();
        }
        self
    }

    pub fn white_player(&mut self, white_player_name: &str) -> &mut Self {
        self.white_player = white_player_name.to_string();
        self
    }

    pub fn build(&mut self) -> Result<Game, String> {
        let mut goban: Goban = Goban::new((self.size.0 as Nat, self.size.1 as Nat));

        goban.push_many(&self.handicap_points, Color::Black);

        let mut g = Game {
            goban,
            passes: 0,
            prisoners: (0, 0),
            outcome: self.outcome,
            move_num: 0,
            turn: self.turn,
            komi: self.komi,
            rule: self.rule,
            handicap: self.handicap_points.len() as u8,
            #[cfg(feature = "history")]
            history: vec![],
            #[cfg(feature = "history")]
            moves_history: vec![],
            hashes: Default::default(),
            last_hash: 0,
            ko_point: None,
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

impl Game {
    pub fn builder() -> GameBuilder {
        GameBuilder::default()
    }
}
