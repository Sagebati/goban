//! Module with the goban and his implementations.

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use ahash::AHashMap;

use crate::pieces::{GoStringPtr, Nat, Ptr, Set};
use crate::pieces::go_string::GoString;
use crate::pieces::stones::*;
use crate::pieces::util::coord::{
    is_coord_valid, neighbor_points, one_to_2dim, Point, two_to_1dim,
};
use crate::pieces::zobrist::*;

/// Represents a Goban. the stones are stored in ROW MAJOR (row, column)
#[derive(Getters, Setters, CopyGetters, Debug, Clone)]
pub struct Goban {
    #[get = "pub"]
    pub(super) go_strings: Vec<Option<GoStringPtr>>,

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
            go_strings: vec![Option::None; height as usize * width as usize],
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
    pub fn raw(&self) -> Vec<Color> {
        self.go_strings
            .iter()
            .map(|point| {
                point
                    .as_ref()
                    .map_or(Color::None, |go_str_ptr| go_str_ptr.color())
            })
            .collect()
    }

    pub fn raw_matrix(&self) -> Vec<Vec<Color>> {
        let mut mat = vec![vec![]];
        for line in self.go_strings.chunks_exact(self.size.1) {
            let v = line
                .iter()
                .map(|o| o.as_ref().map_or(Color::None, |r| r.color()))
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

        self.push_feedback(point, color);
        self
    }

    pub(crate) fn push_feedback(&mut self, point: Point, color: Color) -> (GoStringPtr,Vec<GoStringPtr>) {
        let pushed_stone_idx = two_to_1dim(self.size, point);
        let mut new_string = GoString::new_with_color_and_stone_idx(color, pushed_stone_idx);
        let mut num_stones_connected = 0;

        let mut adjacent_friend_go_str = Set::default();
        let mut adjacent_opposite_color_str_set = Set::default();

        for neighbor_idx in self.neighbor_points_index(pushed_stone_idx) {
            match &self.go_strings[neighbor_idx] {
                Some(adj_go_str_ptr) => {
                    if adj_go_str_ptr.color() == color {
                        num_stones_connected += adj_go_str_ptr.stones().len();
                        adjacent_friend_go_str.insert(adj_go_str_ptr.to_owned());
                    } else {
                        adjacent_opposite_color_str_set.insert(adj_go_str_ptr.to_owned());
                    }
                }
                Option::None => {
                    new_string.add_liberty(neighbor_idx);
                }
            }
        }
        new_string.reserve_stone(num_stones_connected);
        adjacent_friend_go_str.into_iter()
            .for_each(|same_color_string| {
                new_string.merge(&same_color_string);
            });
        #[cfg(debug_assertions)]
        if new_string.contains_liberty(pushed_stone_idx) {
            new_string.remove_liberty(pushed_stone_idx);
        }
        #[cfg(not(debug_assertions))]
            new_string.remove_liberty(pushed_stone_idx);

        let added_str = self.put_string(new_string);

        self.zobrist_hash ^= index_zobrist(pushed_stone_idx, color);

        // for every string of opposite color remove a liberty and update the string.
        let mut dead_strings =
            Vec::with_capacity(adjacent_opposite_color_str_set.len());

        for other_color_string in adjacent_opposite_color_str_set
        {
            let new_other_str =  self.put_string(other_color_string.without_liberty(pushed_stone_idx));
            if new_other_str.is_dead() {
                dead_strings.push(new_other_str);
            }
        }
        (added_str,dead_strings)
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
    pub fn get_neighbors_strings(&self, coord: Point) -> impl Iterator<Item=GoStringPtr> + '_ {
        self.neighbor_points(coord)
            .map(move |point| two_to_1dim(self.size, point))
            .filter_map(move |point| self.go_strings[point].clone())
    }

    #[inline]
    pub fn get_neighbors_strings_index(
        &self,
        index: usize,
    ) -> impl Iterator<Item=GoStringPtr> + '_ {
        self.neighbor_points_index(index)
            .filter_map(move |idx| self.go_strings[idx].clone())
    }

    /// Function for getting the stone in the goban.
    #[inline]
    pub fn get_stone(&self, point: Point) -> Color {
        self.go_strings[two_to_1dim(self.size, point)]
            .as_ref()
            .map_or(Color::None, |go_str_ptr| go_str_ptr.color())
    }

    /// Get all the stones except "Empty stones"
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item=Stone> + '_ {
        self.go_strings
            .iter()
            .enumerate()
            .filter_map(move |(index, o)| match o {
                Some(x) => Some((one_to_2dim(self.size, index), x.color())),
                Option::None => Option::None,
            })
            .map(|(coordinates, color)| Stone { coordinates, color })
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
    pub fn get_points_by_color(&self, color: Color) -> impl Iterator<Item=Point> {
        let mut res = Vec::with_capacity(self.size.0 as usize * self.size.1 as usize);
        for i in 0..self.size.0 as u8 {
            for j in 0..self.size.1 as u8 {
                match &self.go_strings[two_to_1dim(self.size, (i, j))] {
                    Some(go_str_ptr) if go_str_ptr.color() == color => res.push((i, j)),
                    Option::None if color == Color::None => res.push((i, j)),
                    _ => {}
                }
            }
        }
        res.into_iter()
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
    pub fn remove_go_string(&mut self, go_string_to_remove: GoStringPtr) {
        let color_of_the_string = go_string_to_remove.color();

        let mut updates: AHashMap<GoStringPtr, Vec<usize>> = AHashMap::new();
        for &stone_idx in go_string_to_remove.stones() {
            for neighbor_str_ptr in self
                .get_neighbors_strings_index(stone_idx)
                .collect::<Set<_>>()
            {
                if go_string_to_remove != neighbor_str_ptr {
                    updates
                        .entry(neighbor_str_ptr)
                        .and_modify(|v| v.push(stone_idx))
                        .or_insert_with(|| vec![stone_idx]);
                }
            }

            self.zobrist_hash ^= index_zobrist(stone_idx, color_of_the_string);
            self.go_strings[stone_idx] = Option::None;
        }

        for (k, v) in updates {
            self.put_string(k.with_liberties(v.into_iter()));
        }
    }

    /// Just create the Rc pointer and add it to the set.
    /// moves out the string.
    fn put_string(&mut self, string_to_add: GoString) -> GoStringPtr {
        let new_string: GoStringPtr = Ptr::new(string_to_add).into();
        self.update_goban_indexes(&new_string);
        new_string
    }

    /// Updates the indexes to math actual goban. must use after an we put a stone
    fn update_goban_indexes(&mut self, go_string: &GoStringPtr) {
        for &point in go_string.stones() {
            unsafe {
                *self.go_strings.get_unchecked_mut(point) = Some(go_string.clone());
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

    fn neighbor_points_index(&self, index: usize) -> impl Iterator<Item=usize> {
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

impl Eq for Goban {}

impl Default for Goban {
    fn default() -> Self {
        Goban::new((19, 19))
    }
}
