use crate::pieces::util::coord::Point;
use crate::rules::game::{Game, GameBuilder};
use crate::rules::{EndGame, Move, Player};
use sgf_parser::{Action, Color, Outcome, SgfToken};

impl Game {
    pub fn from_sgf(sgf_str: &str) -> Result<Self, String> {
        let game_tree = match sgf_parser::parse(sgf_str) {
            Ok(game) => Ok(game),
            Err(e) => Err(e.to_string()),
        }?;
        let mut gamebuilder: GameBuilder = Default::default();
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
                            gamebuilder.komi(*komi);
                        }
                        SgfToken::Size(x, y) => {
                            gamebuilder.size((*x, *y));
                        }
                        SgfToken::Result(o) => {
                            gamebuilder.outcome((*o).into());
                        }
                        SgfToken::Add {
                            color,
                            coordinate: (x, y),
                        } if *color == Color::Black => {
                            handicap.push((*x as usize - 1, *y as usize - 1));
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
        gamebuilder.handicap(&handicap);
        gamebuilder.moves(&moves);
        gamebuilder.build()
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
            Action::Move(col, line) => Move::Play((line - 1) as usize, (col - 1) as usize),
            Action::Pass => Move::Pass,
        }
    }
}
