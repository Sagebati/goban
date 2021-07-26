use bitvec::bitvec;
use bitvec::macros::internal::core::ops::BitOrAssign;
use bitvec::vec::BitVec;

use crate::pieces::stones::Color;

type SetIdx = BitVec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoString {
    pub color: Color,
    pub origin: usize,
    pub last: usize,
    pub liberties: SetIdx,
    pub used: bool,
    pub num_stones: u16,
}

impl GoString {
    #[inline]
    pub fn new(color: Color, stone: usize) -> Self {
        Self::new_with_liberties(color, stone, bitvec![0;361])
    }

    pub fn new_with_liberties(color: Color, stone: usize, liberties: SetIdx) -> Self {
        GoString {
            color,
            origin: stone,
            last: stone,
            liberties,
            used: true,
            num_stones: 1,
        }
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        self.liberties.not_any()
    }

    #[inline]
    pub fn number_of_liberties(&self) -> usize {
        self.liberties.count_ones()
    }

    /// A go string is atari if it only has one liberty
    #[inline]
    pub fn is_atari(&self) -> bool {
        self.number_of_liberties() == 1
    }

    #[inline]
    pub fn contains_liberty(&self, stone_idx: usize) -> bool {
        self.liberties[stone_idx]
    }

    #[inline]
    pub fn remove_liberty(&mut self, stone_idx: usize) -> &mut Self {
        debug_assert!(self.liberties[stone_idx], "Tried to remove a liberty, who isn't present. stone idx: {}", stone_idx);
        self.liberties.set(stone_idx, false);
        self
    }

    #[inline]
    pub fn add_liberty(&mut self, stone_idx: usize) -> &mut Self {
        debug_assert!(!self.liberties[stone_idx], "Tried to add a liberty already present, stone idx: {}", stone_idx);
        self.liberties.set(stone_idx, true);
        self
    }

    #[inline]
    pub fn add_liberties(&mut self, stones_idx: impl Iterator<Item=usize>) -> &mut Self {
        for idx in stones_idx {
            self.add_liberty(idx);
        }
        self
    }

    #[inline]
    pub fn add_liberties_owned(&mut self, stones_idx: SetIdx) -> &mut Self {
        self.liberties.bitor_assign(stones_idx);
        self
    }
}
