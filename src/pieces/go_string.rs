use crate::pieces::Set;
use crate::pieces::stones::Color;

type SetIdx = Set<usize>;

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
        Self::new_with_liberties(color, stone, Default::default())
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
        self.liberties.is_empty()
    }

    #[inline]
    pub fn number_of_liberties(&self) -> usize {
        self.liberties.len()
    }

    /// A go string is atari if it only has one liberty
    #[inline]
    pub fn is_atari(&self) -> bool {
        self.liberties.len() == 1
    }

    #[inline]
    pub fn contains_liberty(&self, point: usize) -> bool {
        self.liberties.contains(&point)
    }

    #[inline]
    pub fn remove_liberty(&mut self, stone_idx: usize) -> &mut Self {
        #[cfg(debug_assertions)]
        if !self.liberties.contains(&stone_idx) {
            panic!("Tried to remove a liberty who doesn't exist")
        }
        debug_assert!(self.liberties.contains(&stone_idx));
        self.liberties.remove(&stone_idx);
        self
    }

    #[inline]
    pub fn add_liberty(&mut self, stone_idx: usize) -> &mut Self {
        debug_assert!(!self.liberties.contains(&stone_idx));
        self.liberties.insert(stone_idx);
        self
    }

    #[inline]
    pub fn add_liberties(&mut self, stones_idx: impl Iterator<Item = usize>) -> &mut Self {
        for idx in stones_idx {
            self.add_liberty(idx);
        }
        self
    }

    #[inline]
    pub fn add_liberties_owned(&mut self, stones_idx: SetIdx) -> &mut Self {
        self.liberties.extend(stones_idx);
        self
    }
}
