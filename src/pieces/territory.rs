//! Module with tools for getting the connected stones and liberties.

use std::collections::HashSet;

use crate::pieces::goban::Goban;
use crate::pieces::stones::{Color, EMPTY};
use crate::pieces::stones::Point;

impl Goban {
    #[inline]
    pub fn get_dead_chains_by_color(&self, color: Color) -> impl Iterator<Item=usize> + '_ {
        self.chains
            .iter()
            .enumerate()
            .filter(move |(_, go_string)| {
                go_string.used && go_string.color == color && go_string.is_dead()
            })
            .map(|(idx, _)| idx)
    }

    ///
    /// Get the chain of stones connected to a stone. with a Breadth First Search,
    /// works for EMPTY stones too.
    ///
    /// Ex: Passing a stone 'a' it will return and HashSet [a,b,t,z] with the string where the
    /// stone is.
    /// It will return the stone alone if it's lonely
    ///
    pub fn get_chain_from_stone(&self, stone: Point) -> HashSet<Point> {
        let mut explored = HashSet::<Point>::new();
        explored.insert(stone);

        let mut to_explore: Vec<Point> = self
            .get_neighbors_points(stone.coord)
            .filter(|p| p.color == stone.color)
            .collect(); // Acquiring all the neighbors

        while let Some(stone_to_explore) = to_explore.pop() {
            // exploring the graph
            explored.insert(stone_to_explore);

            to_explore.extend(
                self.get_neighbors_points(stone_to_explore.coord)
                    .filter(|p| p.color == stone.color && !explored.contains(p)),
            )
        }
        explored
    }

    ///
    /// Pass a iterator of stones [x,a] and It will compute the string of each stone
    /// stones.
    /// Use a breadth first search to deduce the groups of connected stones.
    /// Get stones connected. [[x,y,z],[a,e,r]] example of return.
    ///
    pub fn get_chains_from_stones(
        &self,
        stones: impl Iterator<Item=Point>,
    ) -> Vec<HashSet<Point>> {
        let mut groups_of_stones: Vec<HashSet<Point>> = Default::default();
        for s in stones {
            let is_handled = groups_of_stones.iter().any(|set| set.contains(&s));

            if !is_handled {
                groups_of_stones.push(self.get_chain_from_stone(s))
            }
        }
        groups_of_stones
    }

    ///
    /// Get two iterators of empty stones.
    ///
    pub fn get_territories(&self) -> (impl Iterator<Item=Point>, impl Iterator<Item=Point>) {
        let empty_strings =
            self.get_chains_from_stones(self.get_empty_coords().map(|point| Point {
                coord: point,
                color: EMPTY,
            }));
        let mut white_territory = Vec::with_capacity(50);
        let mut black_territory = Vec::with_capacity(50);
        for empty_string in empty_strings {
            let mut neutral = (false, false);
            for empty_intersection in &empty_string {
                for stone in self.get_neighbors_points(empty_intersection.coord) {
                    if stone.color == Some(Color::White) {
                        neutral.1 = true; // found white stone
                    }
                    if stone.color == Some(Color::Black) {
                        neutral.0 = true; // found black stone
                    }
                }
            }
            if neutral.0 && !neutral.1 {
                black_territory.extend(empty_string.into_iter())
            } else if !neutral.0 && neutral.1 {
                white_territory.extend(empty_string.into_iter())
            }
        }
        (black_territory.into_iter(), white_territory.into_iter())
    }

    ///
    /// Calculates a score for the endgame. It's a naive implementation, it counts only
    /// territories with the same color surrounding them.
    ///
    /// Returns (black territory,  white territory)
    ///
    #[inline]
    pub fn calculate_territories(&self) -> (usize, usize) {
        let (black_territory, white_territory) = self.get_territories();
        (black_territory.count(), white_territory.count())
    }
}
