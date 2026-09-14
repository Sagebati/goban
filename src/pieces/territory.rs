//! Module with tools for getting the connected stones and liberties.

use std::collections::HashSet;

use crate::pieces::goban::{Goban, BOARD_MAX_LENGTH};
use crate::pieces::stones::Point;
use crate::pieces::stones::{Color, EMPTY};
use crate::pieces::util::coord::one_to_2dim;
use crate::pieces::BoardIdx;

impl Goban {
    ///
    /// Get the group of stones connected to a stone. with a Breadth First Search,
    /// works for EMPTY stones too.
    ///
    /// Ex: Passing a stone 'a' it will return and HashSet [a,b,t,z] with the string where the
    /// stone is.
    /// It will return the stone alone if it's lonely
    ///
    pub fn get_group_from_point(&self, stone: Point) -> HashSet<Point> {
        let mut explored = HashSet::<Point>::new();
        explored.insert(stone);

        let mut to_explore: Vec<Point> = self
            .get_connected_points(stone.coord)
            .filter(|p| p.color == stone.color)
            .collect(); // Acquiring all the neighbors

        while let Some(stone_to_explore) = to_explore.pop() {
            // exploring the graph
            explored.insert(stone_to_explore);

            to_explore.extend(
                self.get_connected_points(stone_to_explore.coord)
                    .filter(|p| p.color == stone.color && !explored.contains(p)),
            )
        }
        explored
    }

    ///
    /// Pass an iterator of stones [x,a] and It will compute the string of each stone
    /// stones.
    /// Use a breadth first search to deduce the groups of connected stones.
    /// Get stones connected. [[x,y,z],[a,e,r]] example of return.
    ///
    pub fn get_chains_from_stones(
        &self,
        stones: impl Iterator<Item = Point>,
    ) -> Vec<HashSet<Point>> {
        let mut groups_of_stones: Vec<HashSet<Point>> = Default::default();
        for s in stones {
            let is_handled = groups_of_stones.iter().any(|set| set.contains(&s));

            if !is_handled {
                groups_of_stones.push(self.get_group_from_point(s))
            }
        }
        groups_of_stones
    }

    /// Get two iterators of empty points. The first one is the territory of black the second is white territory
    ///
    /// A flood fill over the empty points: each empty region is visited once
    /// and belongs to a colour when every stone touching it has that colour.
    pub fn get_territories(&self) -> (impl Iterator<Item = Point>, impl Iterator<Item = Point>) {
        let size = self.size();
        let board = self.board();
        let points = size.0 as usize * size.1 as usize;
        let mut visited = [false; BOARD_MAX_LENGTH];
        let mut stack: Vec<BoardIdx> = Vec::with_capacity(points);
        let mut region: Vec<BoardIdx> = Vec::with_capacity(points);
        let mut black_territory = Vec::with_capacity(50);
        let mut white_territory = Vec::with_capacity(50);

        for start in 0..points {
            if visited[start] || board[start].is_some() {
                continue;
            }
            let (mut touches_black, mut touches_white) = (false, false);
            region.clear();
            visited[start] = true;
            stack.push(start);
            while let Some(idx) = stack.pop() {
                region.push(idx);
                for neighbor in self.neighbors_idx(idx) {
                    match board[neighbor] {
                        None => {
                            if !visited[neighbor] {
                                visited[neighbor] = true;
                                stack.push(neighbor);
                            }
                        }
                        Some(chain) => match self.chain_color(chain) {
                            Color::Black => touches_black = true,
                            Color::White => touches_white = true,
                        },
                    }
                }
            }
            let owner = match (touches_black, touches_white) {
                (true, false) => &mut black_territory,
                (false, true) => &mut white_territory,
                _ => continue,
            };
            owner.extend(region.iter().map(|&idx| Point {
                coord: one_to_2dim(size, idx),
                color: EMPTY,
            }));
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
