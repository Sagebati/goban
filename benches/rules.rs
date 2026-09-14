//! Legality checks in isolation, on fixed positions: move generation, each
//! individual rule check swept over every empty point, and SGF parsing.

mod common;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use goban::pieces::stones::Stone;
use goban::rules::game::Game;
use goban::rules::{CHINESE, JAPANESE};

use common::{
    fragmented_board, ko_position, midgame_for_size, shusaku_phases, RULES, SGF_FILES, SIZES,
};

/// `legals()` on the same three positions under each rule set.
fn legals_by_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("rules/legals");
    for (rule_name, rule) in RULES {
        for (phase, game) in shusaku_phases(rule) {
            group.bench_with_input(BenchmarkId::new(rule_name, phase), &game, |b, g| {
                b.iter(|| g.legals().count())
            });
        }
    }
    group.finish();
}

/// `legals()` on a seeded mid-game position for each board size.
fn legals_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("rules/legals_by_size");
    for (size_name, size) in SIZES {
        let game = midgame_for_size(size, CHINESE);
        group.throughput(Throughput::Elements(game.pseudo_legals().count() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size_name), &game, |b, g| {
            b.iter(|| g.legals().count())
        });
    }
    group.finish();
}

/// Each rule check on its own, applied to every empty point of the position.
fn checks_sweep(c: &mut Criterion) {
    let mut group = c.benchmark_group("rules/check_sweep");
    for (phase, game) in shusaku_phases(CHINESE) {
        let empty: Vec<_> = game.pseudo_legals().collect();
        let stones: Vec<Stone> = empty
            .iter()
            .map(|&coord| Stone {
                coord,
                color: game.turn(),
            })
            .collect();
        group.throughput(Throughput::Elements(empty.len() as u64));

        group.bench_function(BenchmarkId::new("check_point", phase), |b| {
            b.iter(|| {
                empty
                    .iter()
                    .filter(|&&p| game.check_point(p).is_none())
                    .count()
            })
        });
        group.bench_function(BenchmarkId::new("check_suicide", phase), |b| {
            b.iter(|| stones.iter().filter(|&&s| game.check_suicide(s)).count())
        });
        group.bench_function(BenchmarkId::new("check_ko", phase), |b| {
            b.iter(|| stones.iter().filter(|&&s| game.check_ko(s)).count())
        });
        group.bench_function(BenchmarkId::new("check_super_ko", phase), |b| {
            b.iter(|| stones.iter().filter(|&&s| game.check_super_ko(s)).count())
        });
        group.bench_function(BenchmarkId::new("check_eye", phase), |b| {
            b.iter(|| stones.iter().filter(|&&s| game.check_eye(s)).count())
        });
        group.bench_function(BenchmarkId::new("will_capture", phase), |b| {
            b.iter(|| empty.iter().filter(|&&p| game.will_capture(p)).count())
        });
    }
    group.finish();
}

/// Adversarial boards: a live ko, and a board made of hundreds of one-stone
/// chains. (Super ko is not measured on the ko board: with a two-board
/// history it short-circuits before doing any work.)
fn adversarial(c: &mut Criterion) {
    let mut group = c.benchmark_group("rules/adversarial");

    for (rule_name, rule) in [("chinese", CHINESE), ("japanese", JAPANESE)] {
        let ko = ko_position(rule);
        group.bench_with_input(BenchmarkId::new("ko/legals", rule_name), &ko, |b, g| {
            b.iter(|| g.legals().count())
        });
        let retake = Stone {
            coord: (9, 9),
            color: ko.turn(),
        };
        group.bench_with_input(BenchmarkId::new("ko/check_ko", rule_name), &ko, |b, g| {
            b.iter(|| g.check_ko(retake))
        });
    }

    for (size_name, size) in SIZES {
        let game = fragmented_board(size, CHINESE);
        group.bench_with_input(
            BenchmarkId::new("fragmented/legals", size_name),
            &game,
            |b, g| b.iter(|| g.legals().count()),
        );
    }
    group.finish();
}

/// Parsing an SGF file and replaying it into a `Game`.
fn sgf_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("rules/from_sgf");
    for (name, sgf) in SGF_FILES {
        group.bench_function(name, |b| b.iter(|| Game::from_sgf(sgf).unwrap()));
    }
    group.finish();
}

criterion_group!(
    benches,
    legals_by_rule,
    legals_by_size,
    checks_sweep,
    adversarial,
    sgf_parse
);
criterion_main!(benches);
