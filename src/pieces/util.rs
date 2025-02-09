use std::iter::FusedIterator;

pub struct CircularGroupIter<'a> {
    next_stone: &'a [u16],
    origin: usize,
    next: Option<usize>,
}

impl<'a> CircularGroupIter<'a> {
    pub fn new(origin: usize, next_stone: &'a [u16]) -> Self {
        Self {
            next_stone,
            origin,
            next: Some(origin),
        }
    }
}

impl Iterator for CircularGroupIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let origin = self.origin;
        let ret = self.next;
        self.next = self
            .next
            .map(|stone_idx| self.next_stone[stone_idx] as usize)
            .filter(move |&o| o != origin);

        #[cfg(debug_assertions)]
        if ret.is_some() && self.next == ret {
            panic!("infinite loop detected")
        }

        ret
    }
}

impl FusedIterator for CircularGroupIter<'_> {}

pub mod coord {
    use crate::pieces::goban::BoardIdx;
    use crate::pieces::Nat;
    use arrayvec::ArrayVec;

    /// Defining the policy for the columns.
    pub type Coord = (Nat, Nat);
    pub type Size = (u8, u8);

    /// Return true if the coord is in the goban.
    #[inline(always)]
    pub const fn is_coord_valid((height, width): Size, coord: Coord) -> bool {
        (coord.0) < height && (coord.1) < width
    }

    #[inline(always)]
    pub const fn two_to_1dim(size: Size, point: Coord) -> usize {
        (point.0 as u32 * size.0 as u32 + point.1 as u32) as usize
    }

    #[inline(always)]
    pub const fn one_to_2dim(size: Size, index: usize) -> Coord {
        (
            (index / size.0 as usize) as u8,
            (index % size.1 as usize) as u8,
        )
    }

    #[macro_export]
    macro_rules! one2dim {
        ($size: expr, $index: expr) => {
            (
                ($index / $size.0 as usize) as u8,
                ($index % $size.1 as usize) as u8,
            )
        };
    }

    #[inline(always)]
    pub const fn neighbor_coords((x1, x2): Coord) -> [Coord; 4] {
        [
            (x1 + 1, x2),
            (x1.wrapping_sub(1), x2),
            (x1, x2 + 1),
            (x1, x2.wrapping_sub(1)),
        ]
    }

    #[inline(always)]
    pub fn valid_coords((x1, x2): Coord, size: Size) -> ArrayVec<Coord, 4> {
        let mut array_vec = ArrayVec::from(neighbor_coords((x1, x2)));
        array_vec.retain(|x| is_coord_valid(size, *x));
        array_vec
    }

    #[inline(always)]
    pub const fn corner_points((x1, x2): Coord) -> [Coord; 4] {
        [
            (x1 + 1, x2 + 1),
            (x1.wrapping_sub(1), x2.wrapping_sub(1)),
            (x1 + 1, x2.wrapping_sub(1)),
            (x1.wrapping_sub(1), x2 + 1),
        ]
    }

    pub trait IntoCoord {
        fn into_coord(self, size: Size) -> Coord;
    }

    impl IntoCoord for Coord {
        fn into_coord(self, _size: Size) -> Coord {
            self
        }
    }

    impl IntoCoord for BoardIdx {
        fn into_coord(self, size: Size) -> Coord {
            one_to_2dim(size, self)
        }
    }

    pub trait IntoIdx {
        fn into_idx(self, size: Size) -> BoardIdx;
    }

    impl IntoIdx for Coord {
        fn into_idx(self, size: Size) -> BoardIdx {
            two_to_1dim(size, self)
        }
    }

    impl IntoIdx for BoardIdx {
        fn into_idx(self, _size: Size) -> BoardIdx {
            self
        }
    }
}
