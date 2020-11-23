use crate::pieces::util::coord::{Point, one_to_2dim};

#[inline]
pub const fn neighbor_points((x1, x2): Point) -> [Point; 4] {
    [
        (x1 + 1, x2),
        (x1.wrapping_sub(1), x2),
        (x1, x2 + 1),
        (x1, x2.wrapping_sub(1)),
    ]
}

#[inline]
pub const fn corner_points((x1, x2): Point) -> [Point; 4] {
    [
        (x1 + 1, x2 + 1),
        (x1.wrapping_sub(1), x2.wrapping_sub(1)),
        (x1 + 1, x2.wrapping_sub(1)),
        (x1.wrapping_sub(1), x2 + 1),
    ]
}

const fn init_neighbors_table() -> [[Point; 4]; 19 * 19] {
    const fn rec_init(mut tab: [[Point; 4]; 19 * 19], index: usize) -> [[Point; 4]; 19 * 19] {
        if index != 19 * 19 {
            rec_init(tab, index + 1)
        } else {
            tab
        }
    }

    let mut res = [[(0, 0); 4]; 19 * 19];
    let mut i = 0;
    while i < 19 * 19 {
        res[i] = neighbor_points(one_to_2dim((19, 19), i));
        i += 1;
    }
    res
}

const NEIGHBOR_TABLE: [[Point; 4]; 19 * 19] = init_neighbors_table();