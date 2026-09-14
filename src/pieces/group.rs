use std::ops::{BitOrAssign, Index, IndexMut};

use crate::pieces::goban::{GroupIdx, MAX_CHAINS};
use crate::pieces::stones::Color;
use crate::pieces::BoardIdx;
use arrayvec::ArrayVec;
use nonmax::NonMaxU16;
use std::iter::FusedIterator;
//pub type Liberties = BitArr![for 361, in usize];

type Bucket = u8;

const SIZE: usize = 361 / Bucket::BITS as usize + 1;
const BITS: usize = Bucket::BITS as usize;

pub type Liberties = [Bucket; SIZE];

pub const EMPTY_LIBERTIES: Liberties = [0; SIZE];

#[inline(always)]
pub fn set<const VAL: bool>(index: usize, lib: &mut Liberties) {
    let chunk = index / BITS;
    let bit_index = index % BITS;
    let mask = 1 << bit_index;
    if VAL {
        lib[chunk] |= mask;
    } else {
        lib[chunk] &= !mask;
    }
}

#[inline(always)]
pub fn merge(lib: &mut Liberties, o: &Liberties) {
    lib.iter_mut().zip(o).for_each(|(x, o)| x.bitor_assign(o))
}

#[inline(always)]
fn count_ones(lib: &Liberties) -> usize {
    lib.iter().map(|x| x.count_ones() as usize).sum()
}

fn iter_ones(lib: &Liberties) -> impl Iterator<Item = usize> + '_ {
    lib.iter().enumerate().flat_map(|(ix, chunk)| {
        let mut chunk = *chunk;
        let mut ixs = ArrayVec::<usize, BITS>::new();
        let mut index = 0;
        while chunk != 0 {
            let zeros = chunk.trailing_zeros();
            index += zeros as usize + 1;
            ixs.push(index - 1 + BITS * ix);
            chunk = chunk.checked_shr(zeros + 1).unwrap_or(0);
        }
        ixs.into_iter()
    })
}

fn get(index: usize, lib: &Liberties) -> bool {
    let chunk = index / BITS;
    let bit_index = index % BITS;
    (lib[chunk] & (1 << bit_index)) != 0
}

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub struct Group {
    pub color: Color,
    pub origin: u16,
    pub last: u16,
    pub liberties: Liberties,
    pub num_stones: u16,
    /// Number of bits set in `liberties`, kept up to date by the liberty
    /// methods so that atari and death tests do not scan the bitset.
    pub(crate) num_liberties: u16,
}

impl Group {
    #[inline]
    pub fn new(color: Color, stone: BoardIdx) -> Self {
        Self::new_with_liberties(color, stone, EMPTY_LIBERTIES)
    }

    pub fn new_with_liberties(color: Color, stone: BoardIdx, liberties: Liberties) -> Self {
        Group {
            color,
            origin: stone as u16,
            last: stone as u16,
            num_liberties: count_ones(&liberties) as u16,
            liberties,
            num_stones: 1,
        }
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        self.num_liberties == 0
    }

    #[inline]
    pub fn number_of_liberties(&self) -> usize {
        self.num_liberties as usize
    }

    /// Recount the liberties from the bitset; the debug integrity check
    /// compares it with the cached count.
    pub(crate) fn recount_liberties(&self) -> usize {
        count_ones(&self.liberties)
    }

    /// A go string is atari if it only has one liberty
    #[inline]
    pub fn is_atari(&self) -> bool {
        self.number_of_liberties() == 1
    }

    #[inline]
    pub fn contains_liberty(&self, stone_idx: BoardIdx) -> bool {
        //self.liberties[stone_idx]
        get(stone_idx, &self.liberties)
    }

    #[inline]
    pub fn remove_liberty(&mut self, stone_idx: BoardIdx) -> &mut Self {
        debug_assert!(
            get(stone_idx, &self.liberties),
            "Tried to remove a liberty, who isn't present. stone idx: {stone_idx}"
        );
        if get(stone_idx, &self.liberties) {
            set::<false>(stone_idx, &mut self.liberties);
            self.num_liberties -= 1;
        }
        self
    }

    #[inline(always)]
    fn add_liberty_unchecked(&mut self, stone_idx: BoardIdx) -> &mut Self {
        if !get(stone_idx, &self.liberties) {
            set::<true>(stone_idx, &mut self.liberties);
            self.num_liberties += 1;
        }
        self
    }

    #[inline]
    pub fn add_liberty(&mut self, stone_idx: BoardIdx) -> &mut Self {
        debug_assert!(
            //self.liberties[stone_idx],
            !get(stone_idx, &self.liberties),
            "Tried to add a liberty already present, stone idx: {stone_idx}"
        );
        //self.liberties.set(stone_idx, true);
        self.add_liberty_unchecked(stone_idx)
    }

    #[inline]
    pub fn add_liberties(&mut self, stones_idx: impl Iterator<Item = BoardIdx>) -> &mut Self {
        for idx in stones_idx {
            self.add_liberty(idx);
        }
        self
    }

