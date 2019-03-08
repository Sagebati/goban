#[macro_use]
extern crate criterion;

use goban::rules::game::{Game, Move, GobanSizes};
use goban::rules::Rule::Japanese;
use criterion::Criterion;


pub fn perft(pos: &Game, depth: u8) -> u64 {
    if depth < 1 {
        1
    } else {
        let moves = pos.legals();

        if depth == 1 {
            moves.count() as u64
        } else {
            moves.map(|m| {
                let mut child = pos.clone();
                child.play(&Move::Play(m.0, m.1));
                perft(&child, depth - 1)
            }).sum()
        }
    }
}

pub fn perft_bench(_c: &mut Criterion) {
    let g = Game::new(GobanSizes::Nineteen, Japanese);
    let deep = 4;
    let criterion: Criterion = Default::default();
    criterion.sample_size(10).bench_function_over_inputs("perft",
                                                         move |b, size|
                                                             {
                                                                 b.iter(|| {
                                                                     perft(&g, *size);
                                                                 })
                                                             },
                                                         (0..deep).into_iter());
}


criterion_group!(benches, perft_bench);
criterion_main!(benches);