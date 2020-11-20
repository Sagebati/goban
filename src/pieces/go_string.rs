use crate::pieces::stones::{Color};
use bitvec::prelude::*;

#[derive(Clone, Debug, PartialEq, Getters, Eq)]
pub struct GoString {
    pub color: Color,
    pub stones: bitarr!(for 361, in Lsb0, u64),
    pub liberties: bitarr!(for 361, in Lsb0, u64),
}

impl GoString {
    #[inline]
    pub fn new_with_color(color: Color) -> Self {
        Self {
            color,
            stones: Default::default(),
            liberties: Default::default(),
        }
    }

    #[inline]
    pub fn iter_stones(&self) -> impl Iterator<Item=usize> + '_ {
        self.stones.iter()
            .enumerate()
            .filter(|(_, b)| **b)
            .map(|(i, _)| i)
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        self.liberties.not_any()
    }

    #[inline]
    pub fn count_stones(&self) -> usize {
        self.stones.count_ones()
    }

    #[inline]
    pub fn count_liberties(&self) -> usize {
        self.liberties.count_ones()
    }

    #[inline]
    pub fn is_atari(&self) -> bool {
        self.liberties.count_ones() == 1
    }

    #[inline]
    pub fn contains_stone(&self, point: usize) -> bool {
        self.stones[point]
    }

    #[inline]
    pub fn contains_liberty(&self, point: usize) -> bool {
        self.liberties[point]
    }

    #[inline]
    pub fn add_stone(&mut self, stone: usize) {
        debug_assert!(!self.stones[stone]);
        self.stones.set(stone, true);
    }

    #[inline]
    pub fn add_liberty(&mut self, point: usize) {
        debug_assert!(!self.liberties[point]);
        self.liberties.set(point, true);
    }

    #[inline]
    pub fn remove_liberty(&mut self, point: usize) {
        debug_assert!(self.liberties[point]);
        self.liberties.set(point, false);
    }

    #[inline]
    pub fn without_liberty(&self, point: usize) -> GoString {
        let mut new = self.clone();
        new.remove_liberty(point);
        new
    }

    #[inline]
    pub fn with_liberty(&self, point: usize) -> GoString {
        let mut new = self.clone();
        new.add_liberty(point);
        new
    }

    /// Takes ownership of self and the other string then merge into one string
    #[inline]
    pub fn merge_with(
        mut self,
        GoString {
            color,
            stones,
            liberties,
        }: GoString,
    ) -> Self {
        assert_eq!(
            self.color, color,
            "When merging two strings, the 2  go strings need to be of \
             same color. Colors found {} and {}",
            self.color, color
        );

        self.stones |= stones.into_iter().copied();
        self.liberties |= liberties.iter().copied();
        self.liberties &= (!self.stones).iter().copied();

        self
    }
}




