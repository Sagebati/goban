//! Module with the goban and his implementations.

use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use arrayvec::ArrayVec;

use crate::one2dim;
use crate::pieces::chain::{Chain, Liberties, merge, set};
use crate::pieces::{Nat, Neighbors};
use crate::pieces::stones::*;
use crate::pieces::util::CircularRenIter;
use crate::pieces::util::coord::{
    Coord, is_coord_valid, neighbor_coords, one_to_2dim, Size, two_to_1dim,
};
use crate::pieces::zobrist::*;

pub type ChainIdx = usize;
pub type BoardIdx = usize;

const BOARD_MAX_SIZE: (Nat, Nat) = (19, 19);
const BOARD_MAX_LENGTH: usize = BOARD_MAX_SIZE.0 as usize * BOARD_MAX_SIZE.1 as usize;
const MAX_CHAINS: usize = 4 * BOARD_MAX_LENGTH / 5;

macro_rules! iter_stones {
    ($goban: expr, $ren_idx: expr) => {
        CircularRenIter::new(
            $goban.chains[$ren_idx as usize].origin as usize,
            &$goban.next_stone,
        )
    };
}

/// Represents a goban. the stones are stored in ROW MAJOR (row, column)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goban {
    pub(super) chains: Vec<Chain>,
    //free_slots: BitArr!(for MAX_CHAINS),
    /// The board contains indexes of the vec chains
    board: Vec<Option<u16>>,
    next_stone: Vec<u16>,
    size: Size,
    zobrist_hash: u64,
}

impl From<&[MaybeColor]> for Goban {
    fn from(stones: &[MaybeColor]) -> Self {
        let size = (stones.len() as f32).sqrt() as u8;
        let mut game = Goban::new((size, size));
        stones
            .iter()
            .enumerate()
            .map(|(index, color)| (one_to_2dim((size, size), index), color))
            .filter_map(|(coord, mcolor)| mcolor.map(|color| (coord, color)))
            .for_each(|coord_color| {
                game.put(coord_color.0, coord_color.1);
            });
        game
    }
}

