//! Module for ruling in the game of go.

use std::fmt::{Display, Error, Formatter};
use std::ops::Not;
use std::str::FromStr;

use crate::pieces::Nat;
use crate::pieces::stones::Color;
use crate::pieces::util::coord::Point;

mod deadstones;
pub mod game;
pub mod game_builder;
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

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Player::White => write!(f, "White"),
            Player::Black => write!(f, "Black"),
        }
    }
}

impl Player {
    /// Get the stone color of the player
    #[inline]
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
    Custom(Nat, Nat),
}

impl Into<(Nat, Nat)> for GobanSizes {
    fn into(self) -> (Nat, Nat) {
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
    Play(Nat, Nat),
}

impl From<Point> for Move {
    fn from((x0, x1): Point) -> Self {
        Move::Play(x0, x1)
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
            EndGame::WinnerByScore(p, _)
            | EndGame::WinnerByResign(p)
            | EndGame::WinnerByTime(p)
            | EndGame::WinnerByForfeit(p) => Some(p),
            EndGame::Draw => None,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Copy)]
pub enum PlayError {
    Ko,
    Suicide,
    GamePaused,
    FillEye,
    PointNotEmpty,
}

type FlagUInt = u32;
bitflags! {
    /// Behaviours not permitted, if the flag is up then the move is not legal.
    pub struct IllegalRules: FlagUInt{
        /// Rule that filters normal Ko move
        const KO = 1;
        /// Rule that filters SUPER KO moves
        const SUPERKO = 1 << 1;
        /// Rule that filters suicides moves
        const SUICIDE = 1 << 2;
        /// Rule that filters eyes from the legals
        const FILLEYE = 1 << 3;
    }
}
bitflags! {
    /// Types of scoring rules. the territory score is always added to the rules
    pub struct ScoreRules : FlagUInt {
        /// Stones needs to ben counted to the final score.
        const STONES = 1;
        /// The komi needs to be added.
        const KOMI = 1 << 1;
        /// The prisoners need to be added to the score.
        const PRISONNERS = 1 << 2;
    }
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
    #[inline(always)]
    pub fn komi(self) -> f32 {
        match self {
            Self::Japanese => 6.5,
            Self::Chinese => 7.5,
        }
    }

    #[inline(always)]
    pub fn illegal_flag(self) -> IllegalRules {
        match self {
            Self::Japanese => IllegalRules::KO | IllegalRules::SUICIDE,
            Self::Chinese => IllegalRules::SUPERKO | IllegalRules::KO | IllegalRules::SUICIDE,
        }
    }

    #[inline(always)]
    pub fn score_flag(self) -> ScoreRules {
        match self {
            Self::Japanese => ScoreRules::KOMI | ScoreRules::PRISONNERS,
            Self::Chinese => ScoreRules::KOMI | ScoreRules::STONES,
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
