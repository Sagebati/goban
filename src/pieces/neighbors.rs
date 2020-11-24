use crate::pieces::util::coord::{Point, two_to_1dim};
use crate::pieces::{GOBAN_SIZE, N_POINTS};

#[inline]
const fn is_corner(p: Point) -> bool {
    matches!(p, (0, 18) | (18, 0) | (18, 18) | (0, 0))
}

#[inline]
const fn is_border(p: Point) -> bool {
    matches!(p,
        (1..=17, 0) | (0, 1..=17) | (18, 1..=17) | (1..=17, 18))
}

const fn array_to_1_dim<const N: usize>(array: [Point; N]) -> [usize; N] {
    let mut res = [0; N];
    let mut i = 0;
    while i < array.len() {
        res[i] = two_to_1dim(GOBAN_SIZE, array[i]);
        i += 1;
    }

    res
}

#[inline]
pub const fn neighbor_points_center((x1, x2): Point) -> [Point; 4] {
    [
        (x1 + 1, x2),
        (x1 - 1, x2),
        (x1, x2 + 1),
        (x1, x2 - 1),
    ]
}

#[inline]
pub const fn neighbor_points_corner((x1, x2): Point) -> [Point; 2] {
    match (x1, x2) {
        (0, 18) => [(1, 18), (0, 17)],
        (18, 0) => [(18, 1), (17, 0)],
        (18, 18) => [(17, 18), (18, 17)],
        (0, 0) => [(0, 1), (1, 0)],
        _ => unreachable!(),
    }
}

#[inline]
const fn neighbor_points_border((x1, x2): Point) -> [Point; 3] {
    match (x1, x2) {
        (1..=17, 0) => [(x1 + 1, x2), (x1, 1), (x1 - 1, 0)],
        (0, 1..=17) => [(1, x2), (0, x2 - 1), (0, x2 + 1)],
        (18, 1..=17) => [(18, x2 + 1), (17, x2), (18, x2 - 1)],
        (1..=17, 18) => [(x1, 17), (x1 + 1, 18), (x1 - 1, 18)],
        _ => unreachable!()
    }
}

const N_POINTS_BORDER: usize = (GOBAN_SIZE.0 - 2) * 4;
const N_POINTS_CORNER: usize = 4;
const N_POINTS_CENTER: usize = GOBAN_SIZE.0 * GOBAN_SIZE.1 - N_POINTS_BORDER - N_POINTS_CORNER;
const DATA_LENGTH: usize = N_POINTS_BORDER * 3 + N_POINTS_CORNER * 2 + N_POINTS_CENTER * 4;

const fn init_neighbors_table() -> ([usize; DATA_LENGTH], [Point; DATA_LENGTH],
                                    [(usize, usize); N_POINTS]) {
    let mut data = [0; DATA_LENGTH];
    let mut data_point = [(0, 0); DATA_LENGTH];
    let mut indexes = [(0, 0); N_POINTS];
    let mut data_index_offset = 0;
    let mut x = 0;
    while x < GOBAN_SIZE.0 {
        let mut y = 0;
        while y < GOBAN_SIZE.1 {
            let point = (x as u8, y as u8);
            let p_index = two_to_1dim(GOBAN_SIZE, point);
            if is_corner(point) {
                let arr_point = neighbor_points_corner(point);
                let arr = array_to_1_dim(arr_point);
                indexes[p_index] = (data_index_offset, data_index_offset + 2);
                data[data_index_offset] = arr[0];
                data_point[data_index_offset] = arr_point[0];
                data_index_offset += 1;
                data[data_index_offset] = arr[1];
                data_point[data_index_offset] = arr_point[1];
                data_index_offset += 1;
            } else if is_border(point) {
                let points_neighbors = neighbor_points_border(point);
                let arr = array_to_1_dim(points_neighbors);
                let mut arr_index = 0;
                indexes[p_index] = (data_index_offset, data_index_offset + 3);
                while arr_index < 3 {
                    data[data_index_offset + arr_index] = arr[arr_index];
                    data_point[data_index_offset + arr_index] = points_neighbors[arr_index];
                    arr_index += 1;
                }
                data_index_offset += 3;
            } else {
                let points_neighbors = neighbor_points_center(point);
                let indexes_neighbors = array_to_1_dim(points_neighbors);
                let mut arr_index = 0;
                indexes[p_index] = (data_index_offset, data_index_offset + 4);
                while arr_index < 4 {
                    data[data_index_offset + arr_index] = indexes_neighbors[arr_index];
                    data_point[data_index_offset + arr_index] = points_neighbors[arr_index];
                    arr_index += 1;
                }
                data_index_offset += 4;
            }
            y += 1;
        }
        x += 1;
    }
    (data, data_point, indexes)
}

const BUFFER: ([usize; DATA_LENGTH], [Point; DATA_LENGTH], [(usize, usize); N_POINTS]) =
    init_neighbors_table();
pub const NEIGHBOR_TABLE: [usize; DATA_LENGTH] = BUFFER.0;
pub const NEIGHBOR_POINT_TABLE: [Point; DATA_LENGTH] = BUFFER.1;
pub const INDEXES: [(usize, usize); N_POINTS] = BUFFER.2;

pub fn get_neighbors(point_index: usize) -> &'static [usize] {
    let (start, end) = INDEXES[point_index];
    &NEIGHBOR_TABLE[start..end]
}

pub fn get_neighbors_point(point_index: usize) -> &'static [Point] {
    let (start, end) = INDEXES[point_index];
    &NEIGHBOR_POINT_TABLE[start..end]
}

