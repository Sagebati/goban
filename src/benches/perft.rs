#[macro_use]
extern crate bencher;

use goban::rules::game::{Game, Move, GobanSizes};
use bencher::Bencher;
use goban::rules::Rule::Japanese;


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

pub fn perft_bench(bench: &mut Bencher){
    let g = Game::new(GobanSizes::Nineteen, Japanese);
    let n = perft(&g,3);
    println!("number :{}", n)
}


benchmark_group!(benches, perft_bench);
benchmark_main!(benches);