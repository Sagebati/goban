//! Module with the goban and his implementations.

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use arrayvec::ArrayVec;
use bitvec::{BitArr, bitarr};

use crate::pieces::chain::Chain;
use crate::pieces::Nat;
use crate::pieces::stones::*;
use crate::pieces::util::CircularRenIter;
use crate::pieces::util::coord::{
    is_coord_valid, neighbor_points, one_to_2dim, Point, two_to_1dim,
};
use crate::pieces::zobrist::*;

pub type ChainIdx = usize;
pub type BoardIdx = usize;

macro_rules! iter_stones {
    ($goban: expr, $ren_idx: expr) => {
        CircularRenIter::new($goban.chains[$ren_idx].origin, &$goban.next_stone)
    };
}

/// Represents a Goban. the stones are stored in ROW MAJOR (row, column)
#[derive(Getters, Setters, CopyGetters, Debug, Clone)]
pub struct Goban {
    #[get = "pub"]
    pub(super) chains: Vec<Chain>,
    board: Vec<Option<ChainIdx>>,
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
            chains: Vec::with_capacity(128),
            free_slots: Vec::with_capacity(32),
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
            .map(|point| point.map_or(Color::None, |go_str_ptr| self.chains[go_str_ptr].color))
            .collect()
    }

    /// Like vec but in a matrix shape.
    pub fn matrix(&self) -> Vec<Vec<Color>> {
        let mut mat = vec![vec![]];
        for line in self.board.chunks_exact(self.size.1) {
            let v = line
                .iter()
                .map(|o| o.map_or(Color::None, |idx| self.chains[idx].color))
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
    pub(crate) fn board(&self) -> &[Option<ChainIdx>] {
        &self.board
    }

    /// pushes the stone
    /// # Arguments
    /// point: the point where the stone will be placed
    /// color: the color of the stone must be != empty
    /// # Returns
    /// A tuple with (the ren without liberties, the ren where the point was added)
    pub(crate) fn push_wth_feedback(
        &mut self,
        point: Point,
        color: Color,
    ) -> (ArrayVec<usize, 4>, ChainIdx) {
        let pushed_stone_idx = two_to_1dim(self.size, point);

        let mut adjacent_same_color_str_set = ArrayVec::<_, 4>::new();
        let mut adjacent_opposite_color_str_set = ArrayVec::<_, 4>::new();
        let mut liberties = bitarr!(0;361);

        for neighbor_idx in self.neighbor_points_indexes(pushed_stone_idx) {
            match self.board[neighbor_idx] {
                Some(adj_ren_index) => {
                    if self.chains[adj_ren_index].color == color {
                        if !adjacent_same_color_str_set.contains(&adj_ren_index) {
                            adjacent_same_color_str_set.push(adj_ren_index);
                        }
                    } else if !adjacent_opposite_color_str_set.contains(&adj_ren_index) {
                        adjacent_opposite_color_str_set.push(adj_ren_index);
                    }
                }
                Option::None => {
                    liberties.set(neighbor_idx, true);
                }
            }
        }

        let mut dead_ren = ArrayVec::<_, 4>::new();
        // for every string of opposite color remove a liberty and update the string.
        for ren_idx in adjacent_opposite_color_str_set {
            let ren = &mut self.chains[ren_idx];
            ren.remove_liberty(pushed_stone_idx);
            if ren.is_dead() {
                dead_ren.push(ren_idx);
            }
        }

        let number_of_neighbors_strings = adjacent_same_color_str_set.len();
        let updated_ren_index = match number_of_neighbors_strings {
            0 => self.create_chain(pushed_stone_idx, color, liberties),
            1 => {
                let only_ren_idx = adjacent_same_color_str_set.into_iter().next().unwrap();

                self.chains[only_ren_idx]
                    .remove_liberty(pushed_stone_idx)
                    .add_liberties_owned(liberties);
                self.add_stone_to_chain(only_ren_idx, pushed_stone_idx);
                self.board[pushed_stone_idx] = Some(only_ren_idx);
                only_ren_idx
            }
            _ => {
                let mut to_merge = self.create_chain(pushed_stone_idx, color, liberties);
                for adj_ren in adjacent_same_color_str_set {
                    if self.chains[adj_ren].number_of_liberties()
                        < self.chains[to_merge].number_of_liberties()
                    {
                        self.merge_strings(to_merge, adj_ren);
                    } else {
                        self.merge_strings(adj_ren, to_merge);
                        to_merge = adj_ren;
                    }
                }
                self.chains[to_merge].remove_liberty(pushed_stone_idx);
                to_merge
            }
        };
        self.zobrist_hash ^= index_zobrist(pushed_stone_idx, color);
        (dead_ren, updated_ren_index)
    }

    /// Put a stones in the goban.
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

    /// Get the chain from their id
    pub fn get_chain_from_id(&self, ren_idx: ChainIdx) -> &Chain {
        &self.chains[ren_idx]
    }

    pub fn get_chain_by_board_idx(&self, board_idx: BoardIdx) -> Option<&Chain> {
        self.board[board_idx].map(|chain| &self.chains[chain])
    }

    pub fn get_chain_by_point(&self, point: Point) -> Option<&Chain> {
        self.get_chain_by_board_idx(two_to_1dim(self.size, point))
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

    /// Get all the neighbors indexes to the point. Only return point with a color.
    #[inline]
    pub fn get_neighbors_chain_indexes(&self, coord: Point) -> impl Iterator<Item=ChainIdx> + '_ {
        self.neighbor_points(coord)
            .map(move |point| two_to_1dim(self.size, point))
            .into_iter()
            .filter_map(move |point| self.board[point])
    }

    /// Get all the chains adjacent to the point. The result iterator can contains duplicates.
    #[inline]
    pub fn get_neighbors_chains(&self, coord: Point) -> impl Iterator<Item=&Chain> + '_ {
        self.get_neighbors_chain_indexes(coord)
            .map(move |chain_idx| &self.chains[chain_idx])
    }

    #[inline]
    pub fn get_neighbors_strings_indices_by_idx(
        &self,
        index: usize,
    ) -> impl Iterator<Item=ChainIdx> + '_ {
        self.neighbor_points_indexes(index)
            .filter_map(move |idx| self.board[idx])
    }

    /// Function for getting the stone in the goban.
    #[inline(always)]
    pub fn get_stone(&self, point: Point) -> Color {
        self.board[two_to_1dim(self.size, point)]
            .map_or(Color::None, |go_str_index| self.chains[go_str_index].color)
    }

    /// Get all the stones except "Empty stones"
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item=Stone> + '_ {
        self.board.iter().enumerate().filter_map(move |(index, o)| {
            o.map(move |ren_index| Stone {
                coordinates: one_to_2dim(self.size, index),
                color: self.chains[ren_index].color,
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
                option.map(move |ren_idx| self.chains[ren_idx].color)
                    .unwrap_or(Color::None) == color
            })
            .map(move |(index, _)| one_to_2dim(size, index))
    }

    /// Returns the "empty" stones connected to the stone
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
    /// adjacent chains that aren't the same color.
    pub fn remove_chain(&mut self, ren_to_remove_idx: ChainIdx) {
        let color_of_the_string = self.chains[ren_to_remove_idx].color;
        let mut neighbors = ArrayVec::<_, 4>::new();

        for point_idx in iter_stones!(self, ren_to_remove_idx) {
            for neighbor_str_idx in self
                .get_neighbors_strings_indices_by_idx(point_idx) {
                if ren_to_remove_idx != neighbor_str_idx {
                    #[cfg(debug_assertions)]
                    if !neighbors.contains(&neighbor_str_idx) {
                        neighbors.push(neighbor_str_idx)
                    }
                    #[cfg(not(debug_assertions))]
                        neighbors.push(neighbor_str_idx)
                }
            }

            for &n in &neighbors {
                self.chains[n].add_liberty(point_idx);
            }
            neighbors.clear();
            self.zobrist_hash ^= index_zobrist(point_idx, color_of_the_string);
            self.board[point_idx] = Option::None;
        }

        self.put_chain_in_bin(ren_to_remove_idx);
    }

    /// Updates the indexes to match actual goban. must use after we put a stone.
    fn update_chain_indexes_in_board(&mut self, ren_idx: ChainIdx) {
        debug_assert_eq!(
            iter_stones!(self, ren_idx).last().unwrap(),
            self.chains[ren_idx].last
        );
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
        neighbor_points(point).into_iter().filter(move |&p| is_coord_valid(size, p))
    }

    #[inline]
    fn neighbor_points_indexes(&self, board_idx: BoardIdx) -> impl Iterator<Item=usize> {
        let size = self.size;
        self.neighbor_points(one_to_2dim(self.size, board_idx))
            .map(move |x| two_to_1dim(size, x))
    }

    pub fn get_chain_it(&self, chain_idx: ChainIdx) -> impl Iterator<Item=usize> + '_ {
        CircularRenIter::new(self.chains[chain_idx].origin, &self.next_stone)
    }

    #[inline]
    pub fn get_chain_it_by_board_idx(&self, board_idx: BoardIdx) -> impl Iterator<Item=usize> + '_ {
        self.board[board_idx].map(|chain_idx| self.get_chain_it(chain_idx)).unwrap_or_else(|| panic!("The board index: {} was out of bounds", board_idx))
    }

    #[inline]
    fn create_chain(&mut self, origin: usize, color: Color, liberties: BitArr!(for 361)) -> ChainIdx {
        let chain_to_place = Chain::new_with_liberties(color, origin, liberties);

        self.next_stone[origin] = origin;
        let ren_index = if let Some(free_slot_idx) = self.free_slots.pop() {
            self.chains[free_slot_idx] = chain_to_place;
            free_slot_idx
        } else {
            self.chains.push(chain_to_place);
            self.chains.len() - 1
        };
        self.update_chain_indexes_in_board(ren_index);
        ren_index
    }

    fn add_stone_to_chain(&mut self, chain_idx: usize, stone: usize) {
        let chain = &mut self.chains[chain_idx];
        if stone < chain.origin {
            // replace origin
            self.next_stone[stone] = chain.origin;
            self.next_stone[chain.last] = stone;
            chain.origin = stone;
        } else {
            self.next_stone[chain.last] = stone;
            self.next_stone[stone] = chain.origin;
            chain.last = stone;
        }
        chain.num_stones += 1;
        debug_assert_eq!(
            iter_stones!(self, chain_idx).last().unwrap(),
            self.chains[chain_idx].last
        );
    }

    fn merge_strings(&mut self, chain1_idx: ChainIdx, chain2_idx: ChainIdx) {
        debug_assert_eq!(
            self.chains[chain1_idx].color, self.chains[chain2_idx].color,
            "Cannot merge two strings of different color"
        );
        debug_assert_ne!(chain1_idx, chain2_idx, "merging the same string");

        let (chain1, chain2) = if chain1_idx < chain2_idx {
            let (s1, s2) = self.chains.split_at_mut(chain2_idx);
            (&mut s1[chain1_idx], s2.first_mut().unwrap())
        } else {
            // ren2_idx > ren1_idx
            let (contains_chain2, contains_ren1) = self.chains.split_at_mut(chain1_idx);
            (
                contains_ren1.first_mut().unwrap(),
                &mut contains_chain2[chain2_idx],
            )
        };
        chain1.liberties |= chain2.liberties;

        let chain1_last = chain1.last;
        let chain2_last = chain2.last;

        let chain1_origin = chain1.origin;
        let chain2_origin = chain2.origin;

        if chain1_origin > chain2_origin {
            chain1.origin = chain2_origin;
        } else {
            chain1.last = chain2_last;
        }
        self.next_stone.swap(chain1_last, chain2_last);
        chain1.num_stones += chain2.num_stones;

        self.update_chain_indexes_in_board(chain1_idx);
        self.put_chain_in_bin(chain2_idx);
    }

    #[inline]
    fn put_chain_in_bin(&mut self, ren_idx: ChainIdx) {
        self.chains[ren_idx].used = false;
        self.free_slots.push(ren_idx);
    }

    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    fn check_integrity_ren(&self, ren_idx: ChainIdx) {
        assert_eq!(
            iter_stones!(self, ren_idx).next().unwrap(),
            self.chains[ren_idx].origin,
            "The origin doesn't match"
        );
        assert_eq!(
            iter_stones!(self, ren_idx).last().unwrap(),
            self.chains[ren_idx].last,
            "The last doesn't match"
        );
        if iter_stones!(self, ren_idx).count() as u16 != self.chains[ren_idx].num_stones {
            panic!("The number of stones don't match")
        }
    }

    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    fn check_integrity_all(&self) {
        for ren_idx in (0..self.chains.len()).filter(|&ren_idx| self.chains[ren_idx].used) {
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
