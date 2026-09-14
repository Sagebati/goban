//! Real 19x19 games with a counted result, replayed from SGF and scored.
//! See `sgf/results/README.md` for where they come from.

use goban::pieces::stones::Color;
use goban::rules::game::Game;
use goban::rules::{EndGame, ScoreRules};
use std::collections::HashSet;

/// Games played out to the end: every stone on the board is alive.
const ALL_STONES: [(&str, EndGame); 3] = [
    (
        include_str!("../sgf/results/fox_2018-06-13_evc_vs_jinmao_W+2.5.sgf"),
        EndGame::WinnerByScore(Color::White, 2.5),
    ),
    (
        include_str!("../sgf/results/fox_2017-07-16_guaitong_vs_lilong_W+2.5.sgf"),
        EndGame::WinnerByScore(Color::White, 2.5),
    ),
    (
        include_str!("../sgf/results/fox_2018-04-21_coreaface_vs_bensondarr_W+0.5.sgf"),
        EndGame::WinnerByScore(Color::White, 0.5),
    ),
];

/// Games that stopped with dead stones still on the board.
const WITH_DEAD_STONES: [(&str, EndGame); 4] = [
    (
        include_str!("../sgf/results/alphago_zero_vs_alphago_lee_game4_W+0.5.sgf"),
        EndGame::WinnerByScore(Color::White, 0.5),
    ),
    (
        include_str!("../sgf/results/world_ai_open_2018_final2_golaxy_vs_aq_W+0.5.sgf"),
        EndGame::WinnerByScore(Color::White, 0.5),
    ),
    (
        include_str!("../sgf/results/fox_2018-05-05_clover77_vs_jinmao_W+1.5.sgf"),
        EndGame::WinnerByScore(Color::White, 1.5),
    ),
    (
        include_str!("../sgf/results/world_ai_open_2017_r9-2_abacus_vs_tianrang_B+1.5.sgf"),
        EndGame::WinnerByScore(Color::Black, 1.5),
    ),
];

/// Area score (stones + territory, komi for White) with `dead` removed.
fn area_score(game: &Game, dead: &HashSet<usize>) -> (f32, f32) {
    let mut goban = *game.goban();
    for &chain in dead {
        goban.remove_chain(chain);
    }
    let (black_territory, white_territory) = goban.calculate_territories();
    let (black_stones, white_stones) = goban.number_of_stones();
    (
        black_stones as f32 + black_territory as f32,
        white_stones as f32 + white_territory as f32 + game.komi(),
    )
}

fn winner_by_score((black, white): (f32, f32)) -> EndGame {
    if black > white {
        EndGame::WinnerByScore(Color::Black, black - white)
    } else {
        EndGame::WinnerByScore(Color::White, white - black)
    }
}

#[test]
fn recorded_result_is_read_from_the_sgf() {
    for (sgf, expected) in ALL_STONES.iter().chain(WITH_DEAD_STONES.iter()) {
        let game = Game::from_sgf(sgf).unwrap();
        assert_eq!(game.size(), (19, 19));
        assert_eq!(game.komi(), 7.5);
        assert_eq!(game.outcome(), Some(*expected));
    }
}

#[test]
fn area_score_matches_recorded_result_with_all_stones() {
    for (sgf, expected) in ALL_STONES {
        let game = Game::from_sgf(sgf).unwrap();
        let score = game.calculate_score_by(ScoreRules::STONES | ScoreRules::KOMI);
        assert_eq!(winner_by_score(score), expected);
        assert_eq!(score, area_score(&game, &HashSet::new()));
    }
}

#[test]
#[cfg(feature = "deadstones")]
fn area_score_matches_recorded_result_after_removing_dead_stones() {
    for (sgf, expected) in WITH_DEAD_STONES {
        let game = Game::from_sgf(sgf).unwrap();
        // With the dead stones still on the board the count is wrong.
        assert_ne!(
            winner_by_score(area_score(&game, &HashSet::new())),
            expected
        );
        let dead = game.dead_stones();
        assert!(!dead.is_empty());
        assert_eq!(winner_by_score(area_score(&game, &dead)), expected);
    }
}
