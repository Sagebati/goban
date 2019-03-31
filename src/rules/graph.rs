//! Module with tools for getting the connected stones and liberties.

use std::collections::HashSet;
use crate::pieces::stones::Stone;
use crate::pieces::goban::Goban;
use crate::pieces::stones::Color;

impl Goban {
    ///
    /// Test if a group of stones is dead.
    ///
    /// "a group of stones is dead if it doesn't have liberties"
    ///
    pub fn are_dead(&self, stones: &HashSet<Stone>) -> bool {
        !stones // If there is one stone connected who has liberties, they are not captured.
            .iter()
            .any(|s| self.has_liberties(s))
    }

    ///
    /// Get all the groups of connected stones, who has at least one stones
    /// without liberties.
    ///
    pub fn get_groups_of_stones_without_liberties(&self) -> Vec<HashSet<Stone>> {
        let stones_without_liberties = self
            // get all stones without liberties
            .get_stones()
            .filter(|point| !self.has_liberties(point));

        self.get_groups_by_stone(stones_without_liberties)
    }

    ///
    /// Get all the groups of connected stones of a color, who has at leart one stone without
    /// liberties.
    ///
    /// if the color is empty it's an undefined behaviour
    ///
    pub fn get_groups_of_stones_color_without_liberties(&self, color: Color) ->
    Vec<HashSet<Stone>> {
        let stones_without_libnerties = self
            .get_stones_by_color(color)
            // get all stones without liberties
            .filter(|point| !self.has_liberties(point));
        self.get_groups_by_stone(stones_without_libnerties)
    }

    ///
    /// Get the chain of stones connected to a stone. with a Breadth First Search,
    /// works for Empty stones too.
    ///
    pub fn bfs(&self, point: &Stone) -> HashSet<Stone> {
        let mut explored: HashSet<Stone> = HashSet::new();
        explored.insert(point.clone());

        let mut to_explore: Vec<Stone> = self.get_neighbors(&point.coord)
            .filter(|p| p.color == point.color)
            .collect(); // Acquiring all the neighbors

        while let Some(stone_to_explore) = to_explore.pop() { // exploring the graph
            explored.insert(stone_to_explore.clone());
            self.get_neighbors(&stone_to_explore.coord)
                .filter(|p| p.color == point.color && !explored.contains(p))
                .for_each(|s| to_explore.push(s));
        }
        explored
    }

    ///
    /// Use a breadth first search to deduce the groups of connected stones.
    /// Get stones connected. [[x,y,z],[a,e,r]] example of return.
    ///
    pub fn get_groups_by_stone(&self, stones: impl Iterator<Item=Stone>) ->
    Vec<HashSet<Stone>> {
        let mut groups_of_stones: Vec<HashSet<Stone>> = Vec::new();
        for stone in stones {
            // if the stone is already in a group of stones
            let is_handled = groups_of_stones
                .iter()
                .any(|set| set.contains(&stone));

            if !is_handled {
                groups_of_stones.push(self.bfs(&stone))
            }
        }
        groups_of_stones
    }
}