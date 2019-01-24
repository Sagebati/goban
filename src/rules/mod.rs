use crate::pieces::stones::Stone;
use crate::rules::game::Game;

pub mod game;
pub mod graph;

pub mod turn {
    pub const WHITE: bool = true;
    pub const BLACK: bool = false;
}

#[derive(Copy, Clone)]
pub enum PlayError {
    Ko,
    Suicide,
    GamePaused,
}

pub trait Rule {
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


///
/// Struct to identify the japanese rules.
///
pub struct JapRule;

impl Rule for JapRule {
    fn count_points(game: &Game) -> (f32, f32) {
        let mut scores = game.calculate_territories();
        scores.0 += game.prisoners().0 as f32;
        scores.1 += game.prisoners().1 as f32;
        scores.1 += game.komi();

        scores
    }

    fn move_validation(game: &Game, stone: &Stone) -> Option<PlayError> {
        if game.is_suicide(stone) {
            return Some(PlayError::Suicide);
        }
        if game.is_ko(stone) {
            return Some(PlayError::Ko);
        }
        None
    }
}