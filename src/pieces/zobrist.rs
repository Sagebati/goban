use crate::pieces::stones::Color;
use crate::pieces::util::coord::Point;
use rand::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::ops::Index;

const SEED: u64 = 172_147_124;

#[derive(Debug, Clone)]
pub struct ZobristTable {
    hashes: Vec<Vec<u64>>,
    n: usize,
}

impl ZobristTable {
    pub(crate) fn new(n: usize) -> Self {
        let mut rng = XorShiftRng::seed_from_u64(SEED);
        let hashes = (0..n * n)
            .map(|_| (0..2)
                .map(|_| rng.next_u64())
                .collect())
            .collect();
        ZobristTable { hashes, n }
    }
}

impl Index<(Point, Color)> for ZobristTable {
    type Output = u64;

    fn index(&self, (x, color): (Point, Color)) -> &Self::Output {
        &self.hashes[x.0 * self.n + x.1][(color as u8 - 1) as usize]
    }
}

lazy_static! {
    pub static ref ZOBRIST: ZobristTable = ZobristTable::new(19);
}
