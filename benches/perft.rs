#[macro_use]
extern crate criterion;

use criterion::Criterion;
use goban::rules::game::Game;
use goban::rules::Rule::{Chinese, Japanese};
use goban::rules::{GobanSizes, Move};
use rand::prelude::SliceRandom;
use rand::thread_rng;

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

pub fn fast_play_random(state: &Game) -> Move {
    let mut v: Vec<_> = state.pseudo_legals().collect();
    v.shuffle(&mut thread_rng());

    for l in v.into_iter().filter(|&point| state.check_point(point).is_none()) {
        if !state.goban().is_point_an_eye(l, state.turn().stone_color()) {
            return l.into();
        }
    }
    Move::Pass
}

pub fn fast_play_game() {
    let mut g = Game::new(GobanSizes::Nineteen, Chinese);
    while !g.is_over() {
        g.play(fast_play_random(&g));
    }
}

pub fn play_random(state: &Game) -> Move {
    let mut legals = state.legals().collect::<Vec<_>>();
    legals.shuffle(&mut thread_rng());
    for l in legals {
        if !state.goban().is_point_an_eye(l, state.turn().stone_color()) {
            return l.into();
        }
    }
    Move::Pass
}

pub fn play_game() {
    let mut g = Game::new(GobanSizes::Nineteen, Chinese);
    while !g.is_over() {
        g.play(play_random(&g));
    }
}

pub fn perft_bench(_c: &mut Criterion) {
    let g = Game::new(GobanSizes::Nineteen, Japanese);
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

pub fn game_play_bench(_c: &mut Criterion) {
    let criterion: Criterion = Default::default();
    criterion
        .sample_size(20)
        .bench_function("game_play", |b| b.iter(play_game))
        .bench_function("fast_play_game", |b| b.iter(fast_play_game));
}

criterion_group!(benches, game_play_bench);
criterion_main!(benches);
