//! # Example
//! ## Get legals moves and plays some random.
//! ```
//!use crate::goban::rules::*;
//!use crate::goban::rules::game::*;
//!use rand::seq::IteratorRandom;
//! use goban::rules::game_builder::GameBuilder;
//!
//!let mut g = GameBuilder::default()
//!    .size((19,19))
//!    .rule(Rule::Chinese)
//!//  .komi(7.5)  Komi is hardcoded for each rule, but can be override like this.
//!    .build().unwrap();
//!let mut i = 35;
//!while !g.is_over() && i != 0 {
//!       g.play(
//!          // legals return an iterator of (x,y) points (lazy)
//!          g.legals()
//!                .choose(&mut rand::thread_rng())
//!                .map(|point| Move::Play(point.0,point.1))
//!                .unwrap());
//!        i -= 1;
//!        g.display_goban();
//!}
//!
//! ```

#[macro_use]
extern crate getset;
#[macro_use]
extern crate lazy_static;

pub mod pieces;
pub mod rules;
