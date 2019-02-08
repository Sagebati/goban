use crate::pieces::stones::Stone;
use crate::rules::game::Game;

pub mod game;
pub mod graph;

pub mod turn {
    pub const WHITE: bool = true;
    pub const BLACK: bool = false;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Player {
    White,
    Black,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EndGame {
    Score(f32, f32),
    WinnerByResign(Player),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PlayError {
    Ko,
    Suicide,
    GamePaused,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Rule {
    JapRule,
    ChineseRule,
}

pub trait RuleImpl {
    ///
    /// Counts the point for each player.
    /// (black points, white points)
    ///
    fn count_points(game: &Game) -> (f32, f32);
    ///
    /// Returns if a move is valid or not dependent of the rules.
    ///
    fn move_validation(game: &Game, stone: &Stone) -> Option<PlayError>;
}

impl Rule {
    pub fn count_points(&self, game: &Game) -> (f32, f32) {
        match self {
            Rule::JapRule => {
                let mut scores = game.calculate_territories();
                scores.0 += game.prisoners().0 as f32;
                scores.1 += game.prisoners().1 as f32;
                scores.1 += game.komi();

                scores
            }
            Rule::ChineseRule => {
                // Territories in seki are not counted
                let mut scores = game.calculate_territories();
                scores.1 += game.komi();
                scores
            }
        }
    }

    pub fn move_validation(&self, game: &Game, stone: &Stone) -> Option<PlayError> {
        match self {
            Rule::JapRule => {
                if game.is_suicide(stone) {
                    Some(PlayError::Suicide)
                } else if game.is_ko(stone) {
                    Some(PlayError::Ko)
                } else {
                    None
                }
            }
            Rule::ChineseRule => {
                if game.is_suicide(stone) {
                    return Some(PlayError::Suicide);
                } else if game.is_ko(stone) {
                    return Some(PlayError::Ko);
                } else {
                    None
                }
            }
        }
    }
}
