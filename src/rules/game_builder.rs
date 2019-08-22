use std::string::ToString;
use crate::pieces::util::coord::Coord;
use crate::rules::game::{Game, GobanSizes};
use crate::rules::Rule;
use crate::rules::Rule::Chinese;

pub struct GameBuilder {
    size: usize,
    komi: f32,
    player_black: String,
    player_white: String,
    rule: Rule,
    handicap_points: Option<Vec<Coord>>,
}

impl GameBuilder {
    pub fn new() -> GameBuilder {
        GameBuilder {
            size: 19,
            komi: 7.5,
            player_black: "".to_string(),
            player_white: "".to_string(),
            handicap_points: None,
            rule: Chinese,
        }
    }


    pub fn handicap(&mut self, points: &[Coord]) -> &mut Self {
        self.handicap_points = Some(points.to_vec());
        self
    }

    pub fn size(&mut self, size: usize) -> &mut Self {
        self.size = size;
        self
    }

    pub fn komi(&mut self, komi: f32) -> &mut Self {
        self.komi = komi;
        self
    }

    pub fn build(&self) -> Result<Game, String> {
        let mut g = Game::new(GobanSizes::from(self.size), self.rule);
        g.set_komi(self.komi);
        if let Some(handicap_stones) = &self.handicap_points {
            g.put_handicap(handicap_stones);
        }
        Ok(g)
    }
}