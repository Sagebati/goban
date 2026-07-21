//! Module for ruling in the game of go.

use std::str::FromStr;

use crate::pieces::stones::Color;
use crate::pieces::util::coord::{Coord, Size};
use crate::pieces::Nat;

pub mod game;
mod sgf_bridge;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GobanSizes {
    Nineteen,
    Nine,
    Thirteen,
    Custom(usize, usize),
}

impl From<GobanSizes> for Size {
    fn from(goban_sizes: GobanSizes) -> Size {
        match goban_sizes {
            GobanSizes::Nine => (9, 9),
            GobanSizes::Thirteen => (13, 13),
            GobanSizes::Nineteen => (19, 19),
            GobanSizes::Custom(height, width) => (height as u8, width as u8),
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
    Resign(Color),
    Play(Nat, Nat),
}

impl From<Coord> for Move {
    fn from((x0, x1): Coord) -> Self {
        Move::Play(x0, x1)
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum EndGame {
    WinnerByScore(Color, f32),
    WinnerByResign(Color),
    WinnerByTime(Color),
    WinnerByForfeit(Color),
    Draw,
}

impl EndGame {
    /// Return the winner of the game, if none the game is draw.
    #[inline]
    pub const fn get_winner(self) -> Option<Color> {
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rule {
    pub komi: f32,
    pub flag_illegal: IllegalRules,
    pub flag_score: ScoreRules,
}

pub static JAPANESE: Rule = Rule {
    komi: 6.5,
    flag_illegal: IllegalRules::from_bits_truncate(
        IllegalRules::KO.bits() | IllegalRules::SUICIDE.bits(),
    ),
    flag_score: ScoreRules::from_bits_truncate(
        ScoreRules::KOMI.bits() | ScoreRules::PRISONNERS.bits(),
    ),
};

pub static CHINESE: Rule = Rule {
    komi: 7.5,
    flag_illegal: IllegalRules::from_bits_truncate(
        IllegalRules::KO.bits() | IllegalRules::SUPERKO.bits() | IllegalRules::SUICIDE.bits(),
    ),
    flag_score: ScoreRules::from_bits_truncate(ScoreRules::KOMI.bits() | ScoreRules::STONES.bits()),
};

/// American Go Association rules: area scoring with positional superko.
/// The official AGA rules permit multi-stone suicide (while still forbidding
/// single-stone suicide); this crate's flag model cannot distinguish the two
/// cases, so suicide is disallowed entirely here, matching how it is played
/// in practice.
pub static AGA: Rule = Rule {
    komi: 7.5,
    flag_illegal: IllegalRules::from_bits_truncate(
        IllegalRules::KO.bits() | IllegalRules::SUPERKO.bits() | IllegalRules::SUICIDE.bits(),
    ),
    flag_score: ScoreRules::from_bits_truncate(ScoreRules::KOMI.bits() | ScoreRules::STONES.bits()),
};

/// Tromp-Taylor rules: the canonical formal ruleset used for computer Go.
/// Area scoring, positional superko, and suicide (including multi-stone
/// suicide) is explicitly allowed.
pub static TROMP_TAYLOR: Rule = Rule {
    komi: 7.5,
    flag_illegal: IllegalRules::from_bits_truncate(IllegalRules::SUPERKO.bits()),
    flag_score: ScoreRules::from_bits_truncate(ScoreRules::KOMI.bits() | ScoreRules::STONES.bits()),
};

/// French rules (Fédération Française de Go): identical to the AGA ruleset,
/// which the FFG has used since 1994.
pub static FRENCH: Rule = AGA;

/// New Zealand rules: area scoring with positional superko, notable for
/// explicitly allowing suicide of stone groups of any size.
pub static NEW_ZEALAND: Rule = Rule {
    komi: 7.5,
    flag_illegal: IllegalRules::from_bits_truncate(IllegalRules::SUPERKO.bits()),
    flag_score: ScoreRules::from_bits_truncate(ScoreRules::KOMI.bits() | ScoreRules::STONES.bits()),
};

/// Ing (Ing Chang-ki / Ing Cup) rules: area scoring via a stone-counting
/// method, with an integer komi of 8 and suicide disallowed. The official
/// rules break ties in Black's favor instead of using a half-point komi,
/// and distinguish "fighting kos" from "disturbing kos" with a complex
/// adjudication procedure; this crate cannot model either nuance, so ties
/// are left as draws and positional superko is used as the closest
/// approximation of the ko handling.
pub static ING: Rule = Rule {
    komi: 8.,
    flag_illegal: IllegalRules::from_bits_truncate(
        IllegalRules::KO.bits() | IllegalRules::SUPERKO.bits() | IllegalRules::SUICIDE.bits(),
    ),
    flag_score: ScoreRules::from_bits_truncate(ScoreRules::KOMI.bits() | ScoreRules::STONES.bits()),
};

impl FromStr for Rule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "JAP" => Ok(JAPANESE),
            "CHI" => Ok(CHINESE),
            "AGA" => Ok(AGA),
            "TT" => Ok(TROMP_TAYLOR),
            "NZ" => Ok(NEW_ZEALAND),
            "FRA" => Ok(FRENCH),
            "ING" => Ok(ING),
            _ => Err(format!("The rule {s} is not implemented yet.")),
        }
    }
}
