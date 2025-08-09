# Goban

A fast Rust library for working with a Goban (the board used to play Go). It is built with performance in mind.
The library can perform a full playout of a random game in ~1.5 ms while checking all legal moves.

In Go, there are different rule sets; at the moment only two are implemented:

- Chinese (area scoring)
- Japanese (territory scoring)

Adding more rules can be achieved fairly easily.

This crate contains move generation and rules only; it does not include an AI nor a front end.

Optional cargo features:

- history — keep the full history of Gobans so you can iterate over it (this decreases performance)
- deadstones — experimental dead-stone detection using MCTS rollouts; works only when borders are closed

## Example

### Get legal moves and play some at random

The central type is Game, which has all you need to create and manage Go games.

```
use goban::rules::*;
use goban::rules::game::Game;
use rand::seq::IteratorRandom;
// use goban::rules::game_builder::GameBuilder;

let mut g = Game::builder()
    .size((19, 19))
    .rule(CHINESE)
    // Komi is pre-set for each rule but can be overridden like this:
    // .komi(7.5)
    .build()
    .unwrap();

let mut i = 35;
while !g.is_over() && i != 0 {
    // legals returns an iterator of (x, y) points (lazy)
    if let Some((x, y)) = g.legals().choose(&mut rand::thread_rng()) {
        g.play(Move::Play(x, y));
    } else {
        break; // no legal moves
    }
    i -= 1;

    g.display_goban();
    // None if the game is not finished
    println!("{:?}", g.outcome());
    // Inner array using row-major order
    println!("{:?}", g.goban().to_vec());
}

#[cfg(feature = "history")]
{
    // Access first and last positions in the history
    let mut iter_history = g.history().iter();
    println!("{:?}", iter_history.next().unwrap());
    println!("{:?}", iter_history.next_back().unwrap());
}
```

```
┏┯┯┯┯┯┯┯┓
┠┼┼┼┼┼┼┼┨
┠┼┼┼┼┼┼┼┨
┠┼┼┼┼┼┼┼┨
┠┼┼┼┼┼┼┼┨
┠┼┼┼┼┼┼┼┨
┠┼┼┼┼┼┼┼┨
○┼┼┼┼┼┼┼┨
┗┷┷┷┷┷┷┷┛

etc...
```

## Features
- Pretty Unicode board printing in the terminal
- SGF import (Game::from_sgf)
- Generate legal moves (handles ko detection, suicide moves, and superko)
- Japanese and Chinese rules
- Boards of different sizes (e.g., 4x5, 4x9); limited up to 19x19 due to the Zobrist hashing table size
- Experimental dead-stone detection with MCTS rollouts (feature: deadstones)
