//! Fixtures shared by every benchmark: real game positions loaded from the SGF
//! files in `sgf/`, deterministic random positions for the 9x9 and 13x13
//! boards, and a few hand-built adversarial boards.
//!
//! Everything here is built *outside* the timed closures so that the benches
//! measure a single operation of the library and not the fixture setup.
#![allow(dead_code)]

use goban::pieces::stones::{Color, Stone};
use goban::rules::game::Game;
use goban::rules::{GobanSizes, Move, Rule, CHINESE, JAPANESE, TROMP_TAYLOR};
use rand::prelude::*;
use rand::rngs::StdRng;
use sgf_parser::{Action, SgfToken};

pub const SHUSAKU_SGF: &str = include_str!("../../sgf/ShusakuvsInseki.sgf");
pub const SGF_1: &str = include_str!("../../sgf/sgf_1.sgf");
pub const SGF_2_2HA: &str = include_str!("../../sgf/sgf_2_2ha.sgf");

/// Every SGF file shipped with the crate, with a short name for benchmark ids.
pub const SGF_FILES: [(&str, &str); 3] = [
    ("shusaku", SHUSAKU_SGF),
    ("sgf_1", SGF_1),
    ("sgf_2_2ha", SGF_2_2HA),
];

/// Rule sets whose legality checks differ enough to be worth comparing.
pub const RULES: [(&str, Rule); 3] = [
    ("chinese", CHINESE),
    ("japanese", JAPANESE),
    ("tromp_taylor", TROMP_TAYLOR),
];

pub const SIZES: [(&str, GobanSizes); 3] = [
    ("9", GobanSizes::Nine),
    ("13", GobanSizes::Thirteen),
    ("19", GobanSizes::Nineteen),
];

/// Fixed seed so every run of the random benches plays the same games.
pub const SEED: u64 = 0x5EED_60BA_2024;

/// Extract the main-line moves of an SGF game, in play order.
pub fn sgf_moves(sgf: &str) -> Vec<Move> {
    let tree = sgf_parser::parse(sgf).expect("valid sgf");
    tree.iter()
        .skip(1)
        .filter_map(|node| match node.tokens.first() {
            Some(SgfToken::Move { action, .. }) => Some(match *action {
                Action::Move(col, line) => Move::Play(line - 1, col - 1),
                Action::Pass => Move::Pass,
            }),
            _ => None,
        })
        .collect()
}

/// Moves of the Shusaku vs Gennan Inseki game (no handicap, 19x19).
pub fn shusaku_moves() -> Vec<Move> {
    sgf_moves(SHUSAKU_SGF)
}

/// The Shusaku game replayed up to (and including) move `n`, under `rule`.
///
/// Unlike `Game::from_sgf` this leaves `outcome` unset, so the game is not
/// considered over and the position can be used for move generation.
pub fn shusaku_at_move(n: usize, rule: Rule) -> Game {
    let moves = shusaku_moves();
    let mut game = Game::new(GobanSizes::Nineteen, rule);
    for &m in moves.iter().take(n) {
        game.play(m);
    }
    game
}

/// Three phases of the Shusaku game, for benches that depend on how full the
/// board is and how long the history is.
pub fn shusaku_phases(rule: Rule) -> Vec<(&'static str, Game)> {
    vec![
        ("opening_50", shusaku_at_move(50, rule)),
        ("midgame_150", shusaku_at_move(150, rule)),
        ("endgame_300", shusaku_at_move(300, rule)),
    ]
}

/// Pick a random legal move that neither fills one's own eye nor commits
/// suicide (legal under Tromp-Taylor and New Zealand rules, but a random
/// policy that plays suicides never finishes a game).
pub fn random_move(game: &Game, rng: &mut StdRng) -> Move {
    let mut legals: Vec<_> = game.legals().collect();
    legals.shuffle(rng);
    legals
        .into_iter()
        .find(|&coord| {
            let stone = Stone {
                coord,
                color: game.turn(),
            };
            !game.check_eye(stone) && !game.check_suicide(stone)
        })
        .map(Move::from)
        .unwrap_or(Move::Pass)
}

