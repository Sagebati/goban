//! Module with the logic for calculating coordinates.

pub mod coord {
    use crate::pieces::uint;
    use arrayvec::ArrayVec;

    /// Defining the policy for the colums.
    pub type Point = (uint, uint);

    /// Return true if the coord is in the goban.
    #[inline]
    pub fn is_coord_valid((height, width): (uint, uint), coord: Point) -> bool {
        coord.0 < height && coord.1 < width
    }

    #[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
    pub enum Order {
        RowMajor,
        ColumnMajor,
    }

    /// Waiting for const numeric.
    #[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
    pub struct CoordUtil {
        n_rows: uint,
        n_cols: uint,
        order: Order,
    }

    pub fn neighbor_points((x1, x2): Point) -> ArrayVec<[Point; 4]> {
        ArrayVec::from([
            (x1 + 1, x2),
            (x1.wrapping_sub(1), x2),
            (x1, x2 + 1),
            (x1, x2.wrapping_sub(1)),
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

    impl CoordUtil {
        #[inline]
        pub fn new(n_rows: uint, n_cols: uint) -> CoordUtil {
            CoordUtil::new_order(n_rows, n_cols, Order::RowMajor)
        }
        #[inline]
        pub fn new_order(n_rows: uint, n_cols: uint, order: Order) -> CoordUtil {
            CoordUtil {
                n_rows,
                n_cols,
                order,
            }
        }

        #[inline(always)]
        pub fn to(self, coord: Point) -> usize {
            let coord = (coord.0 as usize, coord.1 as usize);
            (match self.order {
                Order::ColumnMajor => (coord.0 * self.n_cols as usize + coord.1 % self.n_rows as usize),
                Order::RowMajor => (coord.0 * self.n_rows as usize + coord.1 % self.n_cols as usize),
            })
        }

        #[inline(always)]
        pub fn from(self, index: usize) -> Point {
            match self.order {
                Order::ColumnMajor => (
                    (index / self.n_cols as usize) as uint,
                    (index % self.n_rows as usize) as uint,
                ),
                Order::RowMajor => (
                    (index / self.n_rows as usize) as uint,
                    (index % self.n_cols as usize) as uint,
                ),
            }
        }
    }
}
