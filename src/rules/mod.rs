//! Module for ruling in the game of go.

use crate::pieces::stones::{Color, Stone};
use crate::pieces::util::coord::Point;
use crate::rules::game::Game;
use std::ops::Not;
use std::str::FromStr;

pub mod game;
mod sgf_bridge;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
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

impl Player {
    pub fn stone_color(self) -> Color {
        match self {
            Player::Black => Color::Black,
            Player::White => Color::White,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GobanSizes {
    Nineteen,
    Nine,
    Thirteen,
    Custom(usize, usize),
}

impl Into<(usize, usize)> for GobanSizes {
    fn into(self) -> (usize, usize) {
        match self {
            GobanSizes::Nine => (9, 9),
            GobanSizes::Thirteen => (13, 13),
            GobanSizes::Nineteen => (19, 19),
            GobanSizes::Custom(height, width) => (height, width),
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Move {
    Pass,
    Resign(Player),
    Play(usize, usize),
}

impl From<Point> for Move {
    fn from(x: (usize, usize)) -> Self {
        Move::Play(x.0, x.1)
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum EndGame {
    WinnerByScore(Player, f32),
    WinnerByResign(Player),
    WinnerByTime(Player),
    WinnerByForfeit(Player),
    Draw,
}

impl EndGame {
    /// Return the winner of the game, if none the game is draw.
    #[inline]
    pub fn get_winner(self) -> Option<Player> {
        match self {
            EndGame::WinnerByScore(p, _) |
            EndGame::WinnerByResign(p) |
            EndGame::WinnerByTime(p) |
            EndGame::WinnerByForfeit(p) => Some(p),
            EndGame::Draw => None,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Copy)]
pub enum PlayError {
    Ko,
    Suicide,
    GamePaused,
}

///
/// This enum describes the rules for the game.
/// for example in chinese rules we don't count prisoners.
///
#[derive(Clone, Eq, PartialEq, Debug, Copy)]
pub enum Rule {
    Japanese,
    Chinese, // Transparent to Taylor-Davis
}

impl Rule {
    /// Gets the komi defined in the rule
    #[inline]
    pub fn komi(self) -> f32 {
        match self {
            Self::Japanese => 6.5,
            Self::Chinese => 7.5,
        }
    }

    /// Count the points of the game including komi and territories.
    #[inline]
    pub fn count_points(self, game: &Game) -> (f32, f32) {
        let (black_score, white_score) = game.goban().calculate_territories();
        match self {
            Rule::Japanese => (black_score as f32, white_score as f32 + game.komi()),
            Rule::Chinese => {
                // Territories in seki are not counted
                let (black_stones, white_stones) = game.goban().number_of_stones();
                (
                    black_score as f32 + black_stones as f32,
                    white_score as f32 + white_stones as f32 + game.komi(),
                )
            }
        }
    }

    /// Specify the constraints in the move validation by rule.
    pub fn move_validation(self, game: &Game, stone: Stone) -> Option<PlayError> {
        match self {
            Rule::Japanese => {
                if game.is_suicide(stone) {
                    Some(PlayError::Suicide)
                } else if game.ko(stone) {
                    Some(PlayError::Ko)
                } else {
                    None
                }
            }
            Rule::Chinese => {
                if game.is_suicide(stone) {
                    Some(PlayError::Suicide)
                }else if game.super_ko(stone) {
                    Some(PlayError::Ko)
                } else {
                    None
                }
            }
        }
    }

    pub fn is_suicide_valid(self) -> bool {
        match self {
            Rule::Chinese | Rule::Japanese => false
        }
    }
}

impl FromStr for Rule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "JAP" => Ok(Rule::Japanese),
            "CHI" => Ok(Rule::Chinese),
            _ => Err(format!("The rule {} is not implemented yet.", s)),
        }
    }
}