impl Goban {
    /// Creates a Goban
    /// # Arguments
    ///
    /// * `(height, width)` a tuple with the height and the width of the desired goban.
    pub fn new((height, width): Size) -> Self {
        assert!(height <= 19 && width <= 19, "We don't handle board > 19 for the moment");
        Goban {
            size: (height, width),
            zobrist_hash: 0,
            board: vec![None; BOARD_MAX_LENGTH],
            next_stone: vec![0; BOARD_MAX_LENGTH],
            chains: Vec::with_capacity(MAX_CHAINS),
            //free_slots: Default::default(),
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.zobrist_hash
    }

    /// Returns the underlying goban in a vector with a RowMajor Policy, calculated on the fly.
    pub fn to_vec(&self) -> Vec<MaybeColor> {
        self.board
            .iter()
            .map(|point| {
                point.map_or(EMPTY, |go_str_ptr| {
                    self.chains[go_str_ptr as usize].color.into()
                })
            })
            .collect()
    }

    /// Like vec but in a matrix shape.
    pub fn matrix(&self) -> Vec<Vec<MaybeColor>> {
        let mut mat = vec![vec![]];
        for line in self.board.chunks_exact(self.size.1 as usize) {
            let v = line
                .iter()
                .map(|o| o.map_or(EMPTY, |idx| self.chains[idx as usize].color.into()))
                .collect();
            mat.push(v);
        }
        mat
    }

    /// Get number of stones on the goban.
    /// (number of black stones, number of white stones)
    pub fn number_of_stones(&self) -> (u32, u32) {
        let mut black_stones = 0;
        let mut white_stones = 0;

        for chain in &self.chains {
            if !chain.is_dead() {
                match chain.color {
                    Color::White => { white_stones += chain.num_stones as u32; }
                    Color::Black => { black_stones += chain.num_stones as u32; }
                }
            }
        }

        (black_stones, white_stones)
    }

    #[inline]
    pub(crate) fn board(&self) -> &[Option<u16>] {
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
        point: Coord,
        color: Color,
    ) -> (ArrayVec<usize, 4>, ChainIdx) {
        let pushed_stone_idx = two_to_1dim(self.size, point);

        let mut adjacent_same_color_str_set = ArrayVec::<BoardIdx, 4>::new();
        let mut adjacent_opposite_color_str_set = ArrayVec::<BoardIdx, 4>::new();
        let mut liberties = ArrayVec::<BoardIdx, 4>::new();

        // For each neighbor we fill the right vector
        for neighbor_idx in self.neighbors_idx(pushed_stone_idx) {
            match self.board[neighbor_idx] {
                Some(adj_ren_index) => {
                    let adj_ren_index = adj_ren_index as usize;
                    if self.chains[adj_ren_index].color == color {
                        if !adjacent_same_color_str_set.contains(&adj_ren_index) {
                            adjacent_same_color_str_set.push(adj_ren_index);
                        }
                    } else if !adjacent_opposite_color_str_set.contains(&adj_ren_index) {
                        adjacent_opposite_color_str_set.push(adj_ren_index);
                    }
                }
                None => {
                    liberties.push(neighbor_idx);
                }
            }
        }

        let mut dead_ren = ArrayVec::<BoardIdx, 4>::new();
        // for every string of opposite color remove a liberty and update the string.
        for ren_idx in adjacent_opposite_color_str_set {
            let ren = &mut self.chains[ren_idx];
            if ren.used {
                ren.remove_liberty(pushed_stone_idx);
                if ren.is_dead() {
                    dead_ren.push(ren_idx);
                }
            }
        }

        let number_of_neighbors_strings = adjacent_same_color_str_set.len();
        let updated_ren_index = match number_of_neighbors_strings {
            0 => self.create_chain(pushed_stone_idx, color, &liberties),
            1 => {
                let only_ren_idx = adjacent_same_color_str_set[0];

                self.chains[only_ren_idx]
                    .remove_liberty(pushed_stone_idx)
                    .union_liberties_slice(&liberties);
                self.add_stone_to_chain(only_ren_idx, pushed_stone_idx);
                self.board[pushed_stone_idx] = Some(only_ren_idx as u16);
                only_ren_idx
            }
            _ => {
                let mut to_merge = self.create_chain(pushed_stone_idx, color, &liberties);
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
        #[cfg(debug_assertions)]
        self.check_integrity_all();
        (dead_ren, updated_ren_index)
    }

    pub(crate) fn remove_captured_stones_aux(
        &mut self,
        suicide_allowed: bool,
        dead_rens_indices: &[ChainIdx],
        added_chain: ChainIdx,
    ) -> ((u32, u32), Option<Coord>) {
        let only_one_ren_removed = dead_rens_indices.len() == 1;
        let mut stones_removed = (0, 0);
        let mut ko_point = None;
        for &dead_ren_idx in dead_rens_indices {
            let dead_chain = &self.chains[dead_ren_idx];
            // If only one stone and one ren is removed then it becomes a ko point
            if dead_chain.num_stones == 1 && only_one_ren_removed {
                ko_point = Some(one_to_2dim(self.size(), dead_chain.origin as usize));
            }
            match dead_chain.color {
                Color::White => { stones_removed.0 += dead_chain.num_stones as u32; }
                Color::Black => { stones_removed.1 += dead_chain.num_stones as u32; }
            }
            self.remove_chain(dead_ren_idx);
        }

        let maybe_dead_chain = &self.chains[added_chain];
        if suicide_allowed && maybe_dead_chain.is_dead() {
            match maybe_dead_chain.color {
                Color::White => { stones_removed.0 += maybe_dead_chain.num_stones as u32; }
                Color::Black => { stones_removed.1 += maybe_dead_chain.num_stones as u32; }
            }
            ko_point = None;
            self.remove_chain(added_chain);
        }
        (stones_removed, ko_point)
    }

    /// Put a stones in the goban.
    /// default (line, column)
    /// the (0,0) point is in the top left.
    ///
    /// # Panics
    /// if the point is out of bounds
    pub fn put(&mut self, point: Coord, color: Color) -> &mut Self {
        debug_assert!(
            (point.0) < self.size.0,
            "Coordinate point.0 {} out of bounds",
            point.0
        );
        debug_assert!(
            (point.1) < self.size.1,
            "Coordinate point.1 {} out of bounds",
            point.1
        );
        self.push_wth_feedback(point, color);
        self
    }

    /// Helper function to put a stone.
    #[inline]
    pub fn put_stone(&mut self, stone: Stone) -> &mut Goban {
        self.put(stone.coord, stone.color)
    }

    /// Put many stones.
    #[inline]
    pub fn put_many(&mut self, points: &[Coord], value: Color) {
        points.iter().for_each(|&point| {
            self.put(point, value);
        })
    }

    /// Get all the neighbors to the coordinate including empty intersections.
    #[inline]
    pub fn get_neighbors_points(&self, point: Coord) -> impl Iterator<Item=Point> + '_ {
        self.neighbors_coords(point).map(move |p| Point {
            coord: p,
            color: self.get_color(p),
        })
    }

    /// Get all the stones that are neighbor to the coord except empty intersections.
    #[inline]
    pub fn get_neighbors_stones(&self, point: Coord) -> impl Iterator<Item=Stone> + '_ {
        self.get_neighbors_points(point).filter_map(|x| Some(x.into()).filter(|_| x.is_empty()))
    }

    /// Get all the chains adjacent to the point. The result iterator can contains duplicates.
    #[inline]
    pub fn get_neighbors_chains(&self, coord: Coord) -> Neighbors<&Chain> {
        self.get_neighbors_chains_idx(two_to_1dim(self.size,coord)).into_iter().map(|chain_idx| &self.chains[chain_idx]).collect()
    }

    #[inline]
    pub fn get_neighbors_chains_idx(
        &self,
        index: BoardIdx,
    ) -> Neighbors<ChainIdx> {
        let mut array_vec: ArrayVec<ChainIdx, 4> = ArrayVec::new_const();
        for idx in self.neighbors_idx(index) {
            if let Some(idx) = self.board[idx] {
                if !array_vec.contains(&(idx as usize)) {
                    array_vec.push(idx as usize);
                }
            }
        }
        array_vec
    }

    pub fn get_color(&self, coord: Coord) -> MaybeColor {
        self.board[two_to_1dim(self.size, coord)]
            .map(|chain_id| self.chains[chain_id as usize].color)
    }

    pub fn get_stone_color(&self, coord: Coord) -> Color {
        self.get_color(coord).expect("Tried to unwrap an empty point")
    }

    /// Get all the stones except "EMPTY stones"
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item=Stone> + '_ {
        self.board.iter().enumerate().filter_map(move |(index, o)| {
            o.map(move |chain_idx| Stone {
                coord: one_to_2dim(self.size, index),
                color: self.chains[chain_idx as usize].color,
            })
        })
    }

