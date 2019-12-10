# Goban

**Library to play with a rusty goban** 

**Use the version > 5.0 because in a bug detecting dead stones and in Ko detection**

*Channel : stable*


Only contains move generation, and rules there is no IA, neither
front-end.

Exemple :

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
.........
.........
.........
.........
.........
.........
.........
⚪........
.........


etc...
```


## What works
- Capturing stones
- Playing
- Passing
- Resigning
- Implementation to count points
- Printing an *ugly* ascii goban
- Generate legals moves (Handling Ko detection and Suicide moves)
- Japanese Rules
- Chinese Rules


## What is not in point:
- Handling dead stones at the end of the game.

## Actively develloped 
