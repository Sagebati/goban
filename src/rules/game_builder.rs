//! # GameBuilder Helper
//! Utility for creating complex games with non standard komi, handicap etc...
//! # Example
//! ```
//! use crate::goban::rules::game_builder::GameBuilder;
//! use crate::goban::rules::Rule;
//! use goban::rules::game::Game;
//! use goban::rules::JAPANESE;
//!
//! let mut builder = GameBuilder::default();
//! // or
//! let mut builder = Game::builder();
//! let game = builder
//!     .rule(JAPANESE)
//!     .size((19,19))
//!     .put_handicap(&[(3,3), (4,4)])
//!     .komi(10.)
//!     .build();
//! ```

use crate::pieces::goban::Goban;
use crate::pieces::stones::{Color, Stone};
use crate::pieces::util::coord::{Coord, Size};
use crate::rules::game::Game;
use crate::rules::{EndGame, Move, Rule, CHINESE};
use std::mem::take;

pub struct GameBuilder {
    size: Size,
    black_player: String,
    white_player: String,
    rule: Rule,
    komi: Option<f32>,
    handicap: Option<u32>,
    handicap_points: Vec<Coord>,
    turn: Option<Color>,
    moves: Vec<Move>,
    outcome: Option<EndGame>,
    setup: Vec<Stone>,
}

impl GameBuilder {
    pub fn moves(&mut self, moves: &[Move]) -> &mut Self {
        self.moves = moves.to_vec();
        self
    }

    pub fn outcome(&mut self, outcome: EndGame) -> &mut Self {
        self.outcome = Some(outcome);
        self
    }

    /// Overrides the turn because it's a game with handicap. So White begins.
    pub fn put_handicap(&mut self, points: &[Coord]) -> &mut Self {
        self.handicap_points = points.to_vec();
        self
    }

    pub fn handicap(&mut self, handicap: u32) -> &mut Self {
        self.handicap = Some(handicap);
        self
    }

    pub fn size(&mut self, size: Size) -> &mut Self {
        self.size = size;
        self
    }

    pub fn komi(&mut self, komi: f32) -> &mut Self {
        self.komi = Some(komi);
        self
    }

    pub fn turn(&mut self, turn: Color) -> &mut Self {
        self.turn = Some(turn);
        self
    }

    pub fn black_player(&mut self, black_player_name: &str) -> &mut Self {
        self.black_player = black_player_name.to_string();
        self
    }

    pub fn rule(&mut self, rule: Rule) -> &mut Self {
        self.rule = rule;
        self
    }

    pub fn white_player(&mut self, white_player_name: &str) -> &mut Self {
        self.white_player = white_player_name.to_string();
        self
    }

    pub fn add(&mut self, stone: Stone) -> &mut Self {
        self.setup.push(stone);
        self
    }

    fn build_inner(mut self) -> Result<Game, String> {
        let mut goban: Goban = Goban::new(self.size);

        let handicap = self.handicap.unwrap_or(self.handicap_points.len() as u32);

        for point in self.handicap_points {
            goban.push(point, Color::Black);
        }

        // Setup
        for s in self.setup {
            goban.push_stone(s);
        }

        if let Some(komi) = self.komi {
            self.rule.komi = komi;
        }

        let turn = {
            self.turn.unwrap_or(if handicap != 0 {
                Color::White
            } else {
                Color::Black
            })
        };

        let mut g = Game {
            goban: goban.clone(),
            passes: 0,
            prisoners: (0, 0),
            outcome: self.outcome,
            turn,
            rule: self.rule,
            handicap,
            history: Default::default(),
            ko_point: None,
        };

        // Moves to play
        for &m in &self.moves {
            g.play(m);
        }

        Ok(g)
    }

    pub fn build(&mut self) -> Result<Game, String> {
        let this = take(self);

        this.build_inner()
    }
}

impl Default for GameBuilder {
    fn default() -> Self {
        GameBuilder {
            size: (19, 19),
            black_player: "".to_string(),
            white_player: "".to_string(),
            handicap_points: vec![],
            rule: CHINESE,
            komi: None,
            turn: None,
            moves: vec![],
            outcome: None,
            setup: vec![],
            handicap: None,
        }
    }
}

impl Game {
    pub fn builder() -> GameBuilder {
        GameBuilder::default()
    }
}
