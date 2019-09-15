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
    #[inline]
    pub fn is_string_dead(&self, string: &HashSet<Stone>) -> bool {
        !string // If there is one stone connected who has liberties, they are not captured.
            .iter()
            .any(|s| self.has_liberties(*s))
    }

    ///
    /// Count the liberties of a string
    ///
    #[inline]
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
    #[inline]
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

            to_explore.extend(self.get_neighbors(stone_to_explore.coordinates)
                .filter(|p| p.color == stone.color && !explored.contains(p))
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
    pub fn get_strings_from_stones(
        &self,
        stones: impl Iterator<Item=Stone>,
    ) -> Vec<HashSet<Stone>> {
        let mut groups_of_stones: Vec<HashSet<Stone>> = Default::default();
        for s in stones {
            let is_handled = groups_of_stones.iter().any(|set| set.contains(&s));

            if !is_handled {
                groups_of_stones.push(self.get_string_from_stone(s))
            }
        }
        groups_of_stones
    }

    ///
    /// Calculates a score for the endgame. It's a naive implementation, it counts only
    /// territories with the same color surrounding them.
    ///
    /// Returns (black territory,  white territory)
    ///
    #[inline]
    pub fn calculate_territories(&self) -> (f32, f32) {
        let (black_territory, white_territoty) = self.get_territories();
        (black_territory.count() as f32, white_territoty.count() as f32)
    }

    ///
    /// Get two iterators with the
    ///
    pub fn get_territories(&self) -> (impl Iterator<Item=Stone>, impl Iterator<Item=Stone>) {
        let empty_strings = self.get_strings_from_stones(self.get_stones_by_color(Color::None));
        let mut white_territory = Vec::new();
        let mut black_territory = Vec::new();
        for group in empty_strings {
            let mut neutral = (false, false);
            for empty_intersection in &group {
                for stone in self.get_neighbors(empty_intersection.coordinates) {
                    if stone.color == Color::White {
                        neutral.1 = true; // found white stone
                    }
                    if stone.color == Color::Black {
                        neutral.0 = true; // found black stone
                    }
                }
            }
            if neutral.0 && !neutral.1 {
                black_territory.extend(group.into_iter())
            } else if !neutral.0 && neutral.1 {
                white_territory.extend(group.into_iter())
            }
        }
        (black_territory.into_iter(), white_territory.into_iter())
    }
}
