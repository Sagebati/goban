use crate::pieces::Set;
use crate::pieces::stones::Color;

type SetBoardIdx = Set<usize>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain {
    pub color: Color,
    pub origin: usize,
    pub last: usize,
    pub liberties: SetBoardIdx,
    pub used: bool,
    pub num_stones: u16,
}

impl Chain {
    #[inline]
    pub fn new(color: Color, stone: usize) -> Self {
        Self::new_with_liberties(color, stone, Default::default())
    }

    pub fn new_with_liberties(color: Color, stone: usize, liberties: SetBoardIdx) -> Self {
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
        self.liberties.is_empty()
    }

    #[inline]
    pub fn number_of_liberties(&self) -> usize {
        self.liberties.len()
    }

    /// A go string is atari if it only has one liberty
    #[inline]
    pub fn is_atari(&self) -> bool {
        self.number_of_liberties() == 1
    }

    #[inline]
    pub fn contains_liberty(&self, stone_idx: usize) -> bool {
        self.liberties.contains(&stone_idx)
    }

    #[inline]
    pub fn remove_liberty(&mut self, stone_idx: usize) -> &mut Self {
        debug_assert!(
            self.liberties.contains(&stone_idx),
            "Tried to remove a liberty, who isn't present. stone idx: {}",
            stone_idx
        );
        self.liberties.remove(&stone_idx);
        self
    }

    #[inline]
    pub fn add_liberty(&mut self, stone_idx: usize) -> &mut Self {
        debug_assert!(
            !self.liberties.contains(&stone_idx),
            "Tried to add a liberty already present, stone idx: {}",
            stone_idx
        );
        self.liberties.insert(stone_idx);
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
    pub fn add_liberties_owned(&mut self, stones_idx: SetBoardIdx) -> &mut Self {
        self.liberties.extend(stones_idx);
        self
    }
}