    #[inline]
    pub fn union_liberties(&mut self, liberties_idx: Liberties) -> &mut Self {
        //self.liberties |= liberties_idx;
        merge(&mut self.liberties, &liberties_idx);
        self.num_liberties = count_ones(&self.liberties) as u16;
        self
    }

    pub fn union_liberties_slice(&mut self, stones_idx: &[BoardIdx]) -> &mut Self {
        for &idx in stones_idx {
            self.add_liberty_unchecked(idx);
        }
        self
    }

    pub fn liberties(&self) -> Vec<usize> {
        iter_ones(&self.liberties).collect()
    }

    pub fn iter<'a>(&self, next_stone: &'a [u16]) -> CircularGroupIter<'a> {
        CircularGroupIter {
            next_stone,
            origin: self.origin as usize,
            next: Some(self.origin as usize),
            num_stones: self.num_stones,
        }
    }
}

/// Slab of chains. Removed chains leave a `None` hole whose index is kept in
/// the free list and handed out again by `put_free_spot`, so the slab does not
/// grow with every chain ever created during a game.
///
/// Both the slab and the free list are inline arrays of `MAX_CHAINS` slots,
/// which keeps the slab `Copy`; pushing beyond that bound panics.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Groups(
    pub(crate) tinyvec::ArrayVec<[Option<Group>; MAX_CHAINS]>,
    tinyvec::ArrayVec<[u16; MAX_CHAINS]>,
);

impl Default for Groups {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<NonMaxU16> for Groups {
    type Output = Group;

    fn index(&self, index: NonMaxU16) -> &Self::Output {
        self.0[index.get() as usize].as_ref().unwrap()
    }
}

impl Index<usize> for Groups {
    type Output = Group;

    fn index(&self, index: usize) -> &Self::Output {
        self.0[index].as_ref().unwrap()
    }
}

impl IndexMut<BoardIdx> for Groups {
    fn index_mut(&mut self, index: BoardIdx) -> &mut Self::Output {
        self.0[index].as_mut().unwrap()
    }
}

impl IndexMut<NonMaxU16> for Groups {
    fn index_mut(&mut self, index: NonMaxU16) -> &mut Self::Output {
        self.0[index.get() as usize].as_mut().unwrap()
    }
}

impl Groups {
    pub fn new() -> Self {
        Self(tinyvec::ArrayVec::new(), tinyvec::ArrayVec::new())
    }

    /// Store a chain, reusing the slot of a previously removed chain when
    /// one is available. Returns the index of the slot.
    pub fn put_free_spot(&mut self, group: Group) -> GroupIdx {
        match self.1.pop() {
            Some(index) => {
                let index = index as usize;
                debug_assert!(self.0[index].is_none(), "free slot {index} is occupied");
                self.0[index] = Some(group);
                index
            }
            None => {
                assert!(
                    self.0.try_push(Some(group)).is_none(),
                    "more chains alive at once than MAX_CHAINS"
                );
                self.0.len() - 1
            }
        }
    }

    /// Remove a chain and make its slot available to `put_free_spot`.
    pub fn remove(&mut self, index: usize) {
        if self.0[index].take().is_some() {
            self.1.push(index as u16);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Group> {
        self.0.iter().filter_map(|e| e.as_ref())
    }

    pub fn iter_with_index(&self) -> impl Iterator<Item = (GroupIdx, Group)> + '_ {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(idx, e)| e.map(|e| (idx, e)))
    }
}

#[derive(Copy, Clone)]
pub struct CircularGroupIter<'a> {
    next_stone: &'a [u16],
    origin: usize,
    next: Option<usize>,
    num_stones: u16,
}

impl Iterator for CircularGroupIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let origin = self.origin;
        let ret = self.next;
        self.next = self
            .next
            .map(|stone_idx| self.next_stone[stone_idx] as usize)
            .filter(move |&o| o != origin);

        #[cfg(debug_assertions)]
        if ret.is_some() && self.next == ret {
            panic!("infinite loop detected")
        }

        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.num_stones as usize, Some(self.num_stones as usize))
    }
}

impl ExactSizeIterator for CircularGroupIter<'_> {}

impl FusedIterator for CircularGroupIter<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_ren_iter_test() {
        let a = vec![0, 0, 4, 0, 6, 0, 2, 0, 8, 0, 0, 0];
        let mut iter = CircularGroupIter {
            next_stone: &a,
            origin: 2,
            next: Some(2),
            num_stones: 3,
        };
        let iter2 = iter.clone();

        assert_eq!(2, iter.next().unwrap());
        assert_eq!(4, iter.next().unwrap());
        assert_eq!(6, iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());

        assert_eq!(6, iter2.last().unwrap());

        let mut iter = CircularGroupIter {
            next_stone: &a,
            origin: 8,
            next: Some(8),
            num_stones: 1,
        };
        assert_eq!(8, iter.next().unwrap());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
    }
}
