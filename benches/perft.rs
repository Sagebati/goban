#[macro_use]
extern crate criterion;

use criterion::Criterion;
use rand::prelude::{SliceRandom, ThreadRng};
use rand::thread_rng;

use goban::pieces::stones::Stone;
use goban::rules::game::Game;
use goban::rules::{GobanSizes, Move, Rule, CHINESE, JAPANESE};
use goban::rules::Move::Play;

pub fn perft(pos: &Game, depth: u8) -> u64 {
    if depth < 1 {
        1
    } else {
        let moves = pos.legals();

        if depth == 1 {
            moves.count() as u64
        } else {
            moves
                .map(|m| {
                    let mut child = pos.clone();
                    child.play(Move::Play(m.0, m.1));
                    perft(&child, depth - 1)
                })
                .sum()
        }
    }
}

pub fn fast_play_random(state: &Game, thread_rng: &mut ThreadRng) -> Move {
    let mut v: Vec<_> = state.pseudo_legals().collect();
    v.shuffle(thread_rng);

    for coordinates in v
        .into_iter()
        .filter(|&point| state.check_point(point).is_none())
    {
        if !state.check_eye(Stone {
            coordinates,
            color: state.turn().stone_color(),
        }) {
            return coordinates.into();
        }
    }
    Move::Pass
}

pub fn fast_play_game(rule: Rule) {
    let mut g = Game::new(GobanSizes::Nineteen, rule);
    while !g.is_over() {
        g.play(fast_play_random(&g, &mut thread_rng()));
    }
}

pub fn play_random(state: &Game) -> Move {
    let mut legals = state.legals().collect::<Vec<_>>();
    legals.shuffle(&mut thread_rng());
    for l in legals {
        if !state.check_eye(Stone {
            coordinates: l,
            color: state.turn().stone_color(),
        }) {
            return l.into();
        }
    }
    Move::Pass
}

pub fn play_game() {
    let mut g = Game::new(GobanSizes::Nineteen, CHINESE);
    while !g.is_over() {
        g.play(play_random(&g));
    }
}

pub fn perft_bench(_c: &mut Criterion) {
    let g = Game::new(GobanSizes::Nineteen, JAPANESE);
    let deep = 2;
    let criterion: Criterion = Default::default();
    criterion.sample_size(10).bench_function_over_inputs(
        "perft",
        move |b, size| {
            b.iter(|| {
                perft(&g, *size);
            })
        },
        (1..=deep).into_iter(),
    );
}

