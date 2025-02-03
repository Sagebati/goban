use std::ops::BitOrAssign;

use arrayvec::ArrayVec;

use crate::pieces::BoardIdx;
use crate::pieces::stones::Color;

//pub type Liberties = BitArr![for 361, in usize];

type Bucket = u32;

const SIZE: usize = 361 / Bucket::BITS as usize + 1;
const BITS: usize = Bucket::BITS as usize;

pub type Liberties = [Bucket; SIZE];

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
fn any(lib: &Liberties) -> bool {
    lib.iter().any(|&x| x != 0)
}

fn count_ones(lib: &Liberties) -> usize {
    lib.iter().map(|x| x.count_ones() as usize).sum()
}

fn iter_ones(lib: &Liberties) -> impl Iterator<Item=usize> + '_ {
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
pub struct Chain {
    pub color: Color,
    pub origin: u16,
    pub last: u16,
    pub liberties: Liberties,
    pub used: bool,
    pub num_stones: u16,
}

impl Chain {
    #[inline]
    pub fn new(color: Color, stone: BoardIdx) -> Self {
        Self::new_with_liberties(color, stone, Default::default())
    }

    pub fn new_with_liberties(color: Color, stone: BoardIdx, liberties: Liberties) -> Self {
        Chain {
            color,
            origin: stone as u16,
            last: stone as u16,
            liberties,
            used: true,
            num_stones: 1,
        }
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        // !self.liberties.any()
        !any(&self.liberties)
    }

    #[inline]
    pub fn number_of_liberties(&self) -> usize {
        //self.liberties.count_ones()
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
        set::<false>(stone_idx, &mut self.liberties);
        self
    }

    #[inline(always)]
    fn add_liberty_unchecked(&mut self, stone_idx: BoardIdx) -> &mut Self {
        set::<true>(stone_idx, &mut self.liberties);
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
    pub fn add_liberties(&mut self, stones_idx: impl Iterator<Item=BoardIdx>) -> &mut Self {
        for idx in stones_idx {
            self.add_liberty(idx);
        }
        self
    }

    #[inline]
    pub fn union_liberties(&mut self, liberties_idx: Liberties) -> &mut Self {
        //self.liberties |= liberties_idx;
        merge(&mut self.liberties, &liberties_idx);
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
}
