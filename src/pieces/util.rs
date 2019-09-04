//! Module with the logic for calculating coordinates.

pub mod coord {
    /// Defining the policy for the colums.

    pub type Coord = (usize, usize);

    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    pub enum Order {
        RowMajor,
        ColumnMajor,
    }

    /// Waiting for const numeric.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CoordUtil {
        n_rows: usize,
        n_cols: usize,
        order: Order,
    }

    pub fn neighbors_coords(coord: Coord) -> Vec<Coord> {
        vec![
            (coord.0.wrapping_add(1), coord.1),
            (coord.0.wrapping_sub(1), coord.1),
            (coord.0, coord.1.wrapping_add(1)),
            (coord.0, coord.1.wrapping_sub(1)),
        ]
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
        pub fn to(&self, coord: Coord) -> usize {
            match self.order {
                Order::ColumnMajor => (coord.0 * self.n_cols + coord.1 % self.n_rows),
                Order::RowMajor => (coord.0 * self.n_rows + coord.1 % self.n_cols),
            }
        }

        #[inline]
        pub fn from(&self, index: usize) -> Coord {
            match self.order {
                Order::ColumnMajor => (index / self.n_cols, index % self.n_rows),
                Order::RowMajor => (index / self.n_rows, index % self.n_cols),
            }
        }
    }
}