fn some_plays_from_sgf() {
    let moves_sgf = vec![
        Move::Play(16, 13),
        Move::Play(16, 11),
        Move::Play(14, 12),
        Move::Play(14, 11),
        Move::Play(13, 11),
        Move::Play(13, 12),
        Move::Play(14, 13),
        Move::Play(14, 10),
        Move::Play(12, 12),
        Move::Play(13, 13),
        Move::Play(14, 14),
        Move::Play(12, 13),
        Move::Play(14, 15),
        Move::Play(11, 12),
        Move::Play(12, 11),
        Move::Play(11, 13),
        Move::Play(12, 9),
        Move::Play(13, 8),
        Move::Play(13, 9),
        Move::Play(14, 9),
        Move::Play(14, 8),
        Move::Play(12, 8),
        Move::Play(11, 11),
        Move::Play(15, 8),
        Move::Play(14, 7),
        Move::Play(13, 6),
        Move::Play(15, 7),
        Move::Play(16, 8),
        Move::Play(16, 7),
        Move::Play(17, 7),
        Move::Play(17, 6),
        Move::Play(17, 8),
        Move::Play(14, 5),
        Move::Play(16, 4),
        Move::Play(12, 6),
        Move::Play(11, 8),
        Move::Play(13, 5),
        Move::Play(10, 11),
        Move::Play(10, 10),
        Move::Play(11, 10),
        Move::Play(13, 2),
        Move::Play(11, 9),
        Move::Play(16, 2),
        Move::Play(15, 2),
        Move::Play(15, 1),
        Move::Play(14, 1),
        Move::Play(16, 1),
        Move::Play(13, 3),
        Move::Play(12, 3),
        Move::Play(12, 2),
        Move::Play(13, 1),
        Move::Play(14, 2),
        Move::Play(13, 4),
        Move::Play(12, 1),
        Move::Play(13, 0),
        Move::Play(14, 0),
        Move::Play(14, 3),
        Move::Play(17, 4),
        Move::Play(17, 5),
        Move::Play(18, 1),
        Move::Play(2, 13),
        Move::Play(2, 11),
        Move::Play(4, 12),
        Move::Play(3, 13),
        Move::Play(3, 12),
        Move::Play(2, 12),
        Move::Play(3, 14),
        Move::Play(4, 13),
        Move::Play(4, 14),
        Move::Play(5, 13),
        Move::Play(2, 14),
        Move::Play(5, 14),
        Move::Play(4, 15),
        Move::Play(2, 6),
        Move::Play(5, 15),
        Move::Play(6, 15),
        Move::Play(6, 16),
        Move::Play(7, 15),
        Move::Play(7, 16),
        Move::Play(8, 16),
        Move::Play(6, 14),
        Move::Play(5, 12),
        Move::Play(3, 10),
        Move::Play(2, 10),
        Move::Play(4, 10),
        Move::Play(6, 10),
        Move::Play(5, 11),
        Move::Play(7, 13),
        Move::Play(6, 11),
        Move::Play(7, 12),
        Move::Play(7, 11),
        Move::Play(3, 8),
        Move::Play(7, 14),
        Move::Play(8, 15),
        Move::Play(8, 14),
        Move::Play(9, 13),
        Move::Play(9, 12),
        Move::Play(8, 13),
        Move::Play(10, 14),
        Move::Play(9, 14),
        Move::Play(10, 12),
        Move::Play(9, 11),
        Move::Play(8, 11),
        Move::Play(9, 10),
        Move::Play(10, 13),
        Move::Play(12, 15),
        Move::Play(12, 14),
        Move::Play(13, 14),
        Move::Play(13, 15),
        Move::Play(11, 14),
        Move::Play(11, 15),
        Move::Play(8, 12),
        Move::Play(12, 14),
        Move::Play(8, 9),
        Move::Play(4, 8),
        Move::Play(3, 9),
        Move::Play(4, 9),
        Move::Play(6, 8),
        Move::Play(4, 6),
        Move::Play(3, 7),
        Move::Play(4, 7),
        Move::Play(6, 6),
        Move::Play(2, 5),
        Move::Play(3, 5),
        Move::Play(3, 6),
        Move::Play(1, 7),
        Move::Play(7, 9),
        Move::Play(7, 8),
        Move::Play(1, 6),
        Move::Play(2, 7),
        Move::Play(5, 5),
        Move::Play(2, 4),
        Move::Play(6, 5),
        Move::Play(7, 5),
        Move::Play(6, 9),
        Move::Play(5, 9),
        Move::Play(7, 10),
        Move::Play(5, 8),
        Move::Play(7, 4),
        Move::Play(8, 5),
        Move::Play(5, 2),
        Move::Play(7, 2),
        Move::Play(7, 3),
        Move::Play(6, 2),
        Move::Play(6, 3),
        Move::Play(5, 1),
        Move::Play(4, 2),
        Move::Play(4, 1),
        Move::Play(3, 2),
        Move::Play(2, 2),
        Move::Play(3, 1),
        Move::Play(2, 1),
        Move::Play(8, 2),
        Move::Play(8, 1),
        Move::Play(9, 4),
        Move::Play(9, 2),
        Move::Play(9, 9),
        Move::Play(10, 9),
        Move::Play(9, 8),
        Move::Play(8, 8),
        Move::Play(8, 10),
        Move::Play(10, 10),
        Move::Play(10, 7),
        Move::Play(11, 6),
        Move::Play(10, 6),
        Move::Play(12, 5),
        Move::Play(11, 5),
        Move::Play(12, 7),
        Move::Play(11, 4),
        Move::Play(8, 4),
        Move::Play(15, 4),
        Move::Play(15, 5),
        Move::Play(14, 6),
        Move::Play(14, 4),
        Move::Play(13, 3),
        Move::Play(16, 6),
        Move::Play(15, 4),
        Move::Play(16, 3),
        Move::Play(16, 5),
        Move::Play(14, 4),
        Move::Play(15, 6),
        Move::Play(15, 4),
        Move::Play(17, 2),
        Move::Play(18, 4),
        Move::Play(18, 5),
        Move::Play(18, 2),
        Move::Play(18, 3),
        Move::Play(17, 3),
        Move::Play(15, 0),
        Move::Play(16, 16),
        Move::Play(17, 15),
        Move::Play(17, 16),
        Move::Play(16, 15),
        Move::Play(14, 17),
        Move::Play(13, 17),
        Move::Play(14, 16),
        Move::Play(15, 16),
        Move::Play(15, 17),
        Move::Play(17, 17),
        Move::Play(18, 17),
        Move::Play(17, 18),
        Move::Play(12, 17),
        Move::Play(13, 16),
        Move::Play(13, 18),
        Move::Play(11, 17),
        Move::Play(10, 17),
        Move::Play(11, 18),
        Move::Play(10, 16),
        Move::Play(11, 16),
        Move::Play(8, 17),
        Move::Play(5, 10),
        Move::Play(3, 11),
        Move::Play(4, 0),
        Move::Play(6, 1),
        Move::Play(17, 12),
        Move::Play(17, 11),
        Move::Play(4, 5),
        Move::Play(3, 4),
        Move::Play(8, 7),
        Move::Play(7, 7),
        Move::Play(7, 18),
        Move::Play(7, 17),
        Move::Play(6, 17),
        Move::Play(4, 17),
        Move::Play(5, 17),
        Move::Play(4, 16),
        Move::Play(5, 16),
        Move::Play(2, 16),
        Move::Play(1, 16),
        Move::Play(1, 17),
        Move::Play(1, 15),
        Move::Play(0, 17),
        Move::Play(2, 15),
        Move::Play(3, 16),
        Move::Play(3, 18),
        Move::Play(2, 17),
        Move::Play(4, 18),
        Move::Play(1, 13),
        Move::Play(1, 18),
        Move::Play(2, 18),
        Move::Play(5, 18),
        Move::Play(1, 14),
        Move::Play(0, 16),
        Move::Play(8, 18),
        Move::Play(3, 17),
        Move::Play(6, 18),
        Move::Play(0, 18),
        Move::Play(8, 3),
        Move::Play(4, 3),
        Move::Play(4, 4),
        Move::Play(5, 4),
        Move::Play(2, 0),
        Move::Play(3, 0),
        Move::Play(5, 0),
        Move::Play(5, 3),
        Move::Play(4, 11),
        Move::Play(8, 6),
        Move::Play(7, 6),
        Move::Play(9, 5),
        Move::Play(11, 2),
        Move::Play(12, 0),
        Move::Play(11, 0),
        Move::Play(10, 1),
        Move::Play(10, 2),
        Move::Play(11, 1),
        Move::Play(9, 1),
        Move::Play(10, 0),
        Move::Play(9, 0),
        Move::Play(11, 0),
        Move::Play(16, 12),
        Move::Play(17, 13),
        Move::Play(15, 12),
        Move::Play(18, 11),
        Move::Play(18, 10),
        Move::Play(18, 12),
        Move::Play(17, 10),
        Move::Play(0, 14),
        Move::Play(1, 12),
        Move::Play(10, 18),
        Move::Play(9, 18),
        Move::Play(18, 7),
        Move::Play(18, 8),
        Move::Play(15, 13),
        Move::Play(13, 10),
        Move::Play(12, 10),
        Move::Play(5, 7),
        Move::Play(11, 3),
        Move::Play(10, 3),
        Move::Play(10, 4),
        Move::Play(0, 13),
        Move::Play(6, 12),
        Move::Play(6, 13),
        Move::Play(18, 3),
        Move::Play(7, 18),
        Move::Play(18, 6),
        Move::Play(0, 15),
        Move::Play(1, 5),
        Move::Play(1, 4),
        Move::Play(0, 14),
        Move::Play(5, 6),
        Move::Play(0, 15),
        Move::Play(2, 3),
        Move::Pass,
        Move::Play(15, 14),
        Move::Pass,
        Move::Play(16, 14),
        Move::Play(17, 14),
        Move::Play(15, 16),
        Move::Play(18, 15),
        Move::Play(18, 16),
        Move::Play(14, 18),
        Move::Play(15, 18),
        Move::Play(12, 18),
        Move::Play(14, 18),
        Move::Play(16, 17),
        Move::Play(10, 15),
        Move::Pass,
        Move::Pass,
    ];
    let handicap = vec![(3, 3), (3, 15), (9, 3), (9, 15), (15, 3), (15, 15)];
    let mut g = Game::new(GobanSizes::Nineteen, CHINESE);
    let inv_coord: Vec<usize> = (0..19).rev().collect();
    g.put_handicap(&handicap);
    for m in moves_sgf {
        let to_play = match m {
            Play(x, y) => {
                let x = x as usize;
                let y = y as usize;
                Play(inv_coord[x] as u8, y as u8)
            }
            m => m,
        };
        g.play(to_play);
    }
}

pub fn game_play_bench(_c: &mut Criterion) {
    let c = Criterion::default();
    c.sample_size(100)
        //.bench_function("game_play", |b| b.iter(play_game))
        .bench_function("fast_play_game_chinese", |b| {
            b.iter(|| fast_play_game(CHINESE))
        })
        .bench_function("fast_play_game_jap", |b| {
            b.iter(|| fast_play_game(JAPANESE))
        })
        .bench_function("play_sgf_game", |b| {
            b.iter(|| some_plays_from_sgf());
        });
}

criterion_group!(benches, game_play_bench);
criterion_main!(benches);
