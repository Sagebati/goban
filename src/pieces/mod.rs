//!
//! Module containing all the necessary for playing Go.
//! The goban structure. The stone structure.
//!
use arrayvec::ArrayVec;

pub(super) type Nat = u8;
pub(super) type BoardIdx = usize;

pub(super) type Connections<T=BoardIdx> = ArrayVec<T, 4>;

pub mod group;
pub mod goban;
pub mod stones;
pub mod territory;
pub mod util;
pub mod zobrist;
