use crate::pieces::goban::*;
use crate::pieces::stones::Color::{Black, White};
use crate::pieces::stones::{Color, Stone, EMPTY};
use crate::pieces::util::coord::{corner_points, is_coord_valid, two_to_1dim, Coord, Size};
use crate::pieces::Nat;
use crate::rules::EndGame::{Draw, WinnerByScore};
use crate::rules::Rule;
use crate::rules::{EndGame, GobanSizes, IllegalRules, Move, ScoreRules};
use crate::rules::{PlayError, CHINESE};
use hash_hasher::{HashBuildHasher, HashHasher};
use indexmap::IndexSet;
use std::hash::BuildHasherDefault;
use std::ops::Deref;

#[cfg(feature = "deadstones")]
pub mod dead_stones;
pub mod game_builder;

/// Entrypoint struct of the library.
/// It represents a Game of Go.
/// Outcome of the suicide test for a move.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SuicideKind {
    NotSuicide,
    /// The played stone connects to nothing and dies alone.
    LoneStone,
    /// The played stone and at least one friendly chain die.
    Chain,
}

#[derive(Clone, Debug)]
pub struct Game {
    goban: Goban,
    passes: u32,
    prisoners: (u32, u32),
    /// None if the game is not finished,
    outcome: Option<EndGame>,
    turn: Color,
    rule: Rule,
    handicap: u32,
    history: IndexSet<Snapshot, BuildHasherDefault<HashHasher>>,
    ko_point: Option<Coord>,
}

impl Deref for Game {
    type Target = Goban;

    fn deref(&self) -> &Self::Target {
        &self.goban
    }
}

impl Game {
    /// Crates a new game for playing Go
    pub fn new(size: GobanSizes, rule: Rule) -> Self {
        let (h, w) = size.into();
        let goban = Goban::new(size.into());
        let length = h as usize * w as usize;
        let prisoners = (0, 0);
        let handicap = 0;
        let history = IndexSet::with_capacity_and_hasher(length, HashBuildHasher::default());
        Self {
            goban,
            turn: Color::Black,
            prisoners,
            passes: 0,
            outcome: None,
            rule,
            handicap,
            history,
            ko_point: None,
        }
    }
}

impl Game {
    /// Resume the game when to players have passed, and want to continue.
    #[inline]
    pub fn resume(&mut self) {
        self.passes = 0;
    }

    #[inline]
    pub fn set_komi(&mut self, komi: f32) {
        self.rule.komi = komi;
    }

    #[inline]
    pub fn komi(&self) -> f32 {
        self.rule.komi
    }

    #[inline]
    pub fn size(&self) -> Size {
        self.goban.size()
    }

    pub fn prisoners(&self) -> (u32, u32) {
        self.prisoners
    }

    pub fn goban(&self) -> &Goban {
        &self.goban
    }

    pub fn turn(&self) -> Color {
        self.turn
    }

