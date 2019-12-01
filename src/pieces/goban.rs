//! Module with the goban and his implementations.

use crate::pieces::stones::Color::None;
use crate::pieces::stones::*;
use crate::pieces::util::coord::{corner_coords, neighbors_points, Coord, CoordUtil, Order};
use crate::pieces::zobrist::*;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::ops::{Index, IndexMut};
use std::collections::HashMap;
use crate::pieces::go_string::GoString;
use std::cell::RefCell;
use std::rc::Rc;
use by_address::ByAddress;

pub type GoStringPtr = ByAddress<Rc<RefCell<GoString>>>;

///
/// Represents a Goban. With an array with the stones encoded in u8. and the size.
/// only square boards are possible for the moment.
///
#[derive(Getters, Setters, CopyGetters, Debug)]
pub struct Goban {
    ///
    /// The values are stored in a one dimension vector.
    /// Using the RowMajor Policy.
    ///
    #[get = "pub"]
    tab: Vec<Color>,

    #[get = "pub"]
    go_strings: HashMap<Coord, GoStringPtr>,

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
            tab: vec![Color::None; size * size],
            size,
            coord_util: CoordUtil::new(size, size),
            hash: 0,
            go_strings: Default::default(),
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
            .map(|k| {
                // k.0 is the index of the coord
                // k.1 is the color
                (coord_util.from(k.0), k.1)
            })
            .filter(|s| *(*s).1 != Color::None)
            .for_each(|coord_color| {
                g.push(coord_color.0, *coord_color.1);
            });
        g
    }

    ///
    /// Put a stones in the goban. The point depends on the order choose.
    /// default (line, column)
    /// the (0,0) point is in the top left.
    ///
    pub fn push(&mut self, point: Coord, color: Color) -> &mut Goban {
        if color == Color::None {
            self.hash ^= ZOBRIST19[(point, self[point])];
        } else {
            self.hash ^= ZOBRIST19[(point, color)];
        }
        self.actualise_go_string(point, color);
        self[point] = color;
        self
    }

    ///
    /// Put many stones.
    ///
    #[inline]
    pub fn push_many(&mut self, coords: impl Iterator<Item=Coord>, value: Color) {
        coords.for_each(|c| {
            self.push(c, value);
        })
    }

    #[inline]
    pub fn push_stone(&mut self, stone: Stone) -> &mut Goban {
        self.push(stone.coordinates, stone.color)
    }

    ///
    /// Get all the neighbors to the coordinate
    ///
    #[inline]
    pub fn get_neighbors(&self, coord: Coord) -> impl Iterator<Item=Stone> + '_ {
        neighbors_points(coord)
            .into_iter()
            .filter(move |x| self.is_coord_valid(*x))
            .map(move |x| Stone {
                coordinates: x,
                color: self[x],
            })
    }

    ///
    /// Get all the stones that are neighbor to the coord except empty intersections
    ///
    #[inline]
    pub fn get_neighbors_stones(&self, coord: Coord) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(coord).filter(|s| s.color != Color::None)
    }

    ///
    /// Get all the stones except "Empty stones"
    ///
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item=Stone> + '_ {
        self.tab
            .iter()
            .enumerate()
            .filter(|(_index, t)| **t != Color::None)
            .map(move |(index, t)| Stone {
                coordinates: self.coord_util.from(index),
                color: *t,
            })
    }

    ///
    /// Get stones by their color.
    ///
    #[inline]
    pub fn get_stones_by_color(&self, color: Color) -> impl Iterator<Item=Stone> + '_ {
        self.get_points_by_color(color).map(move |c| Stone {
            color,
            coordinates: c,
        })
    }

    #[inline]
    pub fn get_points_by_color(&self, color: Color) -> impl Iterator<Item=Coord> + '_ {
        self.tab
            .iter()
            .enumerate()
            .filter(move |(_index, &t)| t == color)
            .map(move |(index, _t)| self.coord_util.from(index))
    }

    ///
    /// Returns the empty stones connected to the stone
    ///
    #[inline]
    pub fn get_liberties(&self, stone: Stone) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors(stone.coordinates)
            .filter(|s| s.color == Color::None)
    }

    ///
    /// Returns the number of liberties of the stone.
    ///
    #[inline]
    pub fn get_nb_liberties(&self, point: Stone) -> u8 {
        self.get_liberties(point).count() as u8
    }

    ///
    /// Returns true if the stone has liberties.
    ///
    #[inline]
    pub fn has_liberties(&self, point: Stone) -> bool {
        self.get_liberties(point).any(|s| Color::None == s.color)
    }

    ///
    /// Get a string for printing the goban in normal shape (0,0) left bottom
    ///
    pub fn pretty_string(&self) -> String {
        let mut buff = String::new();
        for i in 0..self.size {
            buff.push('|');
            for j in 0..self.size {
                buff.push(match self[(i, j)] {
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
    pub fn is_point_an_eye(&self, point: Coord, color: Color) -> bool {
        if self[point] != None {
            return false;
        }
        if self.get_neighbors(point).any(|stone| stone.color != color) {
            return false;
        }
        let mut corner_ally = 0;
        let mut corner_off_board = 0;
        for p in corner_coords(point) {
            if self.is_coord_valid(p) {
                if self[p] == color {
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
    pub fn is_coord_valid(&self, coord: Coord) -> bool {
        coord.0 < self.size && coord.1 < self.size
    }

    ///
    /// Just create the Rc pointer and add it to the set.
    /// moves out the string.
    ///
    fn create_string(&mut self, string_to_add: GoString) {
        let new_string: GoStringPtr = Rc::new(RefCell::new(string_to_add)).into();
        self.replace_string(new_string);
    }

    ///
    /// Deletes all the Rc from the go_strings set then merges the two_string
    ///
    fn merge_two_strings(&mut self, first: GoString, other: GoStringPtr) -> GoString {
        for point in other.borrow().stones() {
            self.go_strings.remove(&point);
        }

        first.merge_with(other.borrow().clone())
    }

    fn replace_string(&mut self, go_string: GoStringPtr) {
        for &stone in go_string.borrow().stones() {
            self.go_strings.insert(stone, go_string.clone());
        }
    }

    ///
    /// Remove a string from the game, then return the string removed, it add liberties to all
    /// adjacents string of not the same color.
    ///
    pub fn remove_string(&mut self, go_string_to_remove: GoStringPtr) {
        for &stone in go_string_to_remove.borrow().stones() {
            let neighbor_points: Vec<_> = neighbors_points(stone)
                .into_iter()
                .filter(|&coord| self.is_coord_valid(coord))
                .collect();
            for neighbor_point in neighbor_points {
                match self.go_strings.get(&neighbor_point)
                    .map(ToOwned::to_owned) {
                    Some(ptr_neighbor_string) => {
                        if ptr_neighbor_string != go_string_to_remove {
                            ptr_neighbor_string.borrow_mut().add_liberty(stone);
                            self.replace_string(ptr_neighbor_string)
                        }
                    }
                    _ => ()
                }
            }
            // Remove the key from the map. The Rc will be dropped.
            self.go_strings.remove(&stone);
        }
    }

    fn actualise_go_string(&mut self, point: Coord, color: Color) {
        if color == Color::None {
            return;
        }
        let mut liberties = hashset! {};
        let mut adjacent_same_color_set = hashset! {};
        let mut adjacent_opposite_color_set = hashset! {};

        for p in neighbors_points(point)
            .into_iter()
            .filter(|&x| self.is_coord_valid(x)) {
            match self.go_strings.get(&p) {
                Some(s) => {
                    match s.borrow().color() {
                        c if c == color => {
                            adjacent_same_color_set.insert(s.to_owned());
                        }
                        Color::None => panic!("a string cannot be of color none"),
                        _ => {
                            adjacent_opposite_color_set.insert(s.to_owned());
                        }
                    }
                }
                Option::None => {
                    { liberties.insert(p); }
                }
            }
        }

        // Merges the neighbors allies string and then creates the string
        let new_string = adjacent_same_color_set
            .drain()
            .fold(GoString::new(
                color,
                hashset! {point},
                liberties,
            ),
                  |go_string, same_color_string| {
                      self.merge_two_strings(go_string,
                                             same_color_string)
                  });
        self.create_string(new_string);
        for other_color_string in adjacent_opposite_color_set.drain() {
            other_color_string.borrow_mut().remove_liberty(point);
            if other_color_string.borrow().is_dead() {
                // self.remove_string(other_color_string);
            } else {
                self.replace_string(other_color_string);
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
        other.hash == self.hash
    }
}

impl Eq for Goban {}

impl Index<Coord> for Goban {
    type Output = Color;

    fn index(&self, index: Coord) -> &Self::Output {
        &self.tab[self.coord_util.to(index)]
    }
}

impl IndexMut<Coord> for Goban {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        &mut self.tab[self.coord_util.to(index)]
    }
}

impl Default for Goban {
    fn default() -> Self {
        Goban::new(19)
    }
}

impl Clone for Goban {
    fn clone(&self) -> Self {
        let go_strings = self.go_strings.iter()
            .map(|(&key, go_str_ptr)| (key, go_str_ptr.borrow().clone()))
            .map(|(key, go_str)| (key, ByAddress(Rc::new(RefCell::new(go_str)))))
            .collect();
        Goban {
            tab: self.tab.clone(),
            go_strings,
            size: self.size,
            coord_util: self.coord_util,
            hash: self.hash,
        }
    }
}
