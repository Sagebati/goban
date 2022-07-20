pub struct CircularRenIter<'a> {
    next_stone: &'a [usize],
    origin: usize,
    next: Option<usize>,
}

impl<'a> CircularRenIter<'a> {
    pub fn new(origin: usize, next_stone: &'a [usize]) -> Self {
        Self {
            next_stone,
            origin,
            next: Some(origin),
        }
    }
}

impl<'a> Iterator for CircularRenIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let origin = self.origin;
        let ret = self.next;
        self.next = self
            .next
            .map(|stone_idx| self.next_stone[stone_idx])
            .filter(move |&o| o != origin);

        #[cfg(debug_assertions)]
        if ret.is_some() && self.next == ret {
            dbg!(self.next_stone.iter().enumerate().collect::<Vec<_>>());
            panic!("infinite loop detected")
        }
        ret
    }
}

pub mod coord {
    use arrayvec::ArrayVec;

    use crate::pieces::Nat;

    /// Defining the policy for the columns.
    pub type Coord = (Nat, Nat);
    pub type Size = (usize, usize);

    /// Return true if the coord is in the goban.
    #[inline(always)]
    pub const fn is_coord_valid((height, width): Size, coord: Coord) -> bool {
        (coord.0 as usize) < height && (coord.1 as usize) < width
    }

    #[inline(always)]
    pub const fn two_to_1dim(size: Size, point: Coord) -> usize {
        (point.0 as u32 * size.0 as u32 + point.1 as u32) as usize
    }

    #[inline(always)]
    pub const fn one_to_2dim(size: Size, index: usize) -> Coord {
        ((index / size.0) as Nat, (index % size.1) as Nat)
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
}