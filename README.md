# Goban

**Library to play with a rusty "Goban" (name of the board where we play Go !)**

**Use the version > 0.5.0 because in a bug detecting dead stones and in Ko detection**

*Channel : stable*

**Only contains move generation, and rules there is no IA, neither front-end.**

Features:

- **history**     // each game will have his all history so you can iterate over it. Decrease perfs !

Thanks to some help in profiling and optimisation we can run a playout randomly of an entire game in 3 ms (i7u 3.0
Ghz) (before it was 600ms ) !

## Example

### Get legals moves and plays some random.

The most important struct is Game who has all you need to create and manages go games.

 ```
use crate::goban::rules::*;
use crate::goban::rules::game::*;
use rand::seq::IteratorRandom;
 use goban::rules::game_builder::GameBuilder;

let mut g = Game::builder()
    .size((19,19))
    .rule(CHINESE)
//  .komi(7.5)  Komi is hardcoded for each rule, but can be override like this.
    .build().unwrap();

let mut i = 35;
while !g.is_over() && i != 0 {
       g.play(
          // legals return an iterator of (x,y) points (lazy)
          g.legals()
                .choose(&mut rand::thread_rng())
                .map(|point| Move::Play(point.0,point.1))
                .unwrap());
        i -= 1;
        g.display_goban();
        // None if the game is not finished
        println!("{:?}", g.outcome());
        // Inner array using row policy
        println!("{:?}", g.goban().vec());


}
#[cfg(feature = "history")]
{
            let mut iter_history = g.history().iter();
            println!("{:?}", iter_history.next().unwrap());
            println!("{:?}", iter_history.next_back().unwrap())
}

 ```

```{bash}
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


## What works
- Capturing stones
- Playing
- Passing
- Resigning
- Implementation to count points
- Printing a *pretty* unicode Board on the terminal !
- SGF Import
- Generate legals moves (Handling Ko detection, Suicide moves, Super ko)
- Japanese Rules
- Chinese Rules
- Boards of different sizes (4x5 , 4x9) Limited  to (19x19) (Due to Zobrist hashing) cas be easy increased in a fork. 
- *Experimental* dead stones detection with MCTS rollouts.

## Waiting for
- Const generics or maybe not (benchmark with a Goban with backed array is slightly faster)
- Into-Iter for arrays for removing arrayvec dep

