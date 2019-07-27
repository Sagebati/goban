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

    pub fn neighbors_coords(coord: &Coord) -> Vec<Coord> {
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
        pub fn to(&self, coord: &Coord) -> usize {
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

/*pub mod zobrist {
    use crate::pieces::stones::Color;
    use crate::pieces::goban::Goban;

    use rand_core::{RngCore, SeedableRng};
    use rand_isaac::Isaac64Rng;
    use crate::pieces::util::coord::CoordUtil;
    use std::collections::HashMap;
    use std::sync::Mutex;

    const SEED: u64 = 0x7fffffffffffffff;

    lazy_static! {
        static ref TABLES: Mutex<HashMap<usize, Vec<Vec<u64>>>> = Mutex::new({
            HashMap::new()
        });
    }

    fn init_table(size: usize) -> Vec<Vec<u64>> {
        let mut rng: Isaac64Rng = Isaac64Rng::seed_from_u64(SEED);
        let coord_util = CoordUtil::new(size, size);
        let mut res = vec![vec![]];
        for i in 0..size {
            for j in 0..size {
                for z in 0..2 {
                    res[coord_util.to(&(i, j))][z] = rng.next_u64();
                }
            }
        }
        res
    }

    pub fn value_stone(color: Color) -> usize {
        if color == Color::White {
            1
        } else {
            0
        }
    }

    pub fn get_table(size: usize) -> Vec<Vec<u64>> {
        if TABLES.lock().unwrap().contains_key(&size) {
            TABLES.lock().unwrap().get(&size).unwrap().clone()
        } else {
            TABLES.lock().unwrap().insert(size, init_table(size));
            TABLES.lock().unwrap().get(&size).unwrap().clone()
        }
    }

    pub fn hash(size: usize, goban: &Goban) -> u64 {
        let mut h = 0;
        let zobrist_table = init_table(size);
        let coord_util = CoordUtil::new(size, size);
        for i in 0..size {
            for j in 0..size {
                let stone = goban[(i, j)];
                if stone != Color::None {
                    h ^= zobrist_table[coord_util.to(&(i, j))][value_stone(stone)];
                }
            }
        }
        h
    }
}*/
