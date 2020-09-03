use crate::pieces::Set;
use crate::pieces::stones::Color;

type SetPoints = Set<usize>;

#[derive(Clone, Debug, PartialEq, Getters, Eq)]
pub struct GoString {
    pub color: Color,
    #[get = "pub"]
    pub(crate) stones: SetPoints,
    #[get = "pub"]
    pub(super) liberties: SetPoints,
}

impl GoString {
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
    pub fn without_liberty(&self, point: usize) -> GoString {
        debug_assert!(self.liberties.contains(&point));
        let mut new = self.clone();
        new.liberties.remove(&point);
        new
    }

    #[inline]
    pub fn with_liberty(&self, point: usize) -> GoString {
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
            mut stones,
            mut liberties,
        }: GoString,
    ) -> Self {
        assert_eq!(
            self.color, color,
            "When merging two strings, the 2  go strings need to be of \
             same color. Colors found {} and {}",
            self.color, color
        );
        let new_stones = if stones.len() < self.stones.len() {
            self.stones.extend(stones);
            self.stones
        } else {
            stones.extend(self.stones);
            stones
        };

        let mut new_liberties = if liberties.len() < self.liberties.len() {
            self.liberties.extend(liberties);
            self.liberties
        } else {
            liberties.extend(self.liberties);
            liberties
        };
        new_liberties = new_liberties.difference(&new_stones).copied().collect();

        GoString {
            color,
            stones: new_stones,
            liberties: new_liberties,
        }
    }
}
