//! Dead-stone estimation by playouts.
//!
//! Chains that do not have two eyes are the candidates. The position is played
//! out to the end many times with a light random policy, and a candidate is
//! reported dead only when it has been removed from the board in more than
//! [`DEAD_THRESHOLD_PERCENT`] percent of those final boards. Anything less
//! stays "floating": the estimator could not decide, so the chain is kept.

use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{HashMap, HashSet};

use crate::pieces::goban::GroupIdx;
use crate::pieces::stones::{Color, Stone};
use crate::pieces::util::coord::Coord;
use crate::rules::game::Game;
use crate::rules::Move;

/// Share of the final boards a chain must be missing from to be called dead.
pub const DEAD_THRESHOLD_PERCENT: usize = 98;

/// Number of playouts used by [`Game::dead_stones`].
pub const DEFAULT_PLAYOUTS: usize = 200;

impl Game {
    /// Returns the chains that do not have two eyes, that is the candidates
    /// for being dead. A chain touching two single-point eyes is assumed alive.
    pub fn get_floating_stones(&self) -> Vec<GroupIdx> {
        let eyes = self.pseudo_legals().filter(|&p| {
            self.check_eye(Stone {
                coord: p,
                color: Color::Black,
            }) || self.check_eye(Stone {
                coord: p,
                color: Color::White,
            })
        });
        let mut eyes_per_chain: HashMap<GroupIdx, usize> = HashMap::new();
        for eye_coord in eyes {
            for chain_idx in self.goban.get_connected_groups_idx(eye_coord) {
                *eyes_per_chain.entry(chain_idx).or_insert(0) += 1;
            }
        }

        self.goban
            .chains_with_index()
            .filter(|(idx, _chain)| eyes_per_chain.get(idx).copied().unwrap_or(0) < 2)
            .map(|(idx, _chain)| idx)
            .collect()
    }

    /// Chains estimated dead after `nb_playouts` random playouts: a chain
    /// without two eyes that has been captured in more than
    /// [`DEAD_THRESHOLD_PERCENT`] percent of the final boards.
    ///
    /// The playouts start from this position with the passes and the outcome
    /// cleared, so a game that ended by two passes or that carries a result is
    /// still played out.
    pub fn dead_stones_wth_simulations(&self, nb_playouts: usize) -> HashSet<GroupIdx> {
        let floating_stones = self.get_floating_stones();
        if floating_stones.is_empty() || nb_playouts == 0 {
            return HashSet::new();
        }
        let chain_stones: Vec<(GroupIdx, Vec<Stone>)> = floating_stones
            .into_iter()
            .map(|idx| (idx, self.chain_stones(idx).collect()))
            .collect();

        let mut rng = rand::rng();
        let mut removed = vec![0usize; chain_stones.len()];
        for _ in 0..nb_playouts {
            let final_goban = self.playout(&mut rng);
            for (count, (_idx, stones)) in removed.iter_mut().zip(&chain_stones) {
                if stones
                    .iter()
                    .any(|s| final_goban.get_color(s.coord) != Some(s.color))
                {
                    *count += 1;
                }
            }
        }

        chain_stones
            .into_iter()
            .zip(removed)
            .filter(|(_, count)| count * 100 > DEAD_THRESHOLD_PERCENT * nb_playouts)
            .map(|((idx, _), _)| idx)
            .collect()
    }

    /// Return an array of dead stones, works better if the game if ended.
    /// the "dead" stones are only potentially dead.
    #[inline]
    pub fn dead_stones(&self) -> HashSet<GroupIdx> {
        self.dead_stones_wth_simulations(DEFAULT_PLAYOUTS)
    }

    /// Play the position to the end with [`Game::playout_move`] and return
    /// the final board.
    fn playout(&self, rng: &mut impl Rng) -> crate::pieces::goban::Goban {
        let mut game = self.clone();
        game.passes = 0;
        game.outcome = None;
        let (height, width) = game.size();
        // Guard against endless cycles under rules without super ko.
        let max_moves = 4 * height as usize * width as usize;
        let mut moves = 0;
        while !game.is_over() && moves < max_moves {
            game.play(game.playout_move(rng));
            moves += 1;
        }
        game.goban
    }

    /// Light playout policy, in order: capture when possible; extend from a
    /// cramped stone of ours (one with a single empty neighbour) onto a point
    /// that keeps at least two empty neighbours; otherwise a random legal
    /// move that is neither a true nor a false eye of ours nor a self-atari.
    /// Never a suicide. Pass when nothing is left.
    ///
    /// Measured on open positions, always rescuing a chain in atari makes a
    /// dead invader run and build a group that a random defender fails to
    /// kill, so the extension rule is deliberately local rather than a
    /// proper "save the chain" rule.
    fn playout_move(&self, rng: &mut impl Rng) -> Move {
        let me = self.turn;
        let mut legal: Vec<Coord> = self
            .legals()
            .filter(|&coord| !self.check_suicide(Stone { coord, color: me }))
            .collect();
        if legal.is_empty() {
            return Move::Pass;
        }
        legal.shuffle(rng);

        if let Some(&coord) = legal.iter().find(|&&c| self.will_capture(c)) {
            return coord.into();
        }

        let goban = &self.goban;
        if let Some(&coord) = legal.iter().find(|&&c| {
            goban
                .get_connected_points(c)
                .any(|p| p.color == Some(me) && goban.get_liberties(p.coord).count() == 1)
                && self.play_for_verification(c).get_liberties(c).count() > 1
        }) {
            return coord.into();
        }

        legal
            .into_iter()
            .find(|&coord| {
                let stone = Stone { coord, color: me };
                !self.check_eye(stone)
                    && !self.check_false_eye(stone)
                    && self.play_for_verification(coord).chain_liberties_at(coord) > 1
            })
            .map(Move::from)
            .unwrap_or(Move::Pass)
    }
}
