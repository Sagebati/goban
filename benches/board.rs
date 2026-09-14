//! Board-level primitives: placing stones, captures, chain merges, hashing,
//! cloning, and scoring. Mutating benches use `iter_batched_ref` so neither
//! the clone of the fixture nor moving it in and out is measured.

mod common;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use goban::pieces::goban::Goban;
use goban::pieces::stones::Color;
use goban::rules::{GobanSizes, Move, CHINESE};

use common::{
    big_capture, four_way_merge, fragmented_board, shusaku_at_move, shusaku_phases, SIZES,
};

fn push(c: &mut Criterion) {
    let mut group = c.benchmark_group("board/push");

    for (size_name, size) in SIZES {
        let (h, w) = common::size_dims(size);
        let empty = Goban::new((h as u8, w as u8));
        let centre = ((h / 2) as u8, (w / 2) as u8);
        // A copy of the board plus one push: the copy is a memcpy of the
        // fixed-size goban, so it stays in the measurement on purpose.
        group.bench_with_input(
            BenchmarkId::new("copy_and_push/empty_board", size_name),
            &empty,
            |b, g| {
                b.iter(|| {
                    let mut goban = *g;
                    goban.push(centre, Color::Black);
                    goban.zobrist_hash()
                })
            },
        );
    }

    // A quiet move in the middle of a real game: no capture, at most a merge.
    let midgame = shusaku_at_move(150, CHINESE);
    let quiet = midgame.legals().next().unwrap();
    group.bench_function("play/midgame_quiet", |b| {
        b.iter_batched_ref(
            || midgame.clone(),
            |game| {
                game.play(Move::Play(quiet.0, quiet.1));
            },
            BatchSize::SmallInput,
        )
    });

    let (capture, capturing_move) = big_capture();
    group.bench_function("play/capture_171_stones", |b| {
        b.iter_batched_ref(
            || capture.clone(),
            |game| {
                game.play(capturing_move);
            },
            BatchSize::SmallInput,
        )
    });

    let (merge, merging_move) = four_way_merge();
    group.bench_function("play/merge_4_chains", |b| {
        b.iter_batched_ref(
            || merge.clone(),
            |game| {
                game.play(merging_move);
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("board/clone");
    for (phase, game) in shusaku_phases(CHINESE) {
        group.bench_with_input(BenchmarkId::new("game", phase), &game, |b, g| {
            b.iter(|| g.clone())
        });
        group.bench_with_input(BenchmarkId::new("goban", phase), game.goban(), |b, g| {
            b.iter(|| g)
        });
    }
    let fragmented = fragmented_board(GobanSizes::Nineteen, CHINESE);
    group.bench_function("goban/fragmented_19", |b| b.iter(|| fragmented.goban()));
    group.finish();
}

fn queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("board/queries");
    let positions: Vec<_> = shusaku_phases(CHINESE)
        .into_iter()
        .chain(std::iter::once((
            "fragmented_19",
            fragmented_board(GobanSizes::Nineteen, CHINESE),
        )))
        .collect();

    for (name, game) in &positions {
        let goban = game.goban();
        let stones: Vec<_> = goban.get_stones().map(|s| s.coord).collect();

        group.bench_function(BenchmarkId::new("chains", name), |b| {
            b.iter(|| goban.chains().count())
        });
        group.bench_function(BenchmarkId::new("get_stones", name), |b| {
            b.iter(|| goban.get_stones().count())
        });
        group.bench_function(BenchmarkId::new("get_liberties_all", name), |b| {
            b.iter(|| {
                stones
                    .iter()
                    .map(|&s| goban.get_liberties(s).count())
                    .sum::<usize>()
            })
        });
        group.bench_function(BenchmarkId::new("has_liberties_all", name), |b| {
            b.iter(|| stones.iter().filter(|&&s| goban.has_liberties(s)).count())
        });
    }
    group.finish();
}

fn scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("board/scoring");
    for (phase, game) in shusaku_phases(CHINESE) {
        let goban = game.goban();
        group.bench_function(BenchmarkId::new("calculate_score", phase), |b| {
            b.iter(|| game.calculate_score())
        });
        group.bench_function(BenchmarkId::new("calculate_territories", phase), |b| {
            b.iter(|| goban.calculate_territories())
        });
        group.bench_function(BenchmarkId::new("get_territories", phase), |b| {
            b.iter(|| {
                let (black, white) = goban.get_territories();
                black.count() + white.count()
            })
        });
    }
    group.finish();
}

criterion_group!(benches, push, clone, queries, scoring);
criterion_main!(benches);
