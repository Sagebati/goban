//! Whole-game benchmarks: seeded random playouts, perft, and replaying a real
//! game move by move. These are the numbers an MCTS or search user cares about.

mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use goban::rules::game::Game;
use goban::rules::{GobanSizes, Move, CHINESE};
use std::time::Duration;

use common::{random_game, shusaku_moves, RULES, SEED, SIZES};

/// Count the leaf nodes of the move tree `depth` plies deep.
pub fn perft(state: &Game, depth: u8) -> u64 {
    if depth < 1 {
        1
    } else {
        let moves = state.legals();

        if depth == 1 {
            moves.count() as u64
        } else {
            moves
                .map(|m| {
                    let mut child = state.clone();
                    child.play(Move::Play(m.0, m.1));
                    perft(&child, depth - 1)
                })
                .sum()
        }
    }
}

/// Random games from an empty board. The seed is fixed per iteration, so every
/// sample plays the same game and the throughput is in moves per second.
fn playout_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("playout/random_game");
    for (rule_name, rule) in RULES {
        for (size_name, size) in SIZES {
            let moves = random_game(size, rule, SEED);
            group.throughput(Throughput::Elements(moves as u64));
            group.bench_function(BenchmarkId::new(rule_name, size_name), |b| {
                b.iter(|| random_game(size, rule, SEED))
            });
        }
    }
    group.finish();
}

/// Replay every move of the Shusaku game through `Game::play`.
fn replay_bench(c: &mut Criterion) {
    let moves = shusaku_moves();
    let mut group = c.benchmark_group("playout/replay_sgf");
    group.throughput(Throughput::Elements(moves.len() as u64));
    group.bench_function("shusaku", |b| {
        b.iter(|| {
            let mut game = Game::new(GobanSizes::Nineteen, CHINESE);
            for &m in &moves {
                game.play(m);
            }
            game
        })
    });
    group.finish();
}

fn perft_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("playout/perft");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(12));

    let root_9 = Game::new(GobanSizes::Nine, CHINESE);
    group.bench_function("9x9/depth_3", |b| b.iter(|| perft(&root_9, 3)));

    let root_19 = Game::new(GobanSizes::Nineteen, CHINESE);
    group.bench_function("19x19/depth_2", |b| b.iter(|| perft(&root_19, 2)));
    group.bench_function("19x19/depth_3", |b| b.iter(|| perft(&root_19, 3)));

    group.finish();
}

criterion_group!(benches, playout_bench, replay_bench, perft_bench);
criterion_main!(benches);
