#[macro_use]
extern crate getset;

//!
//! Library for Go playing and ruling.
//!
//!
//! # Exemple
//! ## Get legals moves and plays some random.
//! ```
//!     let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
//        let mut i = 35;
//        while !g.legals().count() != 0 && i != 0 {
//            g.play(
//                &g.legals().map(|coord| Move::Play(coord.0, coord.1))
//                    .choose(&mut rand::thread_rng())
//                    .unwrap());
//            i -= 1;
//            println!("{}", g.goban().pretty_string());
//        }
//! ```


pub mod pieces;
pub mod rules;


