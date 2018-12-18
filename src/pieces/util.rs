const ORDER: Order = Order::RowMajor;

pub type Coord = (usize, usize);

pub enum Order {
    RowMajor,
    ColumnMajor,
}

pub struct CoordUtil {
    n_rows: usize,
    n_cols: usize,
}

impl CoordUtil {
    pub fn new(n_rows: usize, n_cols: usize) -> CoordUtil {
        CoordUtil { n_cols, n_rows }
    }

    pub fn to(&self, coord: &Coord) -> usize {
        match ORDER {
            Order::ColumnMajor => {
                (coord.0 * self.n_cols + coord.1 % self.n_rows)
            }
            Order::RowMajor => {
                (coord.0 * self.n_rows + coord.1 % self.n_cols)
            }
        }
    }

    pub fn from(&self, index: usize) -> Coord {
        match ORDER {
            Order::ColumnMajor => {
                (index / self.n_cols, index % self.n_rows)
            }
            Order::RowMajor => {
                (index / self.n_rows, index % self.n_cols)
            }
        }
    }
}
