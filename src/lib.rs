//! # Example
//! ## Get legals moves and plays some random.
//! ```
//!     use crate::goban::rules::*;
//!     use crate::goban::rules::game::*;
//!     use rand::seq::IteratorRandom;
//!
//!     let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
//!       let mut i = 35;
//!        while !g.is_over() && i != 0 {
//!            g.play(
//!                 // legals return an iterator on (x,y) points
//!                g.legals()
//!                    .choose(&mut rand::thread_rng())
//!                    .map(|point| Move::Play(point.0,point.1))
//!                     .unwrap());
//!            i -= 1;
//!            g.display_goban();
//!        }
//! ```

#[macro_use]
extern crate getset;
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate maplit;

pub mod pieces;
pub mod rules;
