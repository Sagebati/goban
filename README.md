# Goban

**Library to play with a rusty "Goban" (name of the board where we play Go ! )** 

**Use the version > 5.0 because in a bug detecting dead stones and in Ko detection**

*Channel : stable*

**Only contains move generation, and rules there is no IA, neither
front-end.**

rust cargo features: 
- **thread-safe** // for using Arc instead of Rc for thread safety. Decrease perfs ! 
- **history**     // each game will have his all history so you can iterate over it. Decrease perfs !

Thanks to some help in profiling and optimisation we can run a playout randomly of an entire game in 7 ms (i7u 3.0 Ghz) (before it was 600ms ) ! 


Example :

```{rust}
use crate::goban::rules::*;
use crate::goban::rules::game::*;
use rand::seq::IteratorRandom;

let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
let mut i = 35;
while !g.over() && i != 0 {
   g.play(
        // legals return an iterator on (x,y) points
       g.legals()
           .choose(&mut rand::thread_rng())
           .map(|point| Move::Play(point.0,point.1))
           .unwrap());
   i -= 1;
   println!("{}", g);
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
- Boards of differents sizes (4x5 , 4x9) Limited  to (19x19)

## What is not in point:
- Handling dead stones at the end of the game.

