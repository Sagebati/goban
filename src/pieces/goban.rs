//! Module with the goban and his implementations.

use crate::pieces::go_string::GoString;
use crate::pieces::stones::Color::None;
use crate::pieces::stones::*;
use crate::pieces::util::coord::{corner_coords, neighbors_points, CoordUtil, Order, Point};
use crate::pieces::zobrist::*;
use by_address::ByAddress;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

#[cfg(not(feature = "thread-safe"))]
use std::rc::Rc;

#[cfg(not(feature = "thread-safe"))]
pub type GoStringPtr = ByAddress<Rc<GoString>>;

use std::collections::HashSet;
#[cfg(feature = "thread-safe")]
use std::sync::Arc;

#[cfg(feature = "thread-safe")]
pub type GoStringPtr = ByAddress<Arc<GoString>>;

///
/// Represents a Goban. With an array with the stones encoded in u8. and the size.
/// only square boards are possible for the moment.
///
#[derive(Getters, Setters, CopyGetters, Debug, Clone)]
pub struct Goban {
    #[get = "pub"]
    go_strings: Vec<Option<GoStringPtr>>,

    #[get_copy = "pub"]
    size: usize,

    #[get]
    coord_util: CoordUtil,

    #[get_copy = "pub"]
    hash: u64,
}

impl Goban {
    pub fn new(size: usize) -> Self {
        Goban {
            size,
            coord_util: CoordUtil::new(size, size),
            hash: 0,
            go_strings: vec![Option::None; size * size],
        }
    }

    ///
    /// Creates a goban from an array of stones.
    ///
    pub fn from_array(stones: &[Color], order: Order) -> Self {
        let size = ((stones.len() as f32).sqrt()) as usize;
        let mut g = Goban::new(size);
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

    pub fn tab(&self) -> Vec<Color> {
        self.get_points()
            .map(|point| self.get_stone(point))
            .collect()
    }

    ///
    /// Put a stones in the goban. The point depends on the order choose.
    /// default (line, column)
    /// the (0,0) point is in the top left.
    ///
    pub fn push(&mut self, point: Point, color: Color) -> &mut Self {
        if color == Color::None {
            panic!("We can't put empty stones")
        }
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
                    Color::None => panic!("a string cannot be of color none"),
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
        let mut new_string = GoString::new(color, stones, liberties);
        // Merges the neighbors allies string and then creates the string
        for same_color_string in adjacent_same_color_str_set.drain() {
            new_string = self.merge_two_strings(new_string, same_color_string);
        }

        self.hash ^= ZOBRIST[(point, color)];

        self.create_string(new_string);
        for mut other_color_string in adjacent_opposite_color_str_set
            .drain()
            .map(|go_str_ptr| (**go_str_ptr).clone())
        {
            other_color_string.remove_liberty(point);
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
    /// Function for getting the stone in the goban.
    ///
    #[inline]
    pub fn get_stone(&self, point: Point) -> Color {
        self.go_strings[self.coord_util.to(point)]
            .as_ref()
            .map_or(Color::None, |go_str_ptr| go_str_ptr.color)
    }

    ///
    /// Get all the neighbors to the coordinate
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

    #[inline]
    fn get_points(&self) -> impl Iterator<Item = Point> + '_ {
        (0..self.size * self.size).map(move |index| self.coord_util.from(index))
    }

    #[inline]
    pub fn get_empty_points(&self) -> impl Iterator<Item=Point> + '_ {
        self.go_strings.iter()
            .enumerate()
            .filter(|(_, ptr)| ptr.is_none())
            .map(move |(index, _)| self.coord_util.from(index))
    }

    #[inline]
    pub fn get_points_by_color(&self, color: Color) -> impl Iterator<Item = Point> + '_ {
        self.get_points()
            .filter(move |&point| self.get_stone(point) == color)
    }

    ///
    /// Get all the stones except "Empty stones"
    ///
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item = Stone> + '_ {
        self.get_points()
            .map(move |point| Stone {
                coordinates: point,
                color: self.get_stone(point),
            })
            .filter(|stone| stone.color != Color::None)
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

    ///
    /// Returns the empty stones connected to the stone
    ///
    #[inline]
    pub fn get_liberties(&self, stone: Stone) -> impl Iterator<Item = Stone> + '_ {
        self.get_neighbors(stone.coordinates)
            .filter(|s| s.color == Color::None)
    }

    ///
    /// Returns true if the stone has liberties.
    ///
    #[inline]
    pub fn has_liberties(&self, point: Stone) -> bool {
        self.get_liberties(point).next().is_some()
    }

    ///
    /// Get a string for printing the goban in normal shape (0,0) left bottom
    ///
    pub fn pretty_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size {
            buff.push('|');
            for j in 0..self.size {
                buff.push(match self.get_stone((i, j)) {
                    Color::White => WHITE_STONE,
                    Color::Black => BLACK_STONE,
                    Color::None => EMPTY_STONE,
                });
            }
            buff.push('|');
            buff.push('\n');
        }
        buff
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
        coord.0 < self.size && coord.1 < self.size
    }

    ///
    /// Just create the Rc pointer and add it to the set.
    /// moves out the string.
    ///
    fn create_string(&mut self, string_to_add: GoString) {
        #[cfg(not(feature = "thread-safe"))]
        let new_string: GoStringPtr = Rc::new(string_to_add).into();
        #[cfg(feature = "thread-safe")]
        let new_string: GoStringPtr = Arc::new(string_to_add).into();
        self.update_map_indexes(new_string);
    }

    ///
    /// Deletes all the Rc from the go_strings set then merges the two_string
    ///
    fn merge_two_strings(&mut self, first: GoString, other: GoStringPtr) -> GoString {
        for &point in other.stones() {
            self.go_strings[self.coord_util.to(point)] = Option::None;
        }

        first.merge_with((**other).clone())
    }

    fn update_map_indexes(&mut self, go_string: GoStringPtr) {
        for &stone in go_string.stones() {
            self.go_strings[self.coord_util.to(stone)] = Some(go_string.clone());
        }
    }

    ///
    /// Remove a string from the game, it add liberties to all
    /// adjacent string of not the same color.
    ///
    pub fn remove_string(&mut self, go_string_to_remove: GoStringPtr) {
        let color_of_the_string = go_string_to_remove.color;
        for &point in go_string_to_remove.stones() {
            for neighbor_str_ptr in self.get_neighbors_strings(point).collect::<HashSet<_>>() {
                if go_string_to_remove != neighbor_str_ptr {
                    let mut neighbor_str_ptr = (**neighbor_str_ptr).clone();
                    neighbor_str_ptr.add_liberty(point);
                    self.create_string(neighbor_str_ptr)
                }
            }
            self.hash ^= ZOBRIST[(point, color_of_the_string)];

            // Remove each point from the map. The Rc will be dropped "normally".
            self.go_strings[self.coord_util.to(point)] = Option::None;
        }
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
            self.remove_string(group_of_stones);
        }
        number_of_stones_captured
    }
}

impl Display for Goban {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.pretty_string())
    }
}

impl PartialEq for Goban {
    fn eq(&self, other: &Goban) -> bool {
        other.hash == self.hash
    }
}

impl Eq for Goban {}

impl Default for Goban {
    fn default() -> Self {
        Goban::new(19)
    }
}
