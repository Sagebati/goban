//! Module with the goban and his implementations.

use crate::one2dim;
use crate::pieces::group::CircularGroupIter;
use crate::pieces::group::{merge, set, Group, Groups, Liberties, EMPTY_LIBERTIES};
use crate::pieces::stones::*;
use crate::pieces::util::coord::{
    one_to_2dim, two_to_1dim, valid_coords, Coord, IntoCoord, IntoIdx, Size,
};
use crate::pieces::zobrist::*;
use crate::pieces::{Connections, Nat};
use arrayvec::ArrayVec;
use nonmax::NonMaxU16;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

pub type GroupIdx = usize;
pub type BoardIdx = usize;

const BOARD_MAX_SIZE: (Nat, Nat) = (19, 19);
const BOARD_MAX_LENGTH: usize = BOARD_MAX_SIZE.0 as usize * BOARD_MAX_SIZE.1 as usize;
const MAX_CHAINS: usize = 4 * BOARD_MAX_LENGTH / 5;

/// Represents a goban. the stones are stored in ROW MAJOR (row, column)
#[derive(Debug, Clone, Eq)]
pub struct Goban {
    chains: Groups,
    /// The board contains indexes of the chains
    board: Vec<Option<NonMaxU16>>,
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
            .filter_map(|(coord, m_color)| m_color.map(|color| (coord, color)))
            .for_each(|coord_color| {
                game.push(coord_color.0, coord_color.1);
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
        assert!(
            height <= 19 && width <= 19,
            "We don't handle board > 19 for the moment"
        );
        Goban {
            size: (height, width),
            zobrist_hash: 0,
            board: vec![None; BOARD_MAX_LENGTH],
            next_stone: vec![0; BOARD_MAX_LENGTH],
            chains: Groups::with_capacity(MAX_CHAINS),
            //free_slots: Default::default(),
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.zobrist_hash
    }

    // Returns all the groups in the goban even dead groups
    pub fn chains(&self) -> impl Iterator<Item = &Group> {
        self.chains.iter()
    }

    pub fn chain_stones(&self, idx: impl IntoIdx) -> impl Iterator<Item = Stone> + '_ {
        let idx = idx.into_idx(self.size);
        let chain = self.chains[idx];
        let color = chain.color;
        chain.iter(&self.next_stone).map(move |e| Stone {
            coord: one_to_2dim(self.size, e),
            color,
        })
    }

    /// Returns the underlying goban in a vector with a RowMajor Policy, calculated on the fly.
    pub fn to_vec(&self) -> Vec<MaybeColor> {
        self.board
            .iter()
            .map(|point| {
                point.map_or(EMPTY, |go_str_ptr| {
                    self.chains[go_str_ptr.get() as usize].color.into()
                })
            })
            .collect()
    }

    /// Like vec but in a matrix shape.
    pub fn matrix(&self) -> Vec<Vec<MaybeColor>> {
        let mut mat = vec![];
        for line in self.board.chunks_exact(self.size.1 as usize) {
            let v = line
                .iter()
                .map(|o| o.map_or(EMPTY, |idx| self.chains[idx].color.into()))
                .collect();
            mat.push(v);
        }
        mat
    }

    /// Get the number of stones on the goban.
    /// (number of black stones, number of white stones)
    pub fn number_of_stones(&self) -> (u32, u32) {
        let mut black_stones = 0;
        let mut white_stones = 0;

        for group in self.chains.iter() {
            match group.color {
                Color::White => {
                    white_stones += group.num_stones as u32;
                }
                Color::Black => {
                    black_stones += group.num_stones as u32;
                }
            }
        }

        (black_stones, white_stones)
    }

    #[inline]
    pub(crate) fn board(&self) -> &[Option<NonMaxU16>] {
        &self.board
    }

    /// pushes the stone
    /// # Arguments
    /// point: the point where the stone will be placed
    /// color: the color of the stone must be != empty
    /// # Returns
    /// A tuple with (groups without liberties, the group where the point was added)
    pub(crate) fn push_wth_feedback(
        &mut self,
        point: Coord,
        color: Color,
    ) -> (ArrayVec<usize, 4>, GroupIdx) {
        let pushed_stone_idx = two_to_1dim(self.size, point);

        let mut adjacent_same_color_groups = ArrayVec::<BoardIdx, 4>::new();
        let mut adjacent_opposite_color_groups = ArrayVec::<BoardIdx, 4>::new();
        let mut liberties = ArrayVec::<BoardIdx, 4>::new();

        // For each neighbor we fill the right vector
        for neighbor_idx in self.neighbors_idx(pushed_stone_idx) {
            match self.board[neighbor_idx] {
                Some(adj_ren_index) => {
                    let adj_ren_index = adj_ren_index.get() as usize;
                    if self.chains[adj_ren_index].color == color {
                        if !adjacent_same_color_groups.contains(&adj_ren_index) {
                            adjacent_same_color_groups.push(adj_ren_index);
                        }
                    } else if !adjacent_opposite_color_groups.contains(&adj_ren_index) {
                        adjacent_opposite_color_groups.push(adj_ren_index);
                    }
                }
                None => {
                    liberties.push(neighbor_idx);
                }
            }
        }

        let mut dead_groups = ArrayVec::<BoardIdx, 4>::new();
        // for every string of opposite color remove a liberty and update the string.
        for ren_idx in adjacent_opposite_color_groups {
            let group = &mut self.chains[ren_idx];
            group.remove_liberty(pushed_stone_idx);
            if group.is_dead() {
                dead_groups.push(ren_idx);
            }
        }

        let number_of_neighbors_strings = adjacent_same_color_groups.len();

        let updated_ren_index = match number_of_neighbors_strings {
            0 => self.create_chain(pushed_stone_idx, color, &liberties),
            1 => {
                let only_ren_idx = adjacent_same_color_groups[0];

                self.chains[only_ren_idx]
                    .remove_liberty(pushed_stone_idx)
                    .union_liberties_slice(&liberties);
                self.add_stone_to_chain(only_ren_idx, pushed_stone_idx);
                self.board[pushed_stone_idx] = Some(NonMaxU16::new(only_ren_idx as u16).unwrap());
                only_ren_idx
            }
            _ => {
                let mut to_merge = self.create_chain(pushed_stone_idx, color, &liberties);
                for adj_ren in adjacent_same_color_groups {
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
        (dead_groups, updated_ren_index)
    }

    pub(crate) fn remove_captured_stones_aux(
        &mut self,
        suicide_allowed: bool,
        dead_groups_indices: &[GroupIdx],
        added_group: GroupIdx,
    ) -> ((u32, u32), Option<Coord>) {
        let only_one_ren_removed = dead_groups_indices.len() == 1;
        let mut stones_removed = (0, 0);
        let mut ko_point = None;
        for &dead_ren_idx in dead_groups_indices {
            let dead_chain = &self.chains[dead_ren_idx];
            // If only one stone and one group is removed then it becomes a ko point
            if dead_chain.num_stones == 1 && only_one_ren_removed {
                ko_point = Some(one_to_2dim(self.size(), dead_chain.origin as usize));
            }
            match dead_chain.color {
                Color::White => {
                    stones_removed.0 += dead_chain.num_stones as u32;
                }
                Color::Black => {
                    stones_removed.1 += dead_chain.num_stones as u32;
                }
            }
            self.remove_chain(dead_ren_idx);
        }

        let maybe_dead_chain = &self.chains[added_group];
        if suicide_allowed && maybe_dead_chain.is_dead() {
            match maybe_dead_chain.color {
                Color::White => {
                    stones_removed.0 += maybe_dead_chain.num_stones as u32;
                }
                Color::Black => {
                    stones_removed.1 += maybe_dead_chain.num_stones as u32;
                }
            }
            ko_point = None;
            self.remove_chain(added_group);
        }
        (stones_removed, ko_point)
    }

    /// Put a stone in the goban.
    /// default (line, column)
    /// the (0,0) point is in the top left.
    ///
    /// # Panics
    /// if the point is out of bounds
    pub fn push(&mut self, point: Coord, color: Color) -> &mut Self {
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
    pub fn push_stone(&mut self, stone: Stone) -> &mut Goban {
        self.push(stone.coord, stone.color)
    }

    /// Put many stones.
    #[inline]
    pub fn push_many(&mut self, points: &[Coord], value: Color) {
        points.iter().for_each(|&point| {
            self.push(point, value);
        })
    }

    /// Get all the neighbors to the coordinate including empty intersections.
    #[inline]
    pub fn get_connected_points(&self, point: Coord) -> impl Iterator<Item = Point> + '_ {
        self.neighbors_coords(point).map(move |p| Point {
            coord: p,
            color: self.get_color(p),
        })
    }

    /// Get all the stones that are connected to the coordinates, except empty intersections.
    #[inline]
    pub fn get_connected_stones(&self, point: impl IntoCoord) -> impl Iterator<Item = Stone> + '_ {
        let point = point.into_coord(self.size);
        self.get_connected_points(point)
            .filter_map(|x| x.try_into().ok())
    }

    /// Get all connected groups to the coordinate.
    pub(crate) fn get_connected_groups(&self, index: impl IntoIdx) -> Connections<&Group> {
        self.get_connected_groups_idx(index)
            .into_iter()
            .map(|e| &self.chains[e])
            .collect()
    }

    /// Get a set of the groups adjacent to the point.
    #[inline]
    pub fn get_connected_groups_idx(&self, index: impl IntoIdx) -> Connections<GroupIdx> {
        let index = index.into_idx(self.size);
        let mut array_vec: ArrayVec<GroupIdx, 4> = ArrayVec::new_const();
        for idx in self.neighbors_idx(index) {
            if let Some(idx) = self.board[idx] {
                if !array_vec.contains(&(idx.get() as usize)) {
                    array_vec.push(idx.get() as usize);
                }
            }
        }
        array_vec
    }

    #[inline]
    pub fn get_color(&self, coord: impl IntoIdx) -> MaybeColor {
        let idx = coord.into_idx(self.size);
        self.board[idx].map(|chain_id| self.chains[chain_id.get() as usize].color)
    }

    /// Get all the stones except "EMPTY stones"
    #[inline]
    pub fn get_stones(&self) -> impl Iterator<Item = Stone> + '_ {
        self.board.iter().enumerate().filter_map(move |(index, o)| {
            o.map(move |chain_idx| Stone {
                coord: one_to_2dim(self.size, index),
                color: self.chains[chain_idx.get() as usize].color,
            })
        })
    }

    /// Get stones by their color.
    #[inline]
    pub fn get_stones_by_color(&self, color: MaybeColor) -> impl Iterator<Item = Point> + '_ {
        self.get_coords_by_color(color)
            .map(move |c| Point { color, coord: c })
    }

    pub fn get_empty_idx(&self) -> impl Iterator<Item = BoardIdx> + '_ {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(idx, group)| group.map(|_| idx))
    }

