use bitvec::BitArr;

use crate::pieces::{BoardIdx, Nat};
use crate::pieces::stones::Color;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Chain {
    pub color: Color,
    pub origin: BoardIdx,
    pub last: BoardIdx,
    pub liberties: BitArr!(for 361),
    pub used: bool,
    pub num_stones: u16,
}

impl Chain {
    #[inline]
    pub fn new(color: Color, stone: BoardIdx) -> Self {
        Self::new_with_liberties(color, stone, Default::default())
    }

    pub fn new_with_liberties(color: Color, stone: BoardIdx, liberties: BitArr!(for 361)) -> Self {
        Chain {
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
    pub fn add_liberties_owned(&mut self, stones_idx: BitArr!(for 361)) -> &mut Self {
        self.liberties |= stones_idx;
        self
    }

    pub fn add_liberties_slice(&mut self, stones_idx: &[BoardIdx]) -> &mut Self {
        for &idx in stones_idx {
            self.liberties.set(idx, true);
        }
        self
    }
}
