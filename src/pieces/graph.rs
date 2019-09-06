//! Module with tools for getting the connected stones and liberties.

use crate::pieces::goban::Goban;
use crate::pieces::stones::Color;
use crate::pieces::stones::Stone;
use std::collections::HashSet;

impl Goban {
    ///
    /// Test if a group of string is dead.
    ///
    /// "a group of string is dead if it doesn't have liberties"
    ///
    pub fn is_string_dead(&self, string: &HashSet<Stone>) -> bool {
        !string // If there is one stone connected who has liberties, they are not captured.
            .iter()
            .any(|s| self.has_liberties(*s))
    }

    ///
    /// Count the liberties of a string
    ///
    pub fn count_string_liberties(&self, string: &HashSet<Stone>) -> u8 {
        string
            .iter()
            .flat_map(|s| self.get_liberties(*s))
            .collect::<HashSet<Stone>>()
            .len() as u8
    }

    ///
    /// Get all the groups of connected stones, who has at least one stones
    /// without liberties.
    ///
    pub fn get_strings_of_stones_without_liberties(&self) -> Vec<HashSet<Stone>> {
        let stones_without_liberties = self
            // get all stones without liberties
            .get_stones()
            .filter(|point| !self.has_liberties(*point));

        self.get_strings_from_stones(stones_without_liberties)
    }

    ///
    /// Get all the strings of a color who doesn't have liberties.
    ///
    /// Ex: Passing black to this function will return an structure like this
    /// [[a,b,c],[t,x,y],[y]]
    pub fn get_strings_of_stones_without_liberties_wth_color(
        &self,
        color: Color,
    ) -> Vec<HashSet<Stone>> {
        let stones_without_libnerties = self
            .get_stones_by_color(color)
            // get all stones without liberties
            .filter(|point| !self.has_liberties(*point));
        self.get_strings_from_stones(stones_without_libnerties)
    }

    ///
    /// Get the chain of stones connected to a stone. with a Breadth First Search,
    /// works for Empty stones too.
    ///
    /// Ex: Passing a stone 'a' it will return and HashSet [a,b,t,z] with the string where the
    /// stone is.
    /// It will return the stone alone if it's lonely
    ///
    pub fn get_string_from_stone(&self, stone: Stone) -> HashSet<Stone> {
        let mut explored: HashSet<Stone> = HashSet::new();
        explored.insert(stone);

        let mut to_explore: Vec<Stone> = self
            .get_neighbors(stone.coordinates)
            .filter(|p| p.color == stone.color)
            .collect(); // Acquiring all the neighbors

        while let Some(stone_to_explore) = to_explore.pop() {
            // exploring the graph
            explored.insert(stone_to_explore);
            self.get_neighbors(stone_to_explore.coordinates)
                .filter(|p| p.color == stone.color && !explored.contains(p))
                .for_each(|s| to_explore.push(s));
        }
        explored
    }

    ///
    /// Pass a iterator of stones [x,a] and It will compute the string of each stone
    /// stones.
    /// Use a breadth first search to deduce the groups of connected stones.
    /// Get stones connected. [[x,y,z],[a,e,r]] example of return.
    ///
    pub fn get_strings_from_stones(
        &self,
        stones: impl Iterator<Item = Stone>,
    ) -> Vec<HashSet<Stone>> {
        let mut groups_of_stones: Vec<HashSet<Stone>> = Default::default();
        for stone in stones {
            // if the stone is already in a group of stones
            let is_handled = groups_of_stones.iter().any(|set| set.contains(&stone));

            if !is_handled {
                groups_of_stones.push(self.get_string_from_stone(stone))
            }
        }
        groups_of_stones
    }
}
