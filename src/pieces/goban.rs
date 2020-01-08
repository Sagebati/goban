//! Module with the goban and his implementations.

use crate::pieces::go_string::GoString;
use crate::pieces::stones::Color::None;
use crate::pieces::stones::*;
use crate::pieces::util::coord::{corner_coords, neighbors_points, CoordUtil, Order, Point};
use crate::pieces::zobrist::*;
use by_address::ByAddress;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

#[cfg(not(feature = "thread-safe"))]
use std::rc::Rc;

#[cfg(feature = "thread-safe")]
use std::sync::Arc;

#[cfg(not(feature = "thread-safe"))]
type Ptr<T> = Rc<T>;

#[cfg(feature = "thread-safe")]
type Ptr<T> = Arc<T>;

pub type GoStringPtr = ByAddress<Ptr<GoString>>;

///
/// Represents a Goban. With an array with the stones encoded in u8. and the size.
/// only square boards are possible for the moment.
///
#[derive(Getters, Setters, CopyGetters, Debug, Clone)]
pub struct Goban {
    #[get = "pub"]
    go_strings: Vec<Option<GoStringPtr>>,

    #[get_copy = "pub"]
    size: (usize, usize),

    #[get]
    coord_util: CoordUtil,

    #[get_copy = "pub"]
    zobrist_hash: u64,
}

impl Goban {
    pub fn new((height, width): (usize, usize)) -> Self {
        Goban {
            size: (height, width),
            coord_util: CoordUtil::new(height, width),
            zobrist_hash: 0,
            go_strings: vec![Option::None; height * width],
        }
    }

    ///
    /// Creates a goban from an array of stones.
    ///
    pub fn from_array(stones: &[Color], order: Order) -> Self {
        let size = ((stones.len() as f32).sqrt()) as usize;
        let mut g = Goban::new((size, size));
        let coord_util = CoordUtil::new_order(size, size, order);
        stones
            .iter()
            .enumerate()
            .map(|(index, color)| (coord_util.from(index), color))
            .filter(|s| *(*s).1 != Color::None)
            .for_each(|coord_color| {
                g.push(coord_color.0, *coord_color.1);
            });
        g
    }

    ///
    /// Returns the underlying goban in a vector with a RowMajor Policy
    ///
    pub fn raw(&self) -> Vec<Color> {
        self.go_strings
            .iter()
            .map(|point| {
                point
                    .as_ref()
                    .map_or(Color::None, |go_str_ptr| go_str_ptr.color)
            })
            .collect()
    }

    ///
    /// Get number of stones on the goban.
    /// (number of black stones, number of white stones)
    ///
    pub fn number_of_stones(&self) -> (u32, u32) {
        self.get_stones()
            .fold((0, 0), |(x1, x2), stone| match stone.color {
                Color::Black => (x1 + 1, x2),
                Color::White => (x1, x2 + 1),
                _ => unreachable!(),
            })
    }

    ///
    /// Put a stones in the goban. The point depends on the order choose.
    /// default (line, column)
    /// the (0,0) point is in the top left.
    ///
    /// # Panics
    /// if the point is out of bounds
    ///
    pub fn push(&mut self, point: Point, color: Color) -> &mut Self {
        assert_ne!(color, Color::None, "We can't push Empty stones");
        assert!(point.0 < self.size.0, "Coordinate point.0 {} out of bounds", point.0);
        assert!(point.1 < self.size.1, "Coordinate point.1 {} out of bounds", point.1);
        let mut liberties = HashSet::new();
        let mut adjacent_same_color_str_set = HashSet::new();
        let mut adjacent_opposite_color_str_set = HashSet::new();

        for p in neighbors_points(point)
            .into_iter()
            .filter(|&x| self.is_coord_valid(x))
        {
            match &self.go_strings[self.coord_util.to(p)] {
                Some(go_str_ptr) => match go_str_ptr.color {
                    go_str_color if go_str_color == color => {
                        adjacent_same_color_str_set.insert(go_str_ptr.to_owned());
                    }
                    Color::None => debug_assert!(false, "A string cannot be of color none"),
                    _ => {
                        adjacent_opposite_color_str_set.insert(go_str_ptr.to_owned());
                    }
                },
                Option::None => {
                    liberties.insert(p);
                }
            }
        }
        let mut stones = HashSet::new();
        stones.insert(point);
        let new_string = adjacent_same_color_str_set.drain().fold(
            GoString::new(color, stones, liberties),
            |init, same_color_string| self.merge_two_strings(init, same_color_string),
        );

        self.zobrist_hash ^= ZOBRIST[(point, color)];

        self.create_string(new_string);
        for other_color_string in adjacent_opposite_color_str_set
            .drain()
            .map(|go_str_ptr| go_str_ptr.without_liberty(point))
        {
            self.create_string(other_color_string);
        }
        self
    }

