use crate::pieces::stones::Color;
use crate::pieces::Set;

type SetIdx = Set<usize>;

#[derive(Clone, Debug, PartialEq, Getters, CopyGetters, Eq)]
pub struct GoString {
    #[getset(get_copy = "pub")]
    color: Color,
    #[getset(get = "pub")]
    stones: SetIdx,
    #[getset(get = "pub")]
    liberties: SetIdx,
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
    pub fn new_with_color_and_stone_idx(color: Color, stone: usize) -> Self {
        let mut r = GoString::new_with_color(color);
        r.stones.insert(stone);
        r
    }

    /// Reserve space in the stones set for perforamnces.
    #[inline]
    pub fn reserve_stone(&mut self, number_of_stones: usize) -> &mut Self {
        self.stones.reserve(number_of_stones);
        self
    }

    #[inline]
    pub fn reserve_liberties(&mut self, number_of_lib: usize) -> &mut Self {
        self.liberties.reserve(number_of_lib);
        self
    }

    #[inline]
    pub fn add_stone(&mut self, stone_idx: usize) {
        self.stones.insert(stone_idx);
    }

    /// Returns true if the set of liberties is empty
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
    pub fn contains_stone(&self, point: usize) -> bool {
        self.stones.contains(&point)
    }

    #[inline]
    pub fn contains_liberty(&self, point: usize) -> bool {
        self.liberties.contains(&point)
    }

    #[inline]
    pub fn remove_liberty(&mut self, stone_idx: usize) -> &mut Self {
        //debug_assert!(self.liberties.contains(&stone_idx));
        self.liberties.remove(&stone_idx);
        self
    }

    #[inline]
    pub fn without_liberty(&self, point: usize) -> GoString {
        let mut new = self.clone();
        new.remove_liberty(point);
        new
    }

    #[inline]
    pub fn add_liberty(&mut self, stone_idx: usize) -> &mut Self {
        //debug_assert!(!self.liberties.contains(&stone_idx));
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
    pub fn with_liberty(&self, stone_idx: usize) -> GoString {
        let mut new = self.clone();
        new.add_liberty(stone_idx);
        new
    }

    #[inline]
    pub fn with_liberties(&self, stones_idx: impl Iterator<Item = usize>) -> GoString {
        let mut new = self.clone();
        new.add_liberties(stones_idx);
        new
    }

    /// Merges the string passed in param to self, indeed adding their stones to our struct, and adding
    /// their liberties to our struct.
    /// The method cas produce some bugs, there can be some liberties in excess after the merge.
    #[inline]
    pub fn with_merge(mut self, other: &GoString) -> Self {
        self.merge(other);
        self
    }

    pub fn merge(&mut self, other: &GoString) -> &mut GoString {
        assert_eq!(
            self.color, other.color,
            "When merging two strings, the 2  go strings need to be of \
             same color. Colors found {} and {}",
            self.color, other.color
        );
        self.stones.extend(&other.stones);
        self.liberties.extend(&other.liberties);
        self
    }
}
