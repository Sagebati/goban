//! Module for ruling in the game of go.

use crate::pieces::stones::{Color, Stone};
use crate::rules::game::Game;
use std::ops::Not;

pub mod game;
pub mod graph;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Player {
    White,
    Black,
}

impl Not for Player {
    type Output = Player;

    fn not(self) -> Self::Output {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

impl Into<Color> for Player {
    fn into(self) -> Color {
        match self {
            Player::Black => Color::Black,
            Player::White => Color::White,
        }
    }
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

///
/// This enum describes the rules for the game.
/// for example in chinese rules we don't count prisoners.
///
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Rule {
    Japanese,
    Chinese,
}

impl Rule {
    ///
    /// Count the points of the game
    ///
    pub fn count_points(&self, game: &Game) -> (f32, f32) {
        match self {
            Rule::Japanese => {
                let mut scores = game.calculate_territories();
                scores.0 += game.prisoners().0 as f32;
                scores.1 += game.prisoners().1 as f32;
                scores.1 += game.komi();

                scores
            }
            Rule::Chinese => {
                // Territories in seki are not counted
                let mut scores = game.calculate_territories();
                let ns = game.number_of_stones();
                scores.0 += ns.0 as f32;
                scores.1 += ns.1 as f32;
                scores.1 += game.komi();
                scores
            }
        }
    }
    ///
    /// Specify the constraints in the move validation by rule.
    ///
    pub fn move_validation(&self, game: &Game, stone: &Stone) -> Option<PlayError> {
        match self {
            Rule::Japanese => {
                if game.is_suicide(stone) {
                    Some(PlayError::Suicide)
                } else if game.is_ko(stone) {
                    Some(PlayError::Ko)
                } else {
                    None
                }
            }
            Rule::Chinese => {
                if game.is_suicide(stone) {
                    Some(PlayError::Suicide)
                } else if game.is_ko(stone) {
                    Some(PlayError::Ko)
                } else {
                    None
                }
            }
        }
    }
}
