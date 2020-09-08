use std::ops::Index;

use crate::pieces::util::coord::{
    is_coord_valid, neighbor_points, one_to_2dim, Point, two_to_1dim,
};

#[derive(Debug, Clone)]
pub struct NeighborTable {
    table: Vec<Vec<usize>>,
    n: usize,
}

impl NeighborTable {
    pub(crate) fn new(n: usize) -> Self {
        let table = (0..n * n)
            .map(|i| {
                neighbor_points(one_to_2dim((n, n), i))
                    .into_iter()
                    .filter(|&p| is_coord_valid((n, n), p))
                    .map(|p| two_to_1dim((n, n), p))
                    .collect()
            })
            .collect();
        NeighborTable { table, n }
    }
}

impl Index<usize> for NeighborTable {
    type Output = Vec<usize>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

impl Index<Point> for NeighborTable {
    type Output = Vec<usize>;

    fn index(&self, index: (u8, u8)) -> &Self::Output {
        let index = two_to_1dim((self.n, self.n), index);
        &self[index]
    }
}

lazy_static! {
    pub static ref NEIGHBOR: NeighborTable = NeighborTable::new(19);
}
