use crate::pieces::util::coord::Coord;
use crate::rules::game::{Game, GobanSizes};
use crate::rules::{Rule, Player};
use crate::rules::Rule::Chinese;
use std::string::ToString;
use crate::pieces::goban::Goban;
use crate::pieces::stones::Color;

pub struct GameBuilder {
    size: (u32, u32),
    komi: f32,
    black_player: String,
    white_player: String,
    rule: Rule,
    handicap_points: Vec<Coord>,
    turn: Player,
}

impl GameBuilder {
    pub fn new() -> GameBuilder {
        GameBuilder {
            size: (19, 19),
            komi: 0.,
            black_player: "".to_string(),
            white_player: "".to_string(),
            handicap_points: None,
            rule: Chinese,
            turn: Player::Black,
        }
    }

    pub fn handicap(&mut self, points: &[Coord]) -> &mut Self {
        self.handicap_points = points.to_vec();
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
        if self.handicap_points.len() != 0 {
            goban.push_many(handicap_stones.iter(), Color::Black);
            self.turn = Player::White;
        }
        Ok(Game {
            goban,
            passes: 0,
            prisoners: (0, 0),
            resigned: None,
            turn: self.turn,
            komi: self.komi,
            rule: self.rule,
            handicap: self.handicap_points.len() as u8,
            plays: vec![],
            hashes: Default::default(),
        })
    }
}

impl Default for GameBuilder {
    fn default() -> Self {
        Self::new()
    }
}
