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

pub fn neighbors_connected(coord: &Coord) -> Vec<Coord> {
    let (x, y) = *coord;
    vec![
        (x.wrapping_add(1), y),
        (x.wrapping_sub(1), y),
        (x, y.wrapping_add(1)),
        (x, y.wrapping_sub(1))
    ]
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
