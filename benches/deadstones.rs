//! Dead-stone estimation (`deadstones` feature).
//!
//! `finished_game` runs the default number of playouts on the three shipped
//! SGF games. `playouts` scales the playout count on an unfinished endgame
//! position. The playouts draw from an unseeded RNG, so expect more variance
//! here than elsewhere.

mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use goban::rules::game::Game;
use goban::rules::CHINESE;
use std::time::Duration;

use common::{shusaku_at_move, SGF_FILES};

fn finished_game(c: &mut Criterion) {
    let mut group = c.benchmark_group("deadstones/finished_game");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(20));
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
