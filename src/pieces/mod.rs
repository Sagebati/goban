//!
//! Module containing all the necessary for playing Go.
//! The goban structure. The stone structure.
//!

#[cfg(not(feature = "thread-safe"))]
use std::rc::Rc;

#[cfg(feature = "thread-safe")]
use std::sync::Arc;

#[cfg(not(feature = "thread-safe"))]
type Ptr<T> = Rc<T>;

#[cfg(feature = "thread-safe")]
type Ptr<T> = Arc<T>;

use crate::pieces::go_string::GoString;
use by_address::ByAddress;
use std::collections::HashSet;

/// The go string pointer.
/// ByAddress is needed for equality/hash of pointer by address the hashmap.
type GoStringPtr = ByAddress<Ptr<GoString>>;
type Set<T> = HashSet<T>;
#[allow(non_camel_case_types)]
pub type uint = u16;

pub mod go_string;
pub mod goban;
pub mod stones;
pub mod territory;
pub mod util;
pub mod zobrist;
