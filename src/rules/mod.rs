use crate::pieces::stones::Stone;
use crate::rules::game::Game;

pub mod game;
pub mod graph;


pub mod turn {
    pub const WHITE : bool = true;
    pub const BLACK : bool = false;
}

#[derive(Copy, Clone)]
pub enum Conflicts {
    Ko,
    Suicide,
    GamePaused
}

pub trait Rule {
    ///
    /// Counts the point for each player.
    ///
    fn count_points(game: &Game) -> (f32, f32);
    ///
    /// Returns if a move is valid or not.
    ///
    fn move_validation(game: &Game, stone: &Stone) -> Option<Conflicts>;
}


pub struct JapRule;

impl Rule for JapRule {
    fn count_points(game: &Game) -> (f32, f32) {
        let mut scores = game.calculate_territories();
        scores.0 += game.prisoners().0 as f32;
        scores.1 += game.prisoners().1 as f32;
        scores.1 += game.komi();

        scores
    }

    fn move_validation(game: &Game, stone: &Stone) -> Option<Conflicts> {
        if game.is_suicide(stone) {
            return Some(Conflicts::Suicide);
        }
        if game.is_ko(stone) {
            return Some(Conflicts::Ko);
        }
        None
    }
}