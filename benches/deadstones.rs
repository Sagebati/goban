#[macro_use]
extern crate criterion;

use criterion::Criterion;

use goban::rules::game::Game;
use goban::rules::GobanSizes;
use goban::rules::Rule::Japanese;

pub fn deadstones() {
    let state = Game::new(GobanSizes::Nineteen, Japanese);
    state.get_dead_stones();
}

pub fn deadbench(_c: &mut Criterion) {
    let c = Criterion::default();
    c.sample_size(50)
        .bench_function("dead_stones_mcts", |b| {
            b.iter(|| deadstones())
        });
}

criterion_group!(benches, deadbench);
criterion_main!(benches);
