use bitvec::prelude::*;

use crate::pieces::BoardIdx;
use crate::pieces::stones::Color;

//pub type Liberties = BitArr![for 361, in usize];
pub type Liberties = [u64; 6];

#[inline(always)]
pub fn set<const VAL: bool>(index: usize, lib: &mut Liberties) {
    let chunk = index / u64::BITS as usize;
    let bit_index = index % u64::BITS as usize;
    let mask = 1 << bit_index;
    if VAL {
        lib[chunk] |= mask;
    } else {
        lib[chunk] &= !mask;
    }
}

#[inline(always)]
pub fn merge(lib: &mut Liberties, o: &Liberties) {
    lib[0] |= o[0];
    lib[1] |= o[1];
    lib[2] |= o[2];
    lib[3] |= o[3];
    lib[4] |= o[4];
    lib[5] |= o[5];
}

#[inline(always)]
fn any(lib: &Liberties) -> bool {
    lib.iter().any(|&x| x != 0)
}

fn count_ones(lib: &Liberties) -> usize {
    let mut sum = 0;
    sum += lib[0].count_ones();
    sum += lib[1].count_ones();
    sum += lib[2].count_ones();
    sum += lib[3].count_ones();
    sum += lib[4].count_ones();
    sum += lib[5].count_ones();
    sum as usize
}

fn iter_ones(lib: &Liberties) -> Vec<usize> {
    let mut ones = Vec::with_capacity(64 * 6);
    for i in 0..6 {
        let mut chunk = lib[i];
        let mut index = 0;
        while chunk != 0 {
            let zeros = chunk.trailing_zeros();
            index += zeros as usize + 1;
            ones.push(index - 1 + 64 * i);
            chunk = chunk.checked_shr(zeros + 1).unwrap_or(0);
        }
    }
    ones
}

fn get(index: usize, lib: &Liberties) -> bool {
    let chunk = index / 64;
    let bit_index = index as u64 % 64;
    (lib[chunk] & (1 << bit_index)) != 0
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
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
            //self.liberties[stone_idx],
            get(stone_idx, &self.liberties),
            "Tried to remove a liberty, who isn't present. stone idx: {}",
            stone_idx
        );
        //self.liberties.set(stone_idx, false);
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
            "Tried to add a liberty already present, stone idx: {}",
            stone_idx
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
        iter_ones(&self.liberties)
    }
}
