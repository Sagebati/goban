# Goban

**Library to play with a rusty goban** 

*Channel : stable*

Only contains move generation, and rules there is no IA, neither
front-end.

Exemple :

```{rust}
let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        let mut i = 35;
        while !g.legals().count() != 0 && i != 0 {
            g.play(
                &g.legals().map(|coord| Move::Play(coord.0, coord.1))
                    .choose(&mut rand::thread_rng())
                    .unwrap());
            i -= 1;
            println!("{}", g.goban().pretty_string());
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

**Is not ready**


## What works (or seem to works)
- Capturing stones
- Playing
- Passing
- Resigning
- Naive implementation to count points
- Printing an *ugly* ascii goban
- Generate legals moves
- Japanese Rules
- Chinese Rules

## In actual development
- Documentation
- Best workflow for passing

## TODO
- SGF imports

## What is not in point:
- Complete end game
- Seki handling
