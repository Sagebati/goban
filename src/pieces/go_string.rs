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

    #[inline]
    pub fn is_dead(&self) -> bool {
        self.liberties.is_empty()
    }

    #[inline]
    pub fn number_of_liberties(&self) -> usize {
        self.liberties.len()
    }

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
    pub fn remove_liberty(&mut self, stone_idx: usize) {
        debug_assert!(self.liberties.contains(&stone_idx));
        self.liberties.remove(&stone_idx);
    }

    #[inline]
    pub fn without_liberty(&self, point: usize) -> GoString {
        let mut new = self.clone();
        new.remove_liberty(point);
        new
    }

    #[inline]
    pub fn add_liberty(&mut self, stone_idx: usize) {
        debug_assert!(!self.liberties.contains(&stone_idx));
        self.liberties.insert(stone_idx);
    }

    #[inline]
    pub fn add_liberties(&mut self, stones_idx: impl Iterator<Item = usize>) {
        for idx in stones_idx {
            self.add_liberty(idx);
        }
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

    ///
    /// Takes ownership of self and the other string then merge into one string
    ///
    #[inline]
    pub fn merge_with(
        mut self,
        GoString {
            color,
            stones,
            liberties,
        }: &GoString,
    ) -> Self {
        assert_eq!(
            self.color, *color,
            "When merging two strings, the 2  go strings need to be of \
             same color. Colors found {} and {}",
            self.color, *color
        );
        self.stones.extend(stones);
        self.liberties.extend(liberties);

        self
    }
}
