//! Module with the logic for calculating coordinates.

pub mod coord {
    use arrayvec::ArrayVec;

    use crate::pieces::Nat;

    /// Defining the policy for the colums.
    pub type Point = (Nat, Nat);

    /// Return true if the coord is in the goban.
    #[inline]
    pub fn is_coord_valid((height, width): (usize, usize), coord: Point) -> bool {
        coord.0 < height as u8 && coord.1 < width as u8
    }

    #[inline(always)]
    pub fn two_to_1dim(size: (usize, usize), point: Point) -> usize {
        point.0 as usize * size.0 as usize + point.1 as usize
    }

    #[inline(always)]
    pub fn one_to_2dim(size: (usize, usize), point: usize) -> (Nat, Nat) {
        ((point / size.0) as u8, (point % size.1) as u8)
    }

    #[inline]
    pub fn neighbor_points((x1, x2): Point) -> ArrayVec<[Point; 4]> {
        ArrayVec::from([
            (x1 + 1, x2),
            (x1.wrapping_sub(1), x2),
            (x1, x2 + 1),
            (x1, x2.wrapping_sub(1)),
        ])
    }

    #[inline]
    pub fn neighbor_points_index(size: (usize, usize), index: usize) -> ArrayVec<[usize; 4]> {
        let (x1, x2) = one_to_2dim(size, index);
        ArrayVec::from([
            two_to_1dim(size, (x1 + 1, x2)),
            two_to_1dim(size, (x1.wrapping_sub(1), x2)),
            two_to_1dim(size, (x1, x2 + 1)),
            two_to_1dim(size, (x1, x2.wrapping_sub(1)))
        ])
    }

    pub fn corner_points((x1, x2): Point) -> ArrayVec<[Point; 4]> {
        ArrayVec::from([
            (x1 + 1, x2 + 1),
            (x1.wrapping_sub(1), x2.wrapping_sub(1)),
            (x1 + 1, x2.wrapping_sub(1)),
            (x1.wrapping_sub(1), x2 + 1),
        ])
    }
}
