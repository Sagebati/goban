//!
//! Module containing all the necessary for playing Go.
//! The goban structure. The stone structure.
//!

#[cfg(not(feature = "thread-safe"))]
use std::rc::Rc;
#[cfg(feature = "thread-safe")]
use std::sync::Arc;

use ahash::AHashSet;
use by_address::ByAddress;

use crate::pieces::go_string::GoString;

#[cfg(not(feature = "thread-safe"))]
type Ptr<T> = Rc<T>;

#[cfg(feature = "thread-safe")]
type Ptr<T> = Arc<T>;

/// The go string pointer.
/// ByAddress is needed for equality/hash of pointer by address the hashmap.
pub(crate) type GoStringPtr = ByAddress<Ptr<GoString>>;
type Set<T> = AHashSet<T>;

pub(super) type Nat = u8;

pub mod go_string;
pub mod goban;
pub mod neighbor;
pub mod stones;
pub mod territory;
pub mod util;
pub mod zobrist;
