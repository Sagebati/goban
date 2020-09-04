#[macro_use]
extern crate criterion;

use criterion::Criterion;

use goban::rules::game::Game;

pub fn deadstones() {
    let state = Game::from_sgf(include_str!("../sgf/ShusakuvsInseki.sgf")).unwrap();
    state.dead_stones();
}

pub fn deadbench(_c: &mut Criterion) {
    let c = Criterion::default();
    c.sample_size(10)
        .bench_function("dead_stones_mcts", |b| b.iter(|| deadstones()));
}

criterion_group!(benches, deadbench);
criterion_main!(benches);