    #[inline]
    pub fn push_stone(&mut self, stone: Stone) -> &mut Goban {
        self.push(stone.coordinates, stone.color)
    }

    ///
    /// Put many stones.
    ///
    #[inline]
    pub fn push_many(&mut self, points: &[Point], value: Color) {
        points.iter().for_each(|&point| {
            self.push(point, value);
        })
    }

    ///
    /// Get all the neighbors to the coordinate inluding empty intersections
    ///
    #[inline]
    pub fn get_neighbors(&self, coord: Point) -> impl Iterator<Item = Stone> + '_ {
        neighbors_points(coord)
            .into_iter()
            .filter(move |&point| self.is_coord_valid(point))
            .map(move |point| Stone {
                coordinates: point,
                color: self.get_stone(point),
            })
    }

    ///
    /// Get all the stones that are neighbor to the coord except empty intersections
    ///
    #[inline]
    pub fn get_neighbors_stones(&self, coord: Point) -> impl Iterator<Item = Stone> + '_ {
        self.get_neighbors(coord).filter(|s| s.color != Color::None)
    }

    ///
    /// Get all the neighbors go strings to the point. Only return point with a color.
    ///
    #[inline]
    pub fn get_neighbors_strings(&self, coord: Point) -> impl Iterator<Item = GoStringPtr> + '_ {
        neighbors_points(coord)
            .into_iter()
            .filter(move |&x| self.is_coord_valid(x))
            .filter_map(move |coord| self.go_strings[self.coord_util.to(coord)].clone())
    }

    ///
    /// Function for getting the stone in the goban.
    ///
    #[inline]
    pub fn get_stone(&self, point: Point) -> Color {
        self.go_strings[self.coord_util.to(point)]
            .as_ref()
            .map_or(Color::None, |go_str_ptr| go_str_ptr.color)
    }

    ///
    /// Get all the stones except "Empty stones"
    ///
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item = Stone> + '_ {
        self.go_strings
            .iter()
            .enumerate()
            .filter_map(move |(index, o)| match o {
                Some(x) => Some((self.coord_util.from(index), x.color)),
                Option::None => Option::None,
            })
            .map(|(coordinates, color)| Stone { coordinates, color })
    }

    ///
    /// Get stones by their color.
    ///
    #[inline]
    pub fn get_stones_by_color(&self, color: Color) -> impl Iterator<Item = Stone> + '_ {
        self.get_points_by_color(color).map(move |c| Stone {
            color,
            coordinates: c,
        })
    }

    #[inline]
    pub fn get_points_by_color(&self, color: Color) -> impl Iterator<Item = Point> + '_ {
        self.go_strings
            .iter()
            .enumerate()
            .filter(move |(_, point)| match point {
                Some(go_str_ptr) => go_str_ptr.color == color,
                Option::None => color == Color::None,
            })
            .map(move |(index, _)| self.coord_util.from(index))
    }

    ///
    /// Returns the empty stones connected to the stone
    ///
    #[inline]
    pub fn get_liberties(&self, point: Point) -> impl Iterator<Item = Stone> + '_ {
        self.get_neighbors(point).filter(|s| s.color == Color::None)
    }

    ///
    /// Returns true if the stone has liberties.
    ///
    #[inline]
    pub fn has_liberties(&self, point: Point) -> bool {
        self.get_liberties(point).next().is_some()
    }

    /// Detects true eyes.
    /// Except for this form :
    /// ```{nothing}
    ///  ++
    ///  + ++
    ///  ++ +
    ///    ++
    /// ```
    /// This function is only used for performance checking in the rules,
    /// and not for checking is a point is really an eye !
    pub fn is_point_an_eye(&self, point: Point, color: Color) -> bool {
        if self.get_stone(point) != None {
            return false;
        }
        if self.get_neighbors(point).any(|stone| stone.color != color) {
            return false;
        }
        let mut corner_ally = 0;
        let mut corner_off_board = 0;
        for point in corner_coords(point) {
            if self.is_coord_valid(point) {
                if self.get_stone(point) == color {
                    corner_ally += 1
                }
            } else {
                corner_off_board += 1;
            }
        }
        if corner_off_board > 0 {
            corner_off_board + corner_ally == 4
        } else {
            corner_ally == 4
        }
    }

    ///
    /// Return true if the coord is in the goban.
    ///
    #[inline]
    pub fn is_coord_valid(&self, coord: Point) -> bool {
        coord.0 < self.size.0 && coord.1 < self.size.1
    }

    ///
    /// Get a string for printing the goban in normal shape (0,0) left bottom
    ///
    pub fn pretty_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size.0 {
            for j in 0..self.size.1 {
                buff.push(match self.get_stone((i, j)) {
                    Color::White => '●',
                    Color::Black => '○',
                    Color::None => {
                        match (i == 0, i == self.size.0 - 1, j == 0, j == self.size.1 - 1) {
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

    ///
    /// Remove a string from the game, it add liberties to all
    /// adjacent string of not the same color.
    ///
    pub fn remove_go_string(&mut self, go_string_to_remove: GoStringPtr) {
        let color_of_the_string = go_string_to_remove.color;
        for &point in go_string_to_remove.stones() {
            for neighbor_str_ptr in self.get_neighbors_strings(point).collect::<HashSet<_>>() {
                if go_string_to_remove != neighbor_str_ptr {
                    self.create_string(neighbor_str_ptr.with_liberty(point));
                }
            }
            self.zobrist_hash ^= ZOBRIST[(point, color_of_the_string)];

            // Remove each point from the map. The Rc will be dropped "normally".
            self.go_strings[self.coord_util.to(point)] = Option::None;
        }

        debug_assert!(
            Ptr::strong_count(&go_string_to_remove) == 1,
            "strong count: {}",
            Ptr::strong_count(&go_string_to_remove)
        );
    }

    ///
    /// Removes the dead stones from the goban by specifying a color stone.
    /// Returns the number of stones removed from the goban.
    ///
    pub fn remove_captured_stones_turn(&mut self, color: Color) -> u32 {
        let mut number_of_stones_captured = 0u32;
        let string_without_liberties = self
            .get_strings_of_stones_without_liberties_wth_color(color)
            .collect::<HashSet<_>>();
        for group_of_stones in string_without_liberties {
            number_of_stones_captured += group_of_stones.stones().len() as u32;
            self.remove_go_string(group_of_stones);
        }
        number_of_stones_captured
    }

    ///
    /// Just create the Rc pointer and add it to the set.
    /// moves out the string.
    ///
    fn create_string(&mut self, string_to_add: GoString) {
        let new_string: GoStringPtr = Ptr::new(string_to_add).into();
        self.update_vec_indexes(new_string);
    }

    ///
    /// Deletes all the Rc from the go_strings, then merges the two_string
    ///
    fn merge_two_strings(&mut self, first: GoString, other: GoStringPtr) -> GoString {
        for &point in other.stones() {
            unsafe { *self.go_strings.get_unchecked_mut(self.coord_util.to(point)) = Option::None };
        }

        first.merge_with((**other).clone())
    }

    fn update_vec_indexes(&mut self, go_string: GoStringPtr) {
        for &stone in go_string.stones() {
            unsafe {
                *self.go_strings.get_unchecked_mut(self.coord_util.to(stone)) =
                    Some(go_string.clone());
            }
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
