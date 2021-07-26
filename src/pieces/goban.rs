//! Module with the goban and his implementations.

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use ahash::AHashMap;

use crate::pieces::{Nat, Set};
use crate::pieces::go_string::GoString;
use crate::pieces::stones::*;
use crate::pieces::util::CircularRenIter;
use crate::pieces::util::coord::{
    is_coord_valid, neighbor_points, one_to_2dim, Point, two_to_1dim,
};
use crate::pieces::zobrist::*;

pub type GoStringIndex = usize;


macro_rules! iter_stones {
    ($goban: expr, $ren_idx: expr) => {
        CircularRenIter::new( $goban.go_strings[$ren_idx].origin, &$goban.next_stone)
    };
}

/// Represents a Goban. the stones are stored in ROW MAJOR (row, colum)
#[derive(Getters, Setters, CopyGetters, Debug, Clone)]
pub struct Goban {
    #[get = "pub"]
    pub(super) go_strings: Vec<GoString>,
    board: Vec<Option<GoStringIndex>>,
    next_stone: Vec<usize>,
    free_slots: Vec<usize>,
    #[get_copy = "pub"]
    size: (usize, usize),

    #[get_copy = "pub"]
    zobrist_hash: u64,
}

impl Goban {
    /// Creates a Goban
    /// # Arguments
    ///
    /// * `(height, width)` a tuple with the height and the width of the desired goban.
    pub fn new((height, width): (Nat, Nat)) -> Self {
        Goban {
            size: (height as usize, width as usize),
            zobrist_hash: 0,
            board: vec![None; height as usize * width as usize],
            next_stone: vec![0; height as usize * width as usize],
            go_strings: Vec::with_capacity(60),
            free_slots: Vec::with_capacity(20),
        }
    }

    /// Creates a Goban from an array of stones.
    pub fn from_array(stones: &[Color]) -> Self {
        let size = ((stones.len() as f32).sqrt()) as Nat;
        let mut game = Goban::new((size, size));
        stones
            .iter()
            .enumerate()
            .map(|(index, color)| (one_to_2dim((size as usize, size as usize), index), color))
            .filter(|s| *(*s).1 != Color::None)
            .for_each(|coord_color| {
                game.push(coord_color.0, *coord_color.1);
            });
        game
    }

    /// Returns the underlying goban in a vector with a RowMajor Policy, calculated on the fly.
    pub fn vec(&self) -> Vec<Color> {
        self.board
            .iter()
            .map(|point| {
                point.map_or(Color::None, |go_str_ptr| self.go_strings[go_str_ptr].color)
            })
            .collect()
    }

    pub fn matrix(&self) -> Vec<Vec<Color>> {
        let mut mat = vec![vec![]];
        for line in self.board.chunks_exact(self.size.1) {
            let v = line
                .iter()
                .map(|o| o.map_or(Color::None, |idx| self.go_strings[idx].color))
                .collect();
            mat.push(v);
        }
        mat
    }

    /// Get number of stones on the goban.
    /// (number of black stones, number of white stones)
    pub fn number_of_stones(&self) -> (u32, u32) {
        self.get_stones()
            .fold((0, 0), |(x1, x2), stone| match stone.color {
                Color::Black => (x1 + 1, x2),
                Color::White => (x1, x2 + 1),
                _ => unreachable!("A stone cannot be empty"),
            })
    }

    #[inline]
    pub(crate) fn board(&self) -> &[Option<GoStringIndex>] {
        &self.board
    }

