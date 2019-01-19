use crate::pieces::stones::Stone;
use crate::rules::game::Game;

pub mod game;
pub mod graph;


pub trait Rule {
    ///
    /// Counts the point for each player.
    ///
    fn count_points<T: Rule>(game: &Game<T>) -> (f32, f32);
    ///
    /// Returns if a move is valid or not.
    ///
    fn move_validation<T: Rule>(game: &Game<T>, stone: &Stone) -> bool;
}


pub struct JapRule;


impl Rule for JapRule {
    fn count_points<T: Rule>(game: &Game<T>) -> (f32, f32) {
        let mut scores = game.calculate_territories();
        scores.0 += game.prisoners().0 as f32;
        scores.1 += game.prisoners().1 as f32;

        scores
    }

    fn move_validation<T: Rule>(game: &Game<T>, stone: &Stone) -> bool {
        if game.is_suicide(stone) {
            return false;
        }
        if game.is_ko(stone) {
            return false;
        }
        true
    }
}