    /// Get stones by their color.
    #[inline]
    pub fn get_stones_by_color(&self, color: MaybeColor) -> impl Iterator<Item=Point> + '_ {
        self.get_coords_by_color(color)
            .map(move |c| Point { color, coord: c })
    }

    pub fn get_empty_idx(&self) -> impl Iterator<Item=BoardIdx> + '_ {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(idx, chain)| chain.map(|_| idx))
    }

    pub fn get_empty_coords(&self) -> impl Iterator<Item=Coord> + '_ {
        let board_length = self.size.0 as usize * self.size.1 as usize;
        self.board[..board_length]
            .iter()
            .enumerate()
            .filter_map(|x| {
                if x.1.is_none() {
                    Some(one2dim!(self.size, x.0))
                } else {
                    None
                }
            })
    }

    /// Get points by their color.
    #[inline]
    pub fn get_coords_by_color(&self, color: MaybeColor) -> impl Iterator<Item=Coord> + '_ {
        let mut res = ArrayVec::<Coord, BOARD_MAX_LENGTH>::new();
        for board_idx in 0..(self.size.0 * self.size.1) as usize {
            match color {
                EMPTY => res.push(one_to_2dim(self.size, board_idx)),
                Some(c) => self.board[board_idx]
                    .filter(|&chain_idx| self.chains[chain_idx as usize].color == c)
                    .map(|_| res.push(one_to_2dim(self.size, board_idx)))
                    .unwrap_or(()),
            }
        }
        res.into_iter()
    }

    /// Returns the "empty" stones connected to the stone
    #[inline]
    pub fn get_liberties(&self, coord: Coord) -> impl Iterator<Item=Coord> + '_ {
        self.neighbors_coords(coord).filter(|&x| self.get_color(x).is_none())
    }

    /// Returns true if the stone has liberties.
    #[inline]
    pub fn has_liberties(&self, coord: Coord) -> bool {
        self.get_liberties(coord).next().is_some()
    }

    /// Get a string for printing the goban in normal shape (0,0) left bottom
    pub fn pretty_string(&self) -> String {
        let mut buff = String::with_capacity(361);
        for i in 0..self.size.0 as Nat {
            for j in 0..self.size.1 as Nat {
                buff.push(match self.get_color((i, j)) {
                    Some(Color::Black) => '●',
                    Some(Color::White) => '○',
                    EMPTY => {
                        match (
                            i == 0,
                            i == self.size.0 as Nat - 1,
                            j == 0,
                            j == self.size.1 as Nat - 1,
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
        for point_idx in iter_stones!(self, ren_to_remove_idx as u16) {
            let mut neighbors_chains = self.get_neighbors_chains_idx(point_idx);
            // We remove our chain from the neighbors
            neighbors_chains.retain(|x| *x != ren_to_remove_idx);

            for &n in &neighbors_chains {
                self.chains[n].add_liberty(point_idx);
            }
            self.zobrist_hash ^= index_zobrist(point_idx, color_of_the_string);
            self.board[point_idx] = None;
        }
        self.put_chain_in_bin(ren_to_remove_idx);
    }

    /// Updates the indexes to match actual goban. must use after we put a stone.
    fn update_chain_indexes_in_board(&mut self, ren_idx: ChainIdx) {
        debug_assert_eq!(
            iter_stones!(self, ren_idx).last().unwrap() as u16,
            self.chains[ren_idx].last
        );
        for point in iter_stones!(self, ren_idx) {
            unsafe {
                *self.board.get_unchecked_mut(point) = Some(ren_idx as u16);
            }
        }
    }

    /// Get the neighbors points of a point.
    #[inline]
    fn neighbors_coords(&self, coord: Coord) -> impl Iterator<Item=Coord> {
        let size = self.size;
        neighbor_coords(coord)
            .into_iter()
            .filter(move |&p| is_coord_valid(size, p))
    }

    #[inline]
    fn neighbors_idx(&self, board_idx: BoardIdx) -> impl Iterator<Item=BoardIdx> {
        let size = self.size;
        self.neighbors_coords(one_to_2dim(size, board_idx))
            .map(move |coord| two_to_1dim(size, coord))
    }

    #[inline]
    fn create_chain(&mut self, origin: BoardIdx, color: Color, liberties: &[BoardIdx]) -> ChainIdx {
        let mut lib_bitvec: Liberties = Default::default();
        for &board_idx in liberties {
            set::<true>(board_idx, &mut lib_bitvec);
        }
        let chain_to_place = Chain::new_with_liberties(color, origin, lib_bitvec);
        self.next_stone[origin] = origin as u16;
        self.chains.push(chain_to_place);
        let chain_idx = self.chains.len() - 1;
        self.update_chain_indexes_in_board(chain_idx);
        chain_idx
    }

    fn add_stone_to_chain(&mut self, chain_idx: ChainIdx, stone: BoardIdx) {
        let chain = &mut self.chains[chain_idx];
        if stone < chain.origin as usize {
            // replace origin
            self.next_stone[stone] = chain.origin;
            self.next_stone[chain.last as usize] = stone as u16;
            chain.origin = stone as u16;
        } else {
            self.next_stone[chain.last as usize] = stone as u16;
            self.next_stone[stone] = chain.origin;
            chain.last = stone as u16;
        }
        chain.num_stones += 1;
        debug_assert_eq!(
            iter_stones!(self, chain_idx).last().unwrap() as u16,
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
        merge(&mut chain1.liberties, &chain2.liberties);

        let chain1_last = chain1.last;
        let chain2_last = chain2.last;

        let chain1_origin = chain1.origin;
        let chain2_origin = chain2.origin;

        if chain1_origin > chain2_origin {
            chain1.origin = chain2_origin;
        } else {
            chain1.last = chain2_last;
        }
        self.next_stone
            .swap(chain1_last as usize, chain2_last as usize);
        chain1.num_stones += chain2.num_stones;

        self.update_chain_indexes_in_board(chain1_idx);
        self.put_chain_in_bin(chain2_idx);
    }

    #[inline]
    fn put_chain_in_bin(&mut self, ren_idx: ChainIdx) {
        self.chains[ren_idx].used = false;
        //self.free_slots.set(ren_idx, true);
    }

    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    fn check_integrity_ren(&self, ren_idx: ChainIdx) {
        assert_eq!(
            iter_stones!(self, ren_idx).next().unwrap() as u16,
            self.chains[ren_idx].origin,
            "The origin doesn't match"
        );
        assert_eq!(
            iter_stones!(self, ren_idx).last().unwrap() as u16,
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