    /// The positions before each move, oldest first, rebuilt from the
    /// compact snapshots kept for super ko.
    #[cfg(feature = "history")]
    pub fn history(&self) -> impl Iterator<Item = Goban> + '_ {
        self.history.iter().map(Goban::from)
    }

    /// True when the game is over (two passes, or no more legals moves, Resign)
    #[inline]
    pub fn is_over(&self) -> bool {
        if self.outcome.is_some() {
            true
        } else {
            self.passes >= 2
        }
    }

    /// Returns the endgame.
    /// None if the game is not finished
    #[inline]
    pub fn outcome(&self) -> Option<EndGame> {
        if !self.is_over() {
            None
        } else if self.outcome.is_some() {
            self.outcome
        } else {
            // two passes
            let scores = self.calculate_score();
            if (scores.0 - scores.1).abs() < f32::EPSILON {
                Some(Draw)
            } else if scores.0 > scores.1 {
                Some(WinnerByScore(Black, scores.0 - scores.1))
            } else {
                Some(WinnerByScore(White, scores.1 - scores.0))
            }
        }
    }

    /// Generate all moves on all empty intersections. Lazy.
    #[inline]
    pub fn pseudo_legals(&self) -> impl Iterator<Item = Coord> + '_ {
        self.goban.get_empty_coords()
    }

    /// Get all moves on all empty intersections.
    pub fn pseudo_legals_vec(&self) -> Vec<Coord> {
        let size = self.size();
        let mut vec = Vec::with_capacity((size.0 * size.1) as usize);
        let board = self.goban.board();
        for i in 0..size.0 as Nat {
            for j in 0..size.1 as Nat {
                if board[two_to_1dim(size, (i, j))].is_none() {
                    vec.push((i, j));
                }
            }
        }
        vec
    }

    /// Returns a list with legals moves. from the rule specified in at the creation.
    #[inline]
    pub fn legals(&self) -> impl Iterator<Item = Coord> + '_ {
        self.legals_by(self.rule.flag_illegal)
    }

    /// Return a list with the legals moves. doesn't take the rule specified in the game but take
    /// the one passed on parameter.
    #[inline]
    pub fn legals_by(&self, legals_rules: IllegalRules) -> impl Iterator<Item = Coord> + '_ {
        self.pseudo_legals()
            .filter(move |&s| self.check_point_by(s, legals_rules).is_none())
    }

    /// Method to play on the goban or pass.
    /// (0,0) is in the top left corner of the goban.
    ///
    /// # Panics
    ///
    /// If the coordinates of the move are outside the board.
    pub fn play(&mut self, play: Move) -> &mut Self {
        match play {
            Move::Pass => {
                assert!(self.passes < 2, "This game is already paused");
                self.turn = !self.turn;
                self.ko_point = None;
                self.passes += 1;
                self
            }
            Move::Play(x, y) => {
                //let hash = self.goban.zobrist_hash();
                self.history.insert(self.goban.snapshot());
                let (dead_rens, added_ren) = self.goban.push_wth_feedback((x, y), self.turn);
                self.ko_point = None;
                self.remove_captured_stones(&dead_rens, added_ren);
                //self.prisoners = self.remove_captured_stones();
                self.turn = !self.turn;
                self.passes = 0;
                self
            }
            Move::Resign(player) => {
                self.outcome = Some(EndGame::WinnerByResign(player));
                self
            }
        }
    }

    /// Plays a move then return the simulated goban,
    /// used in legals for fast move simulation in Super Ko situations.
    pub fn play_for_verification(&self, (x, y): Coord) -> Goban {
        let mut test_goban = self.goban;
        let (dead_go_strings, added_ren) = test_goban.push_wth_feedback((x, y), self.turn);
        test_goban.remove_captured_stones_aux(
            !self.rule.flag_illegal.contains(IllegalRules::SUICIDE),
            &dead_go_strings,
            added_ren,
        );
        test_goban
    }

    /// Method to play but it verifies if the play is legal or not.
    ///
    /// # Errors
    ///
    /// If the move is a suicide return `[PLayError::SuicideMove]`
    /// If the move is a ko returns `[PlayError::SuicideMove]`Ko
    /// if point is already filled, then return `[PlayError::PointNotEmpty]``
    /// If the game is paused, then return `[PlayError::GamePaused]`
    pub fn try_play(&mut self, play: Move) -> Result<&mut Self, PlayError> {
        if self.passes == 2 {
            Err(PlayError::GamePaused)
        } else {
            match play {
                Move::Play(x, y) => {
                    if self.goban.get_color((x, y)) != EMPTY {
                        Err(PlayError::PointNotEmpty)
                    } else if let Some(c) = self.check_point((x as Nat, y as Nat)) {
                        Err(c)
                    } else {
                        Ok(self.play(play))
                    }
                }
                Move::Pass | Move::Resign(_) => Ok(self.play(play)),
            }
        }
    }

    /// Put the handicap stones on the goban.
    /// This puts the turn for white but doesn't update the komi.
    pub fn put_handicap(&mut self, points: &[Coord]) {
        self.handicap = points.len() as u32;
        points.iter().for_each(|&coord| {
            self.goban.push(coord, Color::Black);
        });
        self.turn = Color::White;
    }

    /// Calculates score. with prisoners and komi.
    /// Dependant of the rule in the game.
    #[inline]
    pub fn calculate_score(&self) -> (f32, f32) {
        self.calculate_score_by(self.rule.flag_score)
    }

    /// Calculates the score by the rule passed in parameter.
    pub fn calculate_score_by(&self, rule: ScoreRules) -> (f32, f32) {
        let (black_score, white_score) = self.goban.calculate_territories();
        let mut black_score = black_score as f32;
        let mut white_score = white_score as f32;
        if rule.contains(ScoreRules::PRISONNERS) {
            black_score += self.prisoners.0 as f32;
            white_score += self.prisoners.1 as f32;
        }
        if rule.contains(ScoreRules::STONES) {
            let (black_stones, white_stones) = self.goban.number_of_stones();
            black_score += black_stones as f32;
            white_score += white_stones as f32;
        }
        if rule.contains(ScoreRules::KOMI) {
            white_score += self.komi();
        }

        (black_score, white_score)
    }

    /// Returns true if the stone played in that point will capture another
    /// string.
    pub fn will_capture(&self, point: Coord) -> bool {
        self.goban
            .get_connected_groups(point)
            .into_iter()
            .any(|go_str| go_str.color != self.turn && go_str.is_atari())
    }

    /// Test if a point is legal or not for the current player,
    #[inline]
    pub fn check_point(&self, coord: Coord) -> Option<PlayError> {
        self.check_point_by(coord, self.rule.flag_illegal)
    }

    /// Test if a point is legal or not by the rule passed in parameter.
    pub fn check_point_by(&self, coord: Coord, illegal_rules: IllegalRules) -> Option<PlayError> {
        let stone = Stone {
            coord,
            color: self.turn,
        };
        if self.goban.get_color(coord).is_some() {
            Some(PlayError::PointNotEmpty)
        } else if illegal_rules.contains(IllegalRules::SUICIDE) && self.check_suicide(stone) {
            Some(PlayError::Suicide)
        } else if illegal_rules.contains(IllegalRules::KO) && self.check_ko(stone) {
            Some(PlayError::Ko)
        } else if illegal_rules.contains(IllegalRules::FILLEYE) && self.check_eye(stone) {
            Some(PlayError::FillEye)
        } else if illegal_rules.contains(IllegalRules::SUPERKO)
            && self.check_super_ko_by(stone, !illegal_rules.contains(IllegalRules::SUICIDE))
        {
            Some(PlayError::Ko)
        } else {
            None
        }
    }

    /// Detects true single-point eyes, the points a player should never fill.
    ///
    /// The point must be empty with every neighbour of `color`, and the
    /// player must hold enough diagonals: at least three of the four in the
    /// centre, both on the edge and the one in the corner. A diagonal is held
    /// by a stone of `color`, or by an empty point that is itself such an eye
    /// (two eyes sharing a diagonal keep each other true). An enemy stone or
    /// a plain empty point on a diagonal is not held, so a false eye, or a
    /// point that is not an eye yet, may be filled.
    pub fn check_eye(&self, Stone { coord, color }: Stone) -> bool {
        self.goban.get_color(coord).is_none()
            && self.cross_owned(coord, color)
            && self.diagonals_held(coord, color, None)
    }

    /// Detects false eyes: an empty point with every neighbour of `color`
    /// that does not hold enough diagonals to be a true eye (see
    /// [`Game::check_eye`]). Filling it connects the neighbours but never
    /// creates an eye, so a playout should not bother.
    pub fn check_false_eye(&self, Stone { coord, color }: Stone) -> bool {
        self.goban.get_color(coord).is_none()
            && self.cross_owned(coord, color)
            && !self.diagonals_held(coord, color, None)
    }

    /// Every neighbour of the point carries a stone of `color`.
    fn cross_owned(&self, coord: Coord, color: Color) -> bool {
        !self
            .goban
            .get_connected_points(coord)
            .any(|s| s.color != Some(color))
    }

    /// Enough diagonals of `coord` are held for a true eye. `supporter` is
    /// the eye that asked, treated as held; when it is `None`, an empty
    /// diagonal may count as held if it is itself an eye supported by `coord`.
    fn diagonals_held(&self, coord: Coord, color: Color, supporter: Option<Coord>) -> bool {
        let mut held = 0;
        let mut off_board = 0;
        for corner in corner_points(coord) {
            if !is_coord_valid(self.goban.size(), corner) {
                off_board += 1;
            } else if Some(corner) == supporter
                || self.goban.get_color(corner) == Some(color)
                || (supporter.is_none()
                    && self.goban.get_color(corner).is_none()
                    && self.cross_owned(corner, color)
                    && self.diagonals_held(corner, color, Some(coord)))
            {
                held += 1;
            }
        }
        if off_board > 0 {
            held + off_board == 4
        } else {
            held >= 3
        }
    }

    /// Test if a play is ko.
    /// If the goban is in the configuration of two plays ago returns true
    /// If the move captures more than 1 stone, then it's not a ko.
    pub fn check_ko(&self, stone: Stone) -> bool {
        self.ko_point == Some(stone.coord)
            && !self
                .goban
                .get_connected_groups(stone.coord)
                .into_iter()
                .any(|neighbor_go_string| {
                    neighbor_go_string.is_atari() && neighbor_go_string.num_stones > 1
                })
    }

    /// Rule of the super Ko, if any before configuration was already played, then return true.
    ///
    /// Only moves that remove stones can repeat a position: captures, and
    /// suicides under rules that allow them. A suicide that captures nothing
    /// and removes only the stone just played recreates the current position,
    /// so the board before the move is checked as well as the history.
    pub fn check_super_ko(&self, stone: Stone) -> bool {
        self.check_super_ko_by(stone, true)
    }

    /// Super ko check where `suicide_possible` says whether a suicide could
    /// reach this point. Rules that forbid suicide reject those moves before
    /// checking super ko, so `check_point_by` passes `false` and skips the
    /// suicide detection.
    fn check_super_ko_by(&self, stone: Stone, suicide_possible: bool) -> bool {
        if self.will_capture(stone.coord) {
            if self.check_ko(stone) {
                return true;
            }
            if self.history.len() <= 2 {
                // Too few positions for a capture to repeat one.
                return false;
            }
        } else {
            match if suicide_possible {
                self.suicide_kind(stone)
            } else {
                SuicideKind::NotSuicide
            } {
                // Adds a stone and removes none: the position cannot be a repeat.
                SuicideKind::NotSuicide => return false,
                // Only the played stone dies: the position is the current one.
                SuicideKind::LoneStone => return true,
                // A friendly chain dies too: the position changes, check history.
                SuicideKind::Chain => {}
            }
        }
        let to_check = self.play_for_verification(stone.coord);
        self.history.contains(&to_check.snapshot())
    }

    /// Add a stone to the board and then test if the stone or stone group is dead.
    /// Returns true if the move is a suicide
    pub fn check_suicide(&self, stone: Stone) -> bool {
        self.suicide_kind(stone) != SuicideKind::NotSuicide
    }

    /// Whether the move is a suicide, and if so whether only the played stone
    /// dies or a friendly chain with it.
    fn suicide_kind(&self, stone: Stone) -> SuicideKind {
        if self.goban.has_liberties(stone.coord) {
            return SuicideKind::NotSuicide;
        }
        let mut kind = SuicideKind::LoneStone;
        for neighbor_go_string in self.goban.get_connected_groups(stone.coord) {
            if neighbor_go_string.color == stone.color {
                if !neighbor_go_string.is_atari() {
                    // Connecting with another string which is not in danger
                    return SuicideKind::NotSuicide;
                }
                kind = SuicideKind::Chain;
            } else if neighbor_go_string.is_atari() {
                // Capture move
                return SuicideKind::NotSuicide;
            }
        }
        kind
    }

    /// Displays the internal board.
    pub fn display_goban(&self) {
        println!("{}", self.goban)
    }

    #[inline]
    fn remove_captured_stones(&mut self, dead_chains: &[GroupIdx], added_chain: GroupIdx) {
        let ((black_prisoners, white_prisoners), ko_point) = self.goban.remove_captured_stones_aux(
            !self.rule.flag_illegal.contains(IllegalRules::SUICIDE),
            dead_chains,
            added_chain,
        );
        self.prisoners.0 += black_prisoners;
        self.prisoners.1 += white_prisoners;
        self.ko_point = ko_point;
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new(GobanSizes::Nineteen, CHINESE)
    }
}