    pub(crate) fn push_wth_feedback(&mut self, point: Point, color: Color) -> (Vec<usize>, GoStringIndex) {
        let pushed_stone_idx = two_to_1dim(self.size, point);

        let mut adjacent_same_color_str_set = Set::default();
        let mut adjacent_opposite_color_str_set = Set::default();
        let mut liberties = Set::default();

        for neighbor_idx in self.neighbor_points_indexes(pushed_stone_idx) {
            match self.board[neighbor_idx] {
                Some(adj_ren_index) => {
                    if self.go_strings[adj_ren_index].color == color {
                        adjacent_same_color_str_set.insert(adj_ren_index);
                    } else {
                        adjacent_opposite_color_str_set.insert(adj_ren_index);
                    }
                }
                Option::None => {
                    liberties.insert(neighbor_idx);
                }
            }
        }

        let mut dead_ren = Vec::with_capacity(4);
        // for every string of opposite color remove a liberty and update the string.
        for ren_idx in adjacent_opposite_color_str_set {
            let ren = &mut self.go_strings[ren_idx];
            ren.remove_liberty(pushed_stone_idx);
            if ren.is_dead() {
                dead_ren.push(ren_idx);
            }
        }

        let number_of_neighbors_strings = adjacent_same_color_str_set.len();
        let updated_ren_index = match number_of_neighbors_strings {
            0 => {
                self.create_string(pushed_stone_idx, color, liberties)
            }
            1 => {
                let only_ren_idx = adjacent_same_color_str_set.into_iter().next().unwrap();

                self.go_strings[only_ren_idx]
                    .remove_liberty(pushed_stone_idx)
                    .add_liberties_owned(liberties);
                self.add_stone_to_string(only_ren_idx, pushed_stone_idx);
                self.board[pushed_stone_idx] = Some(only_ren_idx);
                only_ren_idx
            }
            _ => {
                let mut to_merge = self.create_string(pushed_stone_idx, color, liberties);
                for adj_ren in adjacent_same_color_str_set {
                    if self.go_strings[adj_ren].number_of_liberties() < self.go_strings[to_merge].number_of_liberties() {
                        self.merge_strings(to_merge, adj_ren);
                    } else {
                        self.merge_strings(adj_ren, to_merge);
                        to_merge = adj_ren;
                    }
                }
                self.go_strings[to_merge].remove_liberty(pushed_stone_idx);
                to_merge
            }
        };
        self.zobrist_hash ^= index_zobrist(pushed_stone_idx, color);
        (dead_ren, updated_ren_index)
    }

    /// Put a stones in the goban. The point depends on the order choose.
    /// default (line, column)
    /// the (0,0) point is in the top left.
    ///
    /// # Panics
    /// if the point is out of bounds
    pub fn push(&mut self, point: Point, color: Color) -> &mut Self {
        assert_ne!(color, Color::None, "We can't push Empty stones");
        assert!(
            (point.0 as usize) < self.size.0,
            "Coordinate point.0 {} out of bounds",
            point.0
        );
        assert!(
            (point.1 as usize) < self.size.1,
            "Coordinate point.1 {} out of bounds",
            point.1
        );
        self.push_wth_feedback(point, color);
        self
    }

    /// Helper function to put a stone.
    #[inline]
    pub fn push_stone(&mut self, stone: Stone) -> &mut Goban {
        self.push(stone.coordinates, stone.color)
    }

    /// Put many stones.
    #[inline]
    pub fn push_many(&mut self, points: &[Point], value: Color) {
        points.iter().for_each(|&point| {
            self.push(point, value);
        })
    }

    pub fn get_go_string(&self, ren_idx: GoStringIndex) -> &GoString {
        &self.go_strings[ren_idx]
    }