    pub fn get_empty_coords(&self) -> impl Iterator<Item = Coord> + '_ {
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
    pub fn get_coords_by_color(&self, color: MaybeColor) -> impl Iterator<Item = Coord> + '_ {
        let mut res = ArrayVec::<Coord, BOARD_MAX_LENGTH>::new();
        for board_idx in 0..(self.size.0 * self.size.1) as usize {
            match color {
                EMPTY => res.push(one_to_2dim(self.size, board_idx)),
                Some(c) => self.board[board_idx]
                    .filter(|&chain_idx| self.chains[chain_idx.get() as usize].color == c)
                    .map(|_| res.push(one_to_2dim(self.size, board_idx)))
                    .unwrap_or(()),
            }
        }
        res.into_iter()
    }

    /// Returns the "empty" stones connected to the stone
    #[inline]
    pub fn get_liberties(&self, coord: Coord) -> impl Iterator<Item = Coord> + '_ {
        self.neighbors_coords(coord)
            .filter(|&x| self.get_color(x).is_none())
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

    /// Remove a string from the game, it adds liberties to all
    /// adjacent chains that are different color.
    pub fn remove_chain(&mut self, ren_to_remove_idx: GroupIdx) {
        let chain = self.chains[ren_to_remove_idx];
        let color_of_the_string = chain.color;
        for point_idx in chain.iter(&self.next_stone) {
            let mut neighbors_chains = self.get_connected_groups_idx(point_idx);
            // We remove our group from the neighbors
            neighbors_chains.retain(|x| *x != ren_to_remove_idx);

            for &n in &neighbors_chains {
                self.chains[n].add_liberty(point_idx);
            }
            self.zobrist_hash ^= index_zobrist(point_idx, color_of_the_string);
            self.board[point_idx] = None;
        }
        self.chains.remove(ren_to_remove_idx);
    }

    /// Updates the group idx of the board when merging groups
    fn update_chain_indexes_in_board(&mut self, chain_idx: GroupIdx) {
        debug_assert_eq!(
            self.chains[chain_idx]
                .iter(&self.next_stone)
                .last()
                .unwrap() as u16,
            self.chains[chain_idx].last
        );
        for point in self.chains[chain_idx].iter(&self.next_stone) {
            self.board[point] = Some(NonMaxU16::new(chain_idx as u16).unwrap());
        }
    }

    /// Get the neighbors points of a point.
    #[inline]
    fn neighbors_coords(&self, coord: Coord) -> impl Iterator<Item = Coord> {
        valid_coords(coord, self.size).into_iter()
    }

    #[inline]
    fn neighbors_idx(&self, board_idx: BoardIdx) -> impl Iterator<Item = BoardIdx> {
        let size = self.size;
        self.neighbors_coords(one_to_2dim(size, board_idx))
            .map(move |coord| two_to_1dim(size, coord))
    }

    #[inline]
    fn create_chain(&mut self, origin: BoardIdx, color: Color, liberties: &[BoardIdx]) -> GroupIdx {
        let mut lib_bitset: Liberties = EMPTY_LIBERTIES;
        for &board_idx in liberties {
            set::<true>(board_idx, &mut lib_bitset);
        }
        let chain_to_place = Group::new_with_liberties(color, origin, lib_bitset);
        self.next_stone[origin] = origin as u16;
        let chain_idx = self.chains.put_free_spot(chain_to_place);
        self.update_chain_indexes_in_board(chain_idx);
        chain_idx
    }

    fn add_stone_to_chain(&mut self, group_idx: GroupIdx, stone: BoardIdx) {
        let group = &mut self.chains[group_idx];
        if stone < group.origin as usize {
            // replace origin
            self.next_stone[stone] = group.origin;
            self.next_stone[group.last as usize] = stone as u16;
            group.origin = stone as u16;
        } else {
            self.next_stone[group.last as usize] = stone as u16;
            self.next_stone[stone] = group.origin;
            group.last = stone as u16;
        }
        group.num_stones += 1;
        debug_assert_eq!(
            self.chains[group_idx].iter(&self.next_stone).last().unwrap() as u16,
            self.chains[group_idx].last
        );
    }

    fn merge_strings(&mut self, group1_idx: GroupIdx, group2_idx: GroupIdx) {
        assert_eq!(
            self.chains[group1_idx].color, self.chains[group2_idx].color,
            "Cannot merge two strings of different color"
        );
        assert_ne!(group1_idx, group2_idx, "merging the same string");

        // We select the biggest group first to optimize the merging
        let (group1, group2) = if group1_idx < group2_idx {
            let (s1, s2) = self.chains.0.split_at_mut(group2_idx);
            (&mut s1[group1_idx], s2.first_mut().unwrap())
        } else {
            // ren2_idx > ren1_idx
            let (contains_chain2, contains_ren1) = self.chains.0.split_at_mut(group1_idx);
            (
                contains_ren1.first_mut().unwrap(),
                &mut contains_chain2[group2_idx],
            )
        };

        let group1 = group1.as_mut().unwrap();
        let group2 = group2.as_ref().unwrap();

        // We merge liberties
        merge(&mut group1.liberties, &group2.liberties);

        // We update chain1 origin and last
        let chain1_last = group1.last;
        let chain2_last = group2.last;

        let chain1_origin = group1.origin;
        let chain2_origin = group2.origin;

        // We need to merge two so we take the least origin, or we make group 1 point to group 2
        if chain1_origin > chain2_origin {
            group1.origin = chain2_origin;
        } else {
            group1.last = chain2_last;
        }

        self.next_stone
            .swap(chain1_last as usize, chain2_last as usize);

        group1.num_stones += group2.num_stones;

        self.update_chain_indexes_in_board(group1_idx);
        self.chains.remove(group2_idx);
    }

    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    fn check_integrity_group(&self, group_idx: GroupIdx) {
        assert_eq!(
            self.iter_stones(group_idx).next().unwrap() as u16,
            self.chains[group_idx].origin,
            "The origin doesn't match"
        );
        assert_eq!(
            self.iter_stones(group_idx).last().unwrap() as u16,
            self.chains[group_idx].last,
            "The last doesn't match"
        );
        
        if self.iter_stones(group_idx).count() as u16 != self.chains[group_idx].num_stones {
            panic!("The number of stones don't match")
        }
    }

    #[inline(always)]
    fn iter_stones(&self, group_idx: usize) -> CircularGroupIter<'_> {
        self.chains[group_idx].iter(&self.next_stone)
    }

    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    fn check_integrity_all(&self) {
        for (ren_idx, _) in self.chains.iter_with_index() {
            self.check_integrity_group(ren_idx);
        }
    }
}

impl Display for Goban {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.pretty_string())
    }
}

impl Default for Goban {
    fn default() -> Self {
        Goban::new((19, 19))
    }
}

impl Hash for Goban {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.zobrist_hash);
    }
}

impl PartialEq for Goban {
    fn eq(&self, other: &Self) -> bool {
        if self.size != other.size || self.zobrist_hash != other.zobrist_hash {
            return false;
        }

        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                if self.get_color((x, y)) != other.get_color((x, y)) {
                    return false;
                }
            }
        }

        true
    }
}
