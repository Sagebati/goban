//! # Example
//! ## Get legals moves and plays some random.
//!
//! The most important struct is Game who has all you need to create and manages go games.
//!
//! ```
//!use crate::goban::rules::*;
//!use crate::goban::rules::game::*;
//!use rand::seq::IteratorRandom;
//! use goban::rules::game_builder::GameBuilder;
//!
//!let mut g = Game::builder()
//!    .size((19,19))
//!    .rule(Rule::Chinese)
//!//  .komi(7.5)  Komi is hardcoded for each rule, but can be override like this.
//!    .build().unwrap();
//!
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
//!        // None if the game is not finished
//!        println!("{:?}", g.outcome());
//!        // Inner array using row policy
//!        println!("{:?}", g.goban().raw());
//!
//!
//!}
//!#[cfg(feature = "history")]
//!{
//!            let mut iter_history = g.history().iter();
//!            println!("{:?}", iter_history.next().unwrap());
//!            println!("{:?}", iter_history.next_back().unwrap())
//!}
//!
//! ```

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate getset;

pub mod pieces;
pub mod rules;
