//! Dead-stone estimation (`deadstones` feature).
//!
//! `Game::from_sgf` sets the outcome from the `RE[]` property, so on those
//! games `dead_stones` finds the game already over and never runs the MCTS
//! playout. That fast path is measured under `finished_game`. To exercise the
//! MCTS itself, `mcts` uses an unfinished endgame position and a reduced
//! simulation budget so a sample stays in the seconds range. The MCTS draws
//! from an unseeded RNG, so expect more variance there than elsewhere.

mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use goban::rules::game::Game;
use goban::rules::CHINESE;
use std::time::Duration;

use common::{shusaku_at_move, SGF_FILES};

fn finished_game(c: &mut Criterion) {
    let mut group = c.benchmark_group("deadstones/finished_game");
    for (name, sgf) in SGF_FILES {
        let game = Game::from_sgf(sgf).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(name), &game, |b, g| {
            b.iter(|| g.dead_stones())
        });
    }
    group.finish();
}

fn mcts(c: &mut Criterion) {
    let mut group = c.benchmark_group("deadstones/mcts");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(20));
    let endgame = shusaku_at_move(320, CHINESE);
    for simulations in [1usize, 5, 20] {
        group.bench_with_input(
            BenchmarkId::from_parameter(simulations),
            &simulations,
            |b, &n| b.iter(|| endgame.dead_stones_wth_simulations(n)),
        );
    }
    group.finish();
}

criterion_group!(benches, finished_game, mcts);
criterion_main!(benches);
