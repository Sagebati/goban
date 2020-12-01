//! Module with the goban and his implementations.

use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use ahash::AHashMap;
use vob::*;

use crate::pieces::{Nat, Set};
use crate::pieces::go_string::GoString;
use crate::pieces::stones::*;
use crate::pieces::util::coord::{
    is_coord_valid, neighbor_points, one_to_2dim, Point, two_to_1dim,
};
use crate::pieces::zobrist::*;

type GoStringIndex = usize;

/// Represents a Goban. the stones are stored in ROW MAJOR (row, colum)
#[derive(Getters, Setters, CopyGetters, Debug, Clone, Eq)]
pub struct Goban {
    #[get = "pub"]
    pub(super) go_strings: Vec<GoString>,
    board: Vec<Option<GoStringIndex>>,
    #[get_copy = "pub"]
    size: (usize, usize),

    #[get_copy = "pub"]
    zobrist_hash: u64,
    pub(super) used: Vob,
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
            go_strings: Default::default(),
            used: vob![],
        }
    }

    /// Creates a Goban from an array of stones;
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
    pub fn raw(&self) -> Vec<Color> {
        self.board
            .iter()
            .map(|point| {
                point
                    .map_or(Color::None, |go_str_idx| self.go_strings[go_str_idx].color())
            })
            .collect()
    }

    pub fn raw_matrix(&self) -> Vec<Vec<Color>> {
        let mut mat = vec![vec![]];
        for line in self.board.chunks_exact(self.size.1) {
            let v = line
                .iter()
                .map(|o| o.map_or(Color::None, |idx| self.go_strings[idx].color()))
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

        let pushed_stone_idx = two_to_1dim(self.size, point);
        let mut new_string = GoString::new_with_color_and_stone_idx(color, pushed_stone_idx);
        let mut num_stones_connected = 0;

        let mut adjacent_same_color_str_set = Set::default();
        let mut adjacent_opposite_color_str_set = Set::default();

        for neighbor_idx in self.neighbor_points_indexes(pushed_stone_idx) {
            match self.board[neighbor_idx] {
                Some(adj_ren_index) =>
                    match self.go_strings[adj_ren_index].color() {
                        go_str_color if go_str_color == color => {
                            num_stones_connected += self.go_strings[adj_ren_index].stones().len();
                            adjacent_same_color_str_set.insert(adj_ren_index);
                        }
                        Color::None => unreachable!("A string cannot be of color none"),
                        go_str_color if go_str_color != color => {
                            adjacent_opposite_color_str_set.insert(adj_ren_index);
                        }
                        _ => unreachable!()
                    },
                Option::None => {
                    new_string.add_liberty(neighbor_idx);
                }
            }
        }

        // for every string of opposite color remove a liberty and update the string.
        adjacent_opposite_color_str_set
            .into_iter()
            .for_each(|go_str_idx| {
                self.go_strings.get_mut(go_str_idx).unwrap().remove_liberty(pushed_stone_idx);
            });

        let number_of_neighbors_strings = adjacent_same_color_str_set.len();
        match number_of_neighbors_strings {
            0 => {
                self.place_string(new_string);
            }
            1 => {
                let &only_ren_idx = adjacent_same_color_str_set.iter().next().unwrap();
                self.go_strings.get_mut(only_ren_idx)
                    .unwrap()
                    .merge(&new_string)
                    .remove_liberty(pushed_stone_idx);
                self.board[pushed_stone_idx] = Some(only_ren_idx);
            }
            _ => {
                new_string.reserve_stone(num_stones_connected);
                adjacent_same_color_str_set.iter()
                    .for_each(|&ren_idx| {
                        new_string.merge(&self.go_strings[ren_idx]);
                        // remove the string because the merge.
                        self.used.set(ren_idx, false);
                    });
                new_string.remove_liberty(pushed_stone_idx);
                self.place_string(new_string);
            }
        }
        self.zobrist_hash ^= index_zobrist(pushed_stone_idx, color);
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
    pub fn get_neighbors_strings_idx(&self, coord: Point) -> impl Iterator<Item=GoStringIndex> + '_ {
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
        self.board[two_to_1dim(self.size, point)]
            .map_or(Color::None, |go_str_index| self.go_strings[go_str_index].color())
    }

    /// Get all the stones except "Empty stones"
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item=Stone> + '_ {
        self.board
            .iter()
            .enumerate()
            .filter_map(move |(index, o)|
                o.map(move |ren_index|
                    Stone {
                        coordinates: one_to_2dim(self.size, index),
                        color: self.go_strings[ren_index].color(),
                    })
            )
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
        self.board.iter()
            .enumerate()
            .filter(move |(_, option)|
                option.map(move |ren_idx| self.go_strings[ren_idx].color()).unwrap_or(Color::None) == color)
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
                    Color::White => '●',
                    Color::Black => '○',
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
        let ren_to_rem = &self.go_strings[ren_to_remove_idx];
        let color_of_the_string = ren_to_rem.color();
        let mut updates: AHashMap<usize, Vec<usize>> = AHashMap::new();
        for &point_idx in ren_to_rem.stones() {
            for neighbor_str_idx in self
                .get_neighbors_strings_indices_by_idx(point_idx)
                .collect::<HashSet<_>>() {
                if ren_to_remove_idx != neighbor_str_idx {
                    updates.entry(neighbor_str_idx)
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

        self.used.set(ren_to_remove_idx, false);
    }

    /// Removes the dead stones from the goban by specifying a color stone.
    /// If there is only one stone captured, then update self.ko_point.
    /// Returns the number of stones removed from the goban.
    pub fn remove_captured_stones_turn(&mut self, color: Color) -> (u32, Option<Point>) {
        let mut number_of_stones_captured = 0;
        let mut ko_point = None;

        let go_strings_without_liberties = self
            .get_go_strings_without_liberties_by_color(color)
            .collect::<Vec<_>>();

        let one_str_captured = go_strings_without_liberties.len() == 1;

        for index_ren in go_strings_without_liberties {
            let ren_without_liberties = &self.go_strings[index_ren];
            number_of_stones_captured += ren_without_liberties.stones().len() as u32;
            // if only one string of one stone is takes then it's a Ko point.
            if one_str_captured && number_of_stones_captured == 1 {
                ko_point = Some(*ren_without_liberties.stones().iter().next().unwrap())
            }
            self.remove_go_string(index_ren);
        }
        let size = self.size;
        (
            number_of_stones_captured,
            ko_point.map(move |v| one_to_2dim(size, v)),
        )
    }

    /// Add the string to the vec then updates the indexes;
    fn place_string(&mut self, string_to_add: GoString) {
        debug_assert_eq!(self.go_strings.len(), self.used.len());
        // If the vector has some dead structure replace by the new
        if let Some(ren_idx) = self.used.iter_unset_bits(0..).next() {
            self.go_strings[ren_idx] = string_to_add;
            self.update_vec_indexes(ren_idx);
            self.used.set(ren_idx, true);
        } else {
            self.go_strings.push(string_to_add);
            self.update_vec_indexes(self.go_strings.len() - 1);
            self.used.push(true);
        }
    }

    /// Updates the indexes to math actual goban. must use after an we put a stone
    fn update_vec_indexes(&mut self, ren_idx: GoStringIndex) {
        for &point in self.go_strings[ren_idx].stones() {
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

impl Default for Goban {
    fn default() -> Self {
        Goban::new((19, 19))
    }
}
