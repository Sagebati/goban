//!
//! Module containing all the necessary for playing Go.
//! The goban structure. The stone structure.
//!
use ahash::AHashSet;

/// ByAddress is needed for equality/hash of pointer by address the hashmap.
type Set<T> = AHashSet<T>;

pub(super) type Nat = u8;

pub mod chain;
pub mod goban;
pub mod stones;
pub mod territory;
pub mod util;
pub mod zobrist;
