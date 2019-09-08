use crate::{GameNode, SgfError, SgfErrorKind, SgfToken};

/// A game tree, containing it's nodes and possible variations following the last node
#[derive(Debug, PartialEq)]
pub struct GameTree {
    pub nodes: Vec<GameNode>,
    pub variations: Vec<GameTree>,
}

impl Default for GameTree {
    /// Creates an empty GameTree
    fn default() -> Self {
        GameTree {
            nodes: vec![],
            variations: vec![],
        }
    }
}

impl GameTree {
    /// Counts number of nodes in the longest variation
    pub fn count_max_nodes(&self) -> usize {
        let count = self.nodes.len();
        let variation_count = self
            .variations
            .iter()
            .map(|v| v.count_max_nodes())
            .max()
            .unwrap_or(0);

        count + variation_count
    }

    /// Gets a vector of all nodes that contain a `SgfToken::Unknown` token
    ///
    /// ```rust
    /// use sgf_parser::*;
    ///
    /// let tree: GameTree = parse("(;B[dc];W[ef]TMP[foobar](;B[aa])(;B[cc];W[ee]))").unwrap();
    ///
    /// let unknown_nodes = tree.get_unknown_nodes();
    /// unknown_nodes.iter().for_each(|node| {
    ///     let unknown_tokens = node.get_unknown_tokens();
    ///     assert_eq!(unknown_tokens.len(), 1);
    ///     if let SgfToken::Unknown((identifier, value)) = unknown_tokens[0] {
    ///         assert_eq!(identifier, "TMP");
    ///         assert_eq!(value, "foobar");
    ///     }
    /// });
    ///
    /// ```
    pub fn get_unknown_nodes(&self) -> Vec<&GameNode> {
        let mut unknowns = self
            .nodes
            .iter()
            .filter(|node| {
                node.tokens.iter().any(|t| match t {
                    SgfToken::Unknown(_) => true,
                    _ => false,
                })
            })
            .collect::<Vec<_>>();
        self.variations.iter().for_each(|variation| {
            let tmp = variation.get_unknown_nodes();
            unknowns.extend(tmp);
        });
        unknowns
    }

    /// Gets a vector of all nodes that contain a `SgfToken::Invalid` token
    ///
    /// ```rust
    /// use sgf_parser::*;
    ///
    /// let tree: GameTree = parse("(;B[dc];W[foobar];B[aa])(;B[cc];W[ee]))").unwrap();
    ///
    /// let invalid_nodes = tree.get_invalid_nodes();
    /// invalid_nodes.iter().for_each(|node| {
    ///     let invalid_tokens = node.get_invalid_tokens();
    ///     if let SgfToken::Invalid((identifier, value)) = invalid_tokens[0] {
    ///         assert_eq!(identifier, "W");
    ///         assert_eq!(value, "foobar");
    ///     }
    /// });
    ///
    /// ```
    pub fn get_invalid_nodes(&self) -> Vec<&GameNode> {
        let mut invalids = self
            .nodes
            .iter()
            .filter(|node| {
                node.tokens.iter().any(|t| match t {
                    SgfToken::Invalid(_) => true,
                    _ => false,
                })
            })
            .collect::<Vec<_>>();
        self.variations.iter().for_each(|variation| {
            let tmp = variation.get_invalid_nodes();
            invalids.extend(tmp);
        });
        invalids
    }

    /// Checks if this GameTree has any variations
    pub fn has_variations(&self) -> bool {
        !self.variations.is_empty()
    }

    /// Counts number of variations in the GameTree
    pub fn count_variations(&self) -> usize {
        self.variations.len()
    }

    /// Get max length of a variation
    ///
    /// ```rust
    /// use sgf_parser::*;
    ///
    /// let tree: GameTree = parse("(;B[dc];W[ef](;B[aa])(;B[cc];W[ee]))").unwrap();
    ///
    /// assert_eq!(tree.get_varation_length(0).unwrap(), 1);
    /// assert_eq!(tree.get_varation_length(1).unwrap(), 2);
    /// ```
    pub fn get_varation_length(&self, variation: usize) -> Result<usize, SgfError> {
        if let Some(variation) = self.variations.get(variation) {
            Ok(variation.count_max_nodes())
        } else {
            Err(SgfErrorKind::VariationNotFound.into())
        }
    }

    /// Gets an iterator for the GameTree
    ///
    /// ```rust
    /// use sgf_parser::*;
    ///
    /// let tree: GameTree = parse("(;B[dc];W[ef](;B[aa])(;B[cc];W[ee]))").unwrap();
    ///
    /// let mut iter = tree.iter();
    ///
    /// assert_eq!(iter.count_variations(), 2);
    /// assert!(iter.pick_variation(1).is_ok());
    ///
    /// let mut count = 0;
    /// iter.for_each(|node| {
    ///     assert!(!node.tokens.is_empty());
    ///     count += 1;
    /// });
    ///
    /// assert_eq!(count, tree.count_max_nodes());
    /// ```
    pub fn iter(&self) -> GameTreeIterator<'_> {
        GameTreeIterator::new(self)
    }
}

impl Into<String> for &GameTree {
    fn into(self) -> String {
        let nodes = self
            .nodes
            .iter()
            .map(|n| -> String { n.into() })
            .collect::<String>();
        let variations = self
            .variations
            .iter()
            .map(|n| -> String { n.into() })
            .collect::<String>();
        format!("({}{})", nodes, variations)
    }
}

impl Into<String> for GameTree {
    fn into(self) -> String {
        (&self).into()
    }
}

pub struct GameTreeIterator<'a> {
    tree: &'a GameTree,
    index: usize,
    variation: usize,
}

impl<'a> GameTreeIterator<'a> {
    fn new(game_tree: &'a GameTree) -> Self {
        GameTreeIterator {
            tree: game_tree,
            index: 0,
            variation: 0,
        }
    }

    /// Checks if the current `GameTree` has any variations
    pub fn has_variations(&self) -> bool {
        self.tree.has_variations()
    }

    /// Counts number of variations in the current `GameTree`
    pub fn count_variations(&self) -> usize {
        self.tree.count_variations()
    }

    /// Picks a varation in the current `GameTree` to continue with, once the nodes haves been exhausted
    pub fn pick_variation(&mut self, variation: usize) -> Result<usize, SgfError> {
        if variation < self.tree.variations.len() {
            self.variation = variation;
            Ok(self.variation)
        } else {
            Err(SgfErrorKind::VariationNotFound.into())
        }
    }
}

impl<'a> Iterator for GameTreeIterator<'a> {
    type Item = &'a GameNode;

    fn next(&mut self) -> Option<&'a GameNode> {
        match self.tree.nodes.get(self.index) {
            Some(node) => {
                self.index += 1;
                Some(&node)
            }
            None => {
                if !self.tree.variations.is_empty() {
                    self.tree = &self.tree.variations[self.variation];
                    self.index = 0;
                    self.variation = 0;
                    self.next()
                } else {
                    None
                }
            }
        }
    }
}
