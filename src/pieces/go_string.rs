use crate::pieces::stones::Color;
use crate::pieces::util::coord::Point;
use crate::pieces::Set;

type SetPoints = Set<Point>;

#[derive(Clone, Debug, PartialEq, Getters, Eq)]
pub struct GoString {
    pub color: Color,
    #[get = "pub"]
    stones: SetPoints,
    #[get = "pub"]
    liberties: SetPoints,
}

impl GoString {
    pub fn new(color: Color, stones: SetPoints, liberties: SetPoints) -> GoString {
        GoString {
            color,
            stones,
            liberties,
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

    #[inline]
    pub fn is_atari(&self) -> bool {
        self.liberties.len() == 1
    }

    #[inline]
    pub fn contains_stone(&self, point: Point) -> bool {
        self.stones.contains(&point)
    }

    #[inline]
    pub fn contains_liberty(&self, point: Point) -> bool {
        self.liberties.contains(&point)
    }

    #[inline]
    pub fn without_liberty(&self, point: Point) -> GoString {
        debug_assert!(self.liberties.contains(&point));
        let mut new = self.clone();
        new.liberties.remove(&point);
        new
    }

    #[inline]
    pub fn with_liberty(&self, point: Point) -> GoString {
        debug_assert!(!self.liberties.contains(&point));
        let mut new = self.clone();
        new.liberties.insert(point);
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
        }: GoString,
    ) -> Self {
        assert_eq!(
            self.color, color,
            "When merging two strings, the 2  go strings need to be of \
        same color. Colors found {} and {}",
            self.color, color
        );
        self.stones.extend(stones);
        self.liberties.extend(liberties);
        self.liberties = self.liberties.difference(&self.stones).copied().collect();

        self
    }
}
