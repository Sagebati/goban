//!
//! Module containing all the necessary for playing Go.
//! The goban structure. The stone structure.
//!
use arrayvec::ArrayVec;

pub(super) type Nat = u8;
pub(super) type BoardIdx = usize;

type Neighbors<T> = ArrayVec<T, 4>;

pub mod chain;
pub mod goban;
pub mod stones;
pub mod territory;
pub mod util;
pub mod zobrist;
