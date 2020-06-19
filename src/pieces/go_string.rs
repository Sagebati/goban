use crate::pieces::stones::Color;
use crate::pieces::util::coord::{Point, CoordUtil};
use crate::pieces::{BitGoban};


#[derive(Clone, Debug, PartialEq, Getters, Eq)]
pub struct GoString {
    pub color: Color,
    pub(super) stones: BitGoban,
    #[get = "pub"]
    pub(super) liberties: BitGoban,
}

impl GoString {
    #[inline]
    pub fn is_dead(&self) -> bool {
        !self.liberties.any()
    }

    #[inline]
    pub fn number_of_liberties(&self) -> usize {
        self.liberties.count_ones()
    }

    #[inline]
    pub fn number_of_stones(&self) -> usize {
        self.stones.count_ones()
    }

    #[inline]
    pub fn is_atari(&self) -> bool {
        self.number_of_liberties() == 1
    }

    #[inline]
    pub fn contains_stone(&self, index:usize) -> bool {
        self.stones[index]
    }

    #[inline]
    pub fn contains_liberty(&self, index: usize) -> bool {
        self.liberties[index]
    }

    #[inline]
    pub fn without_liberty(&self, index: usize) -> GoString {
        debug_assert!(self.contains_liberty(index));
        let mut new = self.clone();
        new.liberties.set(index, false);
        new
    }

    #[inline]
    pub fn with_liberty(&self, index: usize) -> GoString {
        debug_assert!(!self.contains_liberty(index));
        let mut new = self.clone();
        new.liberties.set(index, true);
        new
    }

    pub fn stones(&self, coord_util: CoordUtil) -> impl Iterator<Item=Point> + '_ {
        self.stones.iter().enumerate()
            .filter(|&(_idx, it)| *it)
            .map(move |(idx, _)| coord_util.from(idx))
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
        }: GoString,
    ) -> Self {
        assert_eq!(
            self.color, color,
            "When merging two strings, the 2  go strings need to be of \
             same color. Colors found {} and {}",
            self.color, color
        );
        self.stones |= stones;

        self.liberties |= liberties;
        self.liberties &= !self.stones.clone();
        self
    }
}