/// Play `n` random moves from an empty board with a fixed seed.
/// The same `(size, rule, n, seed)` always yields the same position.
pub fn random_position(size: GobanSizes, rule: Rule, n: usize, seed: u64) -> Game {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut game = Game::new(size, rule);
    for _ in 0..n {
        if game.is_over() {
            break;
        }
        game.play(random_move(&game, &mut rng));
    }
    game
}

/// A mid-game position for any board size: roughly a third of the points
/// filled by seeded random play.
pub fn midgame_for_size(size: GobanSizes, rule: Rule) -> Game {
    let (h, w) = size_dims(size);
    random_position(size, rule, h * w / 3, SEED)
}

pub fn size_dims(size: GobanSizes) -> (usize, usize) {
    match size {
        GobanSizes::Nine => (9, 9),
        GobanSizes::Thirteen => (13, 13),
        GobanSizes::Nineteen => (19, 19),
        GobanSizes::Custom(h, w) => (h, w),
    }
}

/// Play a full random game from an empty board with a fixed seed and return
/// the number of moves played (passes included). Like most playout policies
/// the game is cut off after a few times the number of points on the board.
pub fn random_game(size: GobanSizes, rule: Rule, seed: u64) -> usize {
    let (h, w) = size_dims(size);
    let max_moves = 4 * h * w;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut game = Game::new(size, rule);
    let mut moves = 0;
    while !game.is_over() && moves < max_moves {
        game.play(random_move(&game, &mut rng));
        moves += 1;
    }
    moves
}

/// 19x19 board where White fills rows 0..=8 (171 stones in one chain) and
/// Black fills row 9 except `(9, 0)`, the chain's only liberty.
/// Black to play `(9, 0)` captures the whole chain in one move.
pub fn big_capture() -> (Game, Move) {
    let mut builder = Game::builder();
    builder.size((19, 19)).rule(CHINESE).turn(Color::Black);
    for x in 0..9u8 {
        for y in 0..19u8 {
            builder.add(Stone {
                coord: (x, y),
                color: Color::White,
            });
        }
    }
    for y in 1..19u8 {
        builder.add(Stone {
            coord: (9, y),
            color: Color::Black,
        });
    }
    (builder.build().unwrap(), Move::Play(9, 0))
}

/// 19x19 board with four separate Black chains (arms of a cross around the
/// tengen). Playing the centre point merges the four chains into one.
pub fn four_way_merge() -> (Game, Move) {
    let mut builder = Game::builder();
    builder.size((19, 19)).rule(CHINESE).turn(Color::Black);
    for d in 1..=4u8 {
        for coord in [(9 - d, 9), (9 + d, 9), (9, 9 - d), (9, 9 + d)] {
            builder.add(Stone {
                coord,
                color: Color::Black,
            });
        }
    }
    (builder.build().unwrap(), Move::Play(9, 9))
}

/// Board covered with isolated single-stone chains: Black on (even, even),
/// White on (odd, odd). Stresses anything that walks chains or merges them.
pub fn fragmented_board(size: GobanSizes, rule: Rule) -> Game {
    let (h, w) = size_dims(size);
    let mut builder = Game::builder();
    builder
        .size((h as u8, w as u8))
        .rule(rule)
        .turn(Color::Black);
    for x in 0..h as u8 {
        for y in 0..w as u8 {
            let color = match (x % 2, y % 2) {
                (0, 0) => Color::Black,
                (1, 1) => Color::White,
                _ => continue,
            };
            builder.add(Stone {
                coord: (x, y),
                color,
            });
        }
    }
    builder.build().unwrap()
}

/// A simple ko in the centre of a 19x19 board, right after Black captured.
/// White to play; retaking at `(9, 9)` is illegal by the ko rule.
pub fn ko_position(rule: Rule) -> Game {
    let mut builder = Game::builder();
    builder.size((19, 19)).rule(rule).turn(Color::Black);
    for coord in [(9, 8), (8, 9), (10, 9)] {
        builder.add(Stone {
            coord,
            color: Color::Black,
        });
    }
    for coord in [(8, 10), (10, 10), (9, 11), (9, 9)] {
        builder.add(Stone {
            coord,
            color: Color::White,
        });
    }
    let mut game = builder.build().unwrap();
    game.play(Move::Play(9, 10));
    game
}
