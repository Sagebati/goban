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
        self.next = self.next
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
    use std::array::IntoIter;

    use crate::pieces::Nat;

    /// Defining the policy for the colums.
    pub type Point = (Nat, Nat);

    /// Return true if the coord is in the goban.
    #[inline]
    pub const fn is_coord_valid((height, width): (usize, usize), coord: Point) -> bool {
        coord.0 < height as u8 && coord.1 < width as u8
    }

    #[inline(always)]
    pub const fn two_to_1dim(size: (usize, usize), point: Point) -> usize {
        point.0 as usize * size.0 as usize + point.1 as usize
    }

    #[inline(always)]
    pub const fn one_to_2dim(size: (usize, usize), index: usize) -> (Nat, Nat) {
        ((index / size.0) as u8, (index % size.1) as u8)
    }

    #[inline]
    pub fn neighbor_points((x1, x2): Point) -> impl Iterator<Item=Point> {
        IntoIter::new([
            (x1 + 1, x2),
            (x1.wrapping_sub(1), x2),
            (x1, x2 + 1),
            (x1, x2.wrapping_sub(1)),
        ])
    }

    #[inline]
    pub fn corner_points((x1, x2): Point) -> impl Iterator<Item=Point> {
        IntoIter::new([
            (x1 + 1, x2 + 1),
            (x1.wrapping_sub(1), x2.wrapping_sub(1)),
            (x1 + 1, x2.wrapping_sub(1)),
            (x1.wrapping_sub(1), x2 + 1),
        ])
    }
}
