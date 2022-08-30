use bitvec::prelude::*;

use crate::pieces::BoardIdx;
use crate::pieces::stones::Color;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain {
    pub color: Color,
    pub origin: u16,
    pub last: u16,
    pub liberties: BitBox,
    pub used: bool,
    pub num_stones: u16,
}

impl Chain {
    #[inline]
    pub fn new(color: Color, stone: BoardIdx) -> Self {
        Self::new_with_liberties(color, stone, bitvec![0,361].into_boxed_bitslice())
    }

    pub fn new_with_liberties(color: Color, stone: BoardIdx, liberties: BitBox) -> Self {
        Chain {
            color,
            origin: stone as u16,
            last: stone as u16,
            liberties,
            used: true,
            num_stones: 1,
        }
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        !self.liberties.any()
    }

    #[inline]
    pub fn number_of_liberties(&self) -> usize {
        self.liberties.count_ones()
    }

    /// A go string is atari if it only has one liberty
    #[inline]
    pub fn is_atari(&self) -> bool {
        self.liberties.count_ones() == 1
    }

    #[inline]
    pub fn contains_liberty(&self, stone_idx: BoardIdx) -> bool {
        self.liberties[stone_idx]
    }

    #[inline]
    pub fn remove_liberty(&mut self, stone_idx: BoardIdx) -> &mut Self {
        debug_assert!(
            self.liberties[stone_idx],
            "Tried to remove a liberty, who isn't present. stone idx: {}",
            stone_idx
        );
        self.liberties.set(stone_idx, false);
        self
    }

    #[inline]
    pub fn add_liberty(&mut self, stone_idx: BoardIdx) -> &mut Self {
        debug_assert!(
            !self.liberties[stone_idx],
            "Tried to add a liberty already present, stone idx: {}",
            stone_idx
        );
        self.liberties.set(stone_idx, true);
        self
    }

    #[inline]
    pub fn add_liberties(&mut self, stones_idx: impl Iterator<Item=BoardIdx>) -> &mut Self {
        for idx in stones_idx {
            self.add_liberty(idx);
        }
        self
    }

    #[inline]
    pub fn add_liberties_owned(&mut self, liberties_idx: BitBox) -> &mut Self {
        self.liberties |= liberties_idx;
        self
    }

    pub fn add_liberties_slice(&mut self, stones_idx: &[BoardIdx]) -> &mut Self {
        for &idx in stones_idx {
            self.liberties.set(idx, true);
        }
        self
    }

    pub fn liberties(&self) -> Vec<usize> {
        let mut v = vec![];
        for i in self.liberties.iter_ones() {
            v.push(i);
        }
        v
    }
}
