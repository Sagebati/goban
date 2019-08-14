//! # Example
//! ## Get legals moves and plays some random.
//! ```
//!     use crate::goban::rules::*;
//!     use crate::goban::rules::game::*;
//!     use rand::seq::IteratorRandom;
//!
//!     let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
//!       let mut i = 35;
//!        while !g.legals().count() != 0 && i != 0 {
//!            g.play(
//!                &g.legals().map(|coord| Move::Play(coord.0, coord.1))
//!                    .choose(&mut rand::thread_rng())
//!                    .unwrap());
//!            i -= 1;
//!            println!("{}", g.goban().pretty_string());
//!        }
//! ```

#[macro_use]
extern crate getset;
#[macro_use]
extern crate lazy_static;

pub mod pieces;
pub mod rules;
