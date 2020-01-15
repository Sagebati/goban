//! Module with the logic for calculating coordinates.

pub mod coord {
    use arrayvec::ArrayVec;

    /// Defining the policy for the colums.
    pub type Point = (usize, usize);

    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    pub enum Order {
        RowMajor,
        ColumnMajor,
    }

    /// Waiting for const numeric.
    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    pub struct CoordUtil {
        n_rows: usize,
        n_cols: usize,
        order: Order,
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
    pub fn corner_points((x1, x2): Point) -> ArrayVec<[Point; 4]> {
        ArrayVec::from([
            (x1 + 1, x2 + 1),
            (x1.wrapping_sub(1), x2.wrapping_sub(1)),
            (x1 + 1, x2.wrapping_sub(1)),
            (x1.wrapping_sub(1), x2 + 1),
        ])
    }

    impl CoordUtil {
        pub fn new(n_rows: usize, n_cols: usize) -> CoordUtil {
            CoordUtil {
                n_cols,
                n_rows,
                order: Order::RowMajor,
            }
        }
        pub fn new_order(n_rows: usize, n_cols: usize, order: Order) -> CoordUtil {
            CoordUtil {
                n_rows,
                n_cols,
                order,
            }
        }

        #[inline]
        pub fn to(self, coord: Point) -> usize {
            match self.order {
                Order::ColumnMajor => (coord.0 * self.n_cols + coord.1 % self.n_rows),
                Order::RowMajor => (coord.0 * self.n_rows + coord.1 % self.n_cols),
            }
        }

        #[inline]
        pub fn from(self, index: usize) -> Point {
            match self.order {
                Order::ColumnMajor => (index / self.n_cols, index % self.n_rows),
                Order::RowMajor => (index / self.n_rows, index % self.n_cols),
            }
        }
    }
}
