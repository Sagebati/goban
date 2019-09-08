[![Build Status](https://travis-ci.com/mipli/sgf-parser.svg?branch=master)](https://travis-ci.com/mipli/sgf-parser)
[![Crate](https://img.shields.io/crates/v/sgf-parser.svg)](https://crates.io/crates/sgf-parser)

# SGF Parser

A [SGF](https://www.red-bean.com/sgf/index.html) Parser for Rust. Supports all SGF properties, and tree branching.

Using `pest` for the actual parsing part.

# Example usage
```rust
use sgf_parser::*;

let sgf_source = "(;EV[event]PB[black]PW[white]C[comment];B[aa])";
let tree: Result<GameTree, SgfError> = parse(sgf_source);

let tree = tree.unwrap();
let unknown_nodes = tree.get_unknown_nodes();
assert_eq!(unknown_nodes.len(), 0);

let invalid_nodes = tree.get_invalid_nodes();
assert_eq!(invalid_nodes.len(), 0);

tree.iter().for_each(|node| {
  assert!(!node.tokens.is_empty());
});

let sgf_string: String = tree.into();
assert_eq!(sgf_source, sgf_string());
```
