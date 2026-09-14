Changelog
==

## Unreleased

### Breaking Changes

- `deadstones`: dead stones are now estimated by random playouts instead of an MCTS game. A chain
  without two eyes is dead when it is captured in more than 98% of the playouts; the `oxymcts`
  dependency is gone. `dead_stones_wth_simulations(n)` now takes a number of playouts.

### Fixes

- Chain storage reuses freed slots; a game creating more than 65535 chains no longer panics.
- Under rules that allow suicide (Tromp-Taylor, New Zealand), a lone stone suicide that captures
  nothing is rejected as a super ko violation, since it repeats the position.
- `get_floating_stones` returns the chains without two eyes, as documented.
- `check_eye` follows the usual definition of a true eye: at least three held diagonals in the
  centre, all of them on the edge and in the corner. It used to reject real eyes that had an empty
  or a single enemy diagonal.
- `play_for_verification` removed the suicided stones under the wrong rules.

### Added

- `Goban::chains_with_index`, `Goban::chain_liberties_at`, `Game::check_false_eye`.
- Criterion benchmarks split into `board`, `rules`, `playout` and `deadstones` suites.

## 0.20.0

### Breaking Changes

- Removed the GameBuilder::new() use ::default()

### Fixes

- Now correctly handles SGF with player turn 
- Sgf and the terminal representation where reversed now it's corrected
- Suicide is no longer mislabeled as a ko
- Correctly handling of Eq and Hash for Goban
- Correctly handles super ko

## 0.19.0

### Fixes

- #9 Bug in the Eq og Goban implementation only relying on the zobrist hash
- #10 The matrix returned a empty row.

### Refactor:

- Renamed multiple function from chain to group for clarity

## 0.18.0-beta

Release specialised in removing allocations
- The structure chain doesn't use hashset anymore and use bitvec instead
- optimised the push_wth_feedback method and the remove_chain method
- this version is in beta because bitvec is also in beta

## 0.17

- Uses now "linked list" for storing stones in chains. Doesn't need the "thread-safe" feature anymore because we don't
  use any Rc or Arc now. So now it's thread safe by default

- Performance improved for the play method on game. Others method didn't improve.
- reworked naming things
- updated README.md
- the main doc imported the README.md