    /// Get all the neighbors to the coordinate including empty intersections.
    #[inline]
    pub fn get_neighbors(&self, coord: Point) -> impl Iterator<Item=Stone> + '_ {
        self.neighbor_points(coord).map(move |point| Stone {
            coordinates: point,
            color: self.get_stone(point),
        })
    }

    /// Get all the stones that are neighbor to the coord except empty intersections.
    #[inline]
    pub fn get_neighbors_stones(&self, coord: Point) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(coord).filter(|s| s.color != Color::None)
    }

    /// Get all the neighbors go strings to the point. Only return point with a color.
    #[inline]
    pub fn get_neighbors_strings(&self, coord: Point) -> impl Iterator<Item=&GoString> + '_ {
        self.neighbor_points(coord)
            .map(move |point| two_to_1dim(self.size, point))
            .filter_map(move |point| self.board[point].map(|ren_idx| &self.go_strings[ren_idx]))
    }

    /// Get all the neighbors go strings indexes to the point. Only return point with a color.
    #[inline]
    pub fn get_neighbors_strings_idx(
        &self,
        coord: Point,
    ) -> impl Iterator<Item=GoStringIndex> + '_ {
        self.neighbor_points(coord)
            .map(move |point| two_to_1dim(self.size, point))
            .filter_map(move |point| self.board[point])
    }

    #[inline]
    pub fn get_neighbors_strings_indices_by_idx(
        &self,
        index: usize,
    ) -> impl Iterator<Item=GoStringIndex> + '_ {
        self.neighbor_points_indexes(index)
            .filter_map(move |idx| self.board[idx])
    }

    /// Function for getting the stone in the goban.
    #[inline]
    pub fn get_stone(&self, point: Point) -> Color {
        self.board[two_to_1dim(self.size, point)].map_or(Color::None, |go_str_index| {
            self.go_strings[go_str_index].color
        })
    }

    /// Get all the stones except "Empty stones"
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item=Stone> + '_ {
        self.board.iter().enumerate()
            .filter_map(move |(index, o)| {
                o.map(move |ren_index| Stone {
                    coordinates: one_to_2dim(self.size, index),
                    color: self.go_strings[ren_index].color,
                })
            })
    }

    /// Get stones by their color.
    #[inline]
    pub fn get_stones_by_color(&self, color: Color) -> impl Iterator<Item=Stone> + '_ {
        self.get_points_by_color(color).map(move |c| Stone {
            color,
            coordinates: c,
        })
    }

    /// Get points by their color.
    #[inline]
    pub fn get_points_by_color(&self, color: Color) -> impl Iterator<Item=Point> + '_ {
        let size = self.size;
        self.board
            .iter()
            .enumerate()
            .filter(move |(_, option)| {
                option
                    .map(move |ren_idx| self.go_strings[ren_idx].color)
                    .unwrap_or(Color::None)
                    == color
            })
            .map(move |(index, _)| one_to_2dim(size, index))
    }

    /// Returns the empty stones connected to the stone
    #[inline]
    pub fn get_liberties(&self, point: Point) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(point).filter(|s| s.color == Color::None)
    }

    /// Returns true if the stone has liberties.
    #[inline]
    pub fn has_liberties(&self, point: Point) -> bool {
        self.get_liberties(point).next().is_some()
    }

    /// Get a string for printing the goban in normal shape (0,0) left bottom
    pub fn pretty_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size.0 as u8 {
            for j in 0..self.size.1 as u8 {
                buff.push(match self.get_stone((i, j)) {
                    Color::Black => '●',
                    Color::White => '○',
                    Color::None => {
                        match (
                            i == 0,
                            i == self.size.0 as u8 - 1,
                            j == 0,
                            j == self.size.1 as u8 - 1,
                        ) {
                            (true, _, true, _) => '┏',
                            (true, _, _, true) => '┓',

                            (_, true, true, _) => '┗',
                            (_, true, _, true) => '┛',

                            (true, _, _, _) => '┯',
                            (_, true, _, _) => '┷',
                            (_, _, true, _) => '┠',
                            (_, _, _, true) => '┨',
                            _ => '┼',
                        }
                    }
                });
            }
            buff.push('\n');
        }
        buff
    }

    /// Remove a string from the game, it add liberties to all
    /// adjacent string of not the same color.
    pub fn remove_go_string(&mut self, ren_to_remove_idx: GoStringIndex) {
        let color_of_the_string = self.go_strings[ren_to_remove_idx].color;
        let mut updates: AHashMap<GoStringIndex, Vec<usize>> = AHashMap::new();

        for point_idx in iter_stones!(self, ren_to_remove_idx) {
            for neighbor_str_idx in self
                .get_neighbors_strings_indices_by_idx(point_idx)
                .collect::<Set<_>>()
            {
                if ren_to_remove_idx != neighbor_str_idx {
                    updates
                        .entry(neighbor_str_idx)
                        .and_modify(|v| v.push(point_idx))
                        .or_insert_with(|| vec![point_idx]);
                }
            }
            self.zobrist_hash ^= index_zobrist(point_idx, color_of_the_string);
            self.board[point_idx] = Option::None;
        }

        for (ren_idx, new_liberties) in updates {
            self.go_strings[ren_idx].add_liberties(new_liberties.into_iter());
        }

        self.put_ren_in_bin(ren_to_remove_idx);
    }

    /// Updates the indexes to match actual goban. must use after we put a stone.
    fn update_vec_indexes(&mut self, ren_idx: GoStringIndex) {
        debug_assert_eq!(iter_stones!(self, ren_idx).last().unwrap(), self.go_strings[ren_idx].last);
        for point in iter_stones!(self, ren_idx) {
            unsafe {
                *self.board.get_unchecked_mut(point) = Some(ren_idx);
            }
        }
    }

    /// Get the neighbors points filtered by limits of the board.
    #[inline]
    fn neighbor_points(&self, point: Point) -> impl Iterator<Item=Point> {
        let size = self.size;
        neighbor_points(point)
            .into_iter()
            .filter(move |&p| is_coord_valid(size, p))
    }

    #[inline]
    fn neighbor_points_indexes(&self, index: usize) -> impl Iterator<Item=usize> {
        let size = self.size;
        self.neighbor_points(one_to_2dim(self.size, index))
            .map(move |x| two_to_1dim(size, x))
    }

    #[inline]
    fn get_stones_from_string(&self, ren_idx: usize) -> impl Iterator<Item=usize> + '_ {
        CircularRenIter::new(self.go_strings[ren_idx].origin, &self.next_stone)
    }

    #[inline]
    fn create_string(
        &mut self,
        origin: usize,
        color: Color,
        liberties: Set<usize>,
    ) -> GoStringIndex {
        let ren_to_place = GoString::new_with_liberties(color, origin, liberties);

        self.next_stone[origin] = origin;
        let ren_index = if let Some(free_slot_idx) = self.free_slots.pop()
        {
            self.go_strings[free_slot_idx] = ren_to_place;
            free_slot_idx
        } else {
            self.go_strings.push(ren_to_place);
            self.go_strings.len() - 1
        };
        self.update_vec_indexes(ren_index);
        ren_index
    }

    fn add_stone_to_string(&mut self, ren_idx: usize, stone: usize) {
        let ren = &mut self.go_strings[ren_idx];
        if stone < ren.origin {
            // replace origin
            self.next_stone[stone] = ren.origin;
            self.next_stone[ren.last] = stone;
            ren.origin = stone;
        } else {
            self.next_stone[ren.last] = stone;
            self.next_stone[stone] = ren.origin;
            ren.last = stone;
        }
        ren.num_stones += 1;
        debug_assert_eq!(iter_stones!(self, ren_idx).last().unwrap(), self.go_strings[ren_idx].last);
    }

    fn merge_strings(&mut self, ren1_idx: GoStringIndex, ren2_idx: GoStringIndex) {
        debug_assert_eq!(
            self.go_strings[ren1_idx].color, self.go_strings[ren2_idx].color,
            "Cannot merge two strings of different color"
        );
        debug_assert_ne!(ren1_idx, ren2_idx, "merging the same string");

        let (ren1, ren2) = if ren1_idx < ren2_idx {
            let (s1, s2) = self.go_strings.split_at_mut(ren2_idx);
            (&mut s1[ren1_idx], s2.first_mut().unwrap())
        } else {
            // ren2_idx > ren1_idx
            let (s_contains_ren2, s_contains_ren1) = self.go_strings.split_at_mut(ren1_idx);
            (s_contains_ren1.first_mut().unwrap(), &mut s_contains_ren2[ren2_idx])
        };
        ren1.liberties.extend(&ren2.liberties);


        let ren1_last = ren1.last;
        let ren2_last = ren2.last;

        let ren1_origin = ren1.origin;
        let ren2_origin = ren2.origin;

        if ren1_origin > ren2_origin {
            ren1.origin = ren2_origin;
        } else {
            ren1.last = ren2_last;
        }
        self.next_stone.swap(ren1_last, ren2_last);
        ren1.num_stones += ren2.num_stones;

        self.update_vec_indexes(ren1_idx);
        self.put_ren_in_bin(ren2_idx);
    }

    #[inline]
    fn put_ren_in_bin(&mut self, ren_idx: GoStringIndex) {
        self.go_strings[ren_idx].used = false;
        self.free_slots.push(ren_idx);
    }

    #[cfg(debug_assertions)]
    fn check_integrity_ren(&self, ren_idx: GoStringIndex) {
        assert_eq!(iter_stones!(self, ren_idx).next().unwrap(), self.go_strings[ren_idx].origin, "The origin doesn't match");
        assert_eq!(iter_stones!(self, ren_idx).last().unwrap(), self.go_strings[ren_idx].last, "The last doesn't match");
        if iter_stones!(self, ren_idx).count() as u16 != self.go_strings[ren_idx].num_stones {
            panic!("The number of stones don't match")
        }
    }

    #[cfg(debug_assertions)]
    fn check_integrity_all(&self) {
        for ren_idx in (0..self.go_strings.len()).filter(|&ren_idx| self.go_strings[ren_idx].used) {
            self.check_integrity_ren(ren_idx);
        }
    }
}

impl Display for Goban {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.pretty_string())
    }
}

impl PartialEq for Goban {
    fn eq(&self, other: &Goban) -> bool {
        other.zobrist_hash == self.zobrist_hash
    }
}

impl Hash for Goban {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.zobrist_hash.hash(state)
    }
}

impl Eq for Goban {}

impl Default for Goban {
    fn default() -> Self {
        Goban::new((19, 19))
    }
}
