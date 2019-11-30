//! Module with the goban and his implementations.

use crate::pieces::stones::Color::None;
use crate::pieces::stones::*;
use crate::pieces::util::coord::{corner_coords, neighbors_points, Coord, CoordUtil, Order};
use crate::pieces::zobrist::*;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::ops::{Index, IndexMut};
use std::collections::{HashSet, HashMap};
use crate::pieces::go_string::GoString;
use std::cell::RefCell;
use std::rc::Rc;
use by_address::ByAddress;

type GoStringPtr = ByAddress<Rc<RefCell<GoString>>>;

///
/// Represents a Goban. With an array with the stones encoded in u8. and the size.
/// only square boards are possible for the moment.
///
#[derive(Clone, Getters, Setters, CopyGetters, Debug)]
pub struct Goban {
    ///
    /// The values are stored in a one dimension vector.
    /// Using the RowMajor Policy.
    ///
    #[get = "pub"]
    tab: Vec<Color>,

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
            go_strings: HashMap::default(),
            size,
            coord_util: CoordUtil::new(size, size),
            hash: 0,
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
                g.push(coord_color.0, *coord_color.1)
                    .expect("Play the stone");
            });
        g
    }

    ///
    /// Put a stones in the goban. The point depends on the order choose.
    /// default (line, column)
    /// the (0,0) point is in the top left.
    ///
    pub fn push(&mut self, point: Coord, color: Color) -> Result<&mut Goban, String> {
        if self.is_coord_valid(point) {
            if color == Color::None {
                self.hash ^= ZOBRIST19[(point, self[point])];
            } else {
                self.hash ^= ZOBRIST19[(point, color)];
            }
            self.actualise_go_string(point, color);
            self[point] = color;
            Ok(self)
        } else {
            Err(format!(
                "the point :({},{}) are outside the goban",
                point.0, point.1
            ))
        }
    }

    fn actualise_go_string(&mut self, point: Coord, color: Color) {
        let mut liberties: HashSet<Coord> = Default::default();
        let mut adjacent_same_color_set = HashSet::new();
        let mut adjacent_opposite_color_set = HashSet::new();

        for p in neighbors_points(point) {
            match self.go_strings.get(&p){
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
        let mut stone = HashSet::new();
        stone.insert(point);
        let mut new_string = GoString::new(
            color,
            stone,
            liberties,
        );
        for same_color_string in adjacent_same_color_set.drain() {
            let removed_string = self.remove_string(same_color_string.clone());
            new_string = new_string.merge_with(removed_string)
        }

        let new_string: GoStringPtr = Rc::new(RefCell::new(new_string)).into();
        self.replace_string(new_string);

        for other_color_string in adjacent_opposite_color_set.drain(){
            other_color_string.borrow_mut().remove_liberty(point);
            if other_color_string.borrow().is_dead() {
                self.remove_string(other_color_string);
            }else {
                self.replace_string(other_color_string);
            }
        }
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
    fn remove_string(&mut self, go_string_to_remove: GoStringPtr) -> GoString {
        for &stone in go_string_to_remove.borrow().stones() {
            for neighbor_point in neighbors_points(stone) {
                let some = self.go_strings.get(&neighbor_point).is_some();
                match self.go_strings.get(&neighbor_point).map(|s|ToOwned::to_owned(s)) {
                    Some(ptr_neighbor_string) => {
                        if ptr_neighbor_string != go_string_to_remove {
                            ptr_neighbor_string.borrow_mut().add_liberty(neighbor_point);
                            self.replace_string(ptr_neighbor_string)
                        }
                    }
                    Option::None => {}
                }

                if some {}
            }
            // removing hash from the stones
            self.hash ^= ZOBRIST19[(stone, RefCell::borrow(&go_string_to_remove).color())];
            self.go_strings.remove(&stone);
        }
        go_string_to_remove.borrow().clone()
    }

    ///
    /// Put many stones.
    ///
    #[inline]
    pub fn push_many(&mut self, coords: impl Iterator<Item=Coord>, value: Color) {
        coords.for_each(|c| {
            self.push(c, value)
                .expect("Add one of the stones to the goban.");
        })
    }

    #[inline]
    pub fn push_stone(&mut self, stone: Stone) -> Result<&mut Goban, String> {
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
    fn is_coord_valid(&self, coord: Coord) -> bool {
        coord.0 < self.size && coord.1 < self.size
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
