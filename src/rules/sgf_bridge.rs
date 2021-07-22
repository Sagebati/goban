use sgf_parser::{Action, Color, Outcome, RuleSet, SgfToken};

use crate::pieces::Nat;
use crate::pieces::util::coord::Point;
use crate::rules::{CHINESE, EndGame, JAPANESE, Move, Player, Rule};
use crate::rules::game::Game;
use crate::rules::game_builder::GameBuilder;

impl Game {
    pub fn from_sgf(sgf_str: &str) -> Result<Self, String> {
        let game_tree = match sgf_parser::parse(sgf_str) {
            Ok(game) => Ok(game),
            Err(e) => Err(e.to_string()),
        }?;
        let mut game_builder: GameBuilder = Default::default();
        let mut first = true;
        let mut moves = vec![];
        let mut handicap: Vec<Point> = vec![];

        for node in game_tree.iter() {
            if first {
                // first token if the root token
                // Game information
                for token in &node.tokens {
                    match token {
                        SgfToken::Komi(komi) => {
                            game_builder.komi(*komi);
                        }
                        SgfToken::Size(x, y) => {
                            game_builder.size((*x, *y));
                        }
                        SgfToken::Result(o) => {
                            game_builder.outcome((*o).into());
                        }
                        SgfToken::Add {
                            color,
                            coordinate: (x, y),
                        } if *color == Color::Black => {
                            handicap.push(((*x - 1) as Nat, (*y - 1) as Nat));
                        }
                        SgfToken::Rule(rule) => {
                            game_builder.rule(rule.clone().into());
                        }
                        //TODO another options
                        _ => (),
                    }
                }
                first = false;
            } else if !node.tokens.is_empty() {
                let token = node.tokens.first().unwrap();
                if let SgfToken::Move { action, .. } = token {
                    moves.push((*action).into());
                }
            }
        }
        game_builder.handicap(&handicap);
        game_builder.moves(&moves);
        game_builder.build()
    }
}

impl From<RuleSet> for Rule {
    fn from(r: RuleSet) -> Self {
        match r {
            RuleSet::Japanese => JAPANESE,
            RuleSet::Chinese => CHINESE,
            _ => panic!(
                "The rule {} is not implemented yet !",
                r.to_string()
            ),
        }
    }
}

impl From<Outcome> for EndGame {
    fn from(o: Outcome) -> Self {
        match o {
            Outcome::WinnerByResign(c) => EndGame::WinnerByResign(c.into()),
            Outcome::WinnerByForfeit(c) => EndGame::WinnerByForfeit(c.into()),
            Outcome::WinnerByPoints(c, p) => EndGame::WinnerByScore(c.into(), p),
            Outcome::WinnerByTime(c) => EndGame::WinnerByTime(c.into()),
            Outcome::Draw => EndGame::Draw,
        }
    }
}

impl From<Color> for Player {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => Player::Black,
            Color::White => Player::White,
        }
    }
}

impl From<Action> for Move {
    fn from(a: Action) -> Self {
        match a {
            Action::Move(col, line) => Move::Play((line - 1) as Nat, (col - 1) as Nat),
            Action::Pass => Move::Pass,
        }
    }
}
