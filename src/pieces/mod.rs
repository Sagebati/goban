//!
//! Module containing all the necessary for playing Go.
//! The goban structure. The stone structure.
//!
pub(super) type Nat = u8;
pub(super) type BoardIdx = usize;

pub mod chain;
pub mod goban;
pub mod stones;
pub mod territory;
pub mod util;
pub mod zobrist;
