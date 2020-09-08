#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::seq::SliceRandom;

    use goban::pieces::goban::Goban;
    use goban::pieces::stones::Color;
    use goban::pieces::stones::Color::Black;
    use goban::pieces::stones::Stone;
    use goban::pieces::zobrist::ZOBRIST;
    use goban::rules::{EndGame, GobanSizes, Move, Player};
    use goban::rules::game::Game;
    use goban::rules::Move::Play;
    use goban::rules::Rule;

    #[test]
    fn goban() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push((1, 2), Color::White);
        println!("{}", g.pretty_string());
        assert!(true)
    }

    #[test]
    fn goban_new_array() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push((1, 2), Color::White);
        g.push((1, 3), Color::Black);
        let tab = g.raw();
        let g2 = Goban::from_array(&tab);
        assert_eq!(g, g2)
    }

    #[test]
    fn passes() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        g.play(Move::Play(3, 3));
        g.play(Move::Pass);
        g.play(Move::Play(4, 3));
        let goban: &Goban = g.goban();
        assert_eq!(goban.get_stone((4, 3)), Color::Black);
    }

    #[test]
    fn get_all_stones() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push((1, 2), Color::White);
        g.push((0, 0), Color::Black);

        let expected = vec![
            Stone {
                coordinates: (0, 0),
                color: Color::Black,
            },
            Stone {
                coordinates: (1, 2),
                color: Color::White,
            },
        ];
        let vec: Vec<Stone> = g.get_stones().collect();
        assert_eq!(expected, vec)
    }

    #[test]
    fn some_plays() {
        let mut g = Game::new(GobanSizes::Nineteen, Rule::Chinese);
        let mut i = 300;
        while !g.is_over() && i != 0 {
            g.play(
                *g.legals()
                    .map(|coord| Move::Play(coord.0, coord.1))
                    .collect::<Vec<Move>>()
                    .choose(&mut rand::thread_rng())
                    .unwrap(),
            );
            i -= 1;
            println!("{}", g.goban().pretty_string());
        }
    }

    #[test]
    fn test_eye() {
        let g = Game::from_sgf(include_str!("../sgf/ShusakuvsInseki.sgf")).unwrap();
        assert!(g.check_eye(Stone {
            coordinates: (0, 14),
            color: Color::White,
        }));
        assert!(g.check_eye(Stone {
            coordinates: (0, 12),
            color: Color::White,
        }));
        assert!(g.check_eye(Stone {
            coordinates: (1, 11),
            color: Color::White,
        }));
        assert!(!g.check_eye(Stone {
            coordinates: (1, 8),
            color: Color::Black,
        }));
        assert!(!g.check_eye(Stone {
            coordinates: (17, 18),
            color: Color::Black,
        }));
    }

    #[test]
    fn some_plays_from_sgf() {
        let moves_sgf = vec![
            Move::Play(16, 13),
            Move::Play(16, 11),
            Move::Play(14, 12),
            Move::Play(14, 11),
            Move::Play(13, 11),
            Move::Play(13, 12),
            Move::Play(14, 13),
            Move::Play(14, 10),
            Move::Play(12, 12),
            Move::Play(13, 13),
            Move::Play(14, 14),
            Move::Play(12, 13),
            Move::Play(14, 15),
            Move::Play(11, 12),
            Move::Play(12, 11),
            Move::Play(11, 13),
            Move::Play(12, 9),
            Move::Play(13, 8),
            Move::Play(13, 9),
            Move::Play(14, 9),
            Move::Play(14, 8),
            Move::Play(12, 8),
            Move::Play(11, 11),
            Move::Play(15, 8),
            Move::Play(14, 7),
            Move::Play(13, 6),
            Move::Play(15, 7),
            Move::Play(16, 8),
            Move::Play(16, 7),
            Move::Play(17, 7),
            Move::Play(17, 6),
            Move::Play(17, 8),
            Move::Play(14, 5),
            Move::Play(16, 4),
            Move::Play(12, 6),
            Move::Play(11, 8),
            Move::Play(13, 5),
            Move::Play(10, 11),
            Move::Play(10, 10),
            Move::Play(11, 10),
            Move::Play(13, 2),
            Move::Play(11, 9),
            Move::Play(16, 2),
            Move::Play(15, 2),
            Move::Play(15, 1),
            Move::Play(14, 1),
            Move::Play(16, 1),
            Move::Play(13, 3),
            Move::Play(12, 3),
            Move::Play(12, 2),
            Move::Play(13, 1),
            Move::Play(14, 2),
            Move::Play(13, 4),
            Move::Play(12, 1),
            Move::Play(13, 0),
            Move::Play(14, 0),
            Move::Play(14, 3),
            Move::Play(17, 4),
            Move::Play(17, 5),
            Move::Play(18, 1),
            Move::Play(2, 13),
            Move::Play(2, 11),
            Move::Play(4, 12),
            Move::Play(3, 13),
            Move::Play(3, 12),
            Move::Play(2, 12),
            Move::Play(3, 14),
            Move::Play(4, 13),
            Move::Play(4, 14),
            Move::Play(5, 13),
            Move::Play(2, 14),
            Move::Play(5, 14),
            Move::Play(4, 15),
            Move::Play(2, 6),
            Move::Play(5, 15),
            Move::Play(6, 15),
            Move::Play(6, 16),
            Move::Play(7, 15),
            Move::Play(7, 16),
            Move::Play(8, 16),
            Move::Play(6, 14),
            Move::Play(5, 12),
            Move::Play(3, 10),
            Move::Play(2, 10),
            Move::Play(4, 10),
            Move::Play(6, 10),
            Move::Play(5, 11),
            Move::Play(7, 13),
            Move::Play(6, 11),
            Move::Play(7, 12),
            Move::Play(7, 11),
            Move::Play(3, 8),
            Move::Play(7, 14),
            Move::Play(8, 15),
            Move::Play(8, 14),
            Move::Play(9, 13),
            Move::Play(9, 12),
            Move::Play(8, 13),
            Move::Play(10, 14),
            Move::Play(9, 14),
            Move::Play(10, 12),
            Move::Play(9, 11),
            Move::Play(8, 11),
            Move::Play(9, 10),
            Move::Play(10, 13),
            Move::Play(12, 15),
            Move::Play(12, 14),
            Move::Play(13, 14),
            Move::Play(13, 15),
            Move::Play(11, 14),
            Move::Play(11, 15),
            Move::Play(8, 12),
            Move::Play(12, 14),
            Move::Play(8, 9),
            Move::Play(4, 8),
            Move::Play(3, 9),
            Move::Play(4, 9),
            Move::Play(6, 8),
            Move::Play(4, 6),
            Move::Play(3, 7),
            Move::Play(4, 7),
            Move::Play(6, 6),
            Move::Play(2, 5),
            Move::Play(3, 5),
            Move::Play(3, 6),
            Move::Play(1, 7),
            Move::Play(7, 9),
            Move::Play(7, 8),
            Move::Play(1, 6),
            Move::Play(2, 7),
            Move::Play(5, 5),
            Move::Play(2, 4),
            Move::Play(6, 5),
            Move::Play(7, 5),
            Move::Play(6, 9),
            Move::Play(5, 9),
            Move::Play(7, 10),
            Move::Play(5, 8),
            Move::Play(7, 4),
            Move::Play(8, 5),
            Move::Play(5, 2),
            Move::Play(7, 2),
            Move::Play(7, 3),
            Move::Play(6, 2),
            Move::Play(6, 3),
            Move::Play(5, 1),
            Move::Play(4, 2),
            Move::Play(4, 1),
            Move::Play(3, 2),
            Move::Play(2, 2),
            Move::Play(3, 1),
            Move::Play(2, 1),
            Move::Play(8, 2),
            Move::Play(8, 1),
            Move::Play(9, 4),
            Move::Play(9, 2),
            Move::Play(9, 9),
            Move::Play(10, 9),
            Move::Play(9, 8),
            Move::Play(8, 8),
            Move::Play(8, 10),
            Move::Play(10, 10),
            Move::Play(10, 7),
            Move::Play(11, 6),
            Move::Play(10, 6),
            Move::Play(12, 5),
            Move::Play(11, 5),
            Move::Play(12, 7),
            Move::Play(11, 4),
            Move::Play(8, 4),
            Move::Play(15, 4),
            Move::Play(15, 5),
            Move::Play(14, 6),
            Move::Play(14, 4),
            Move::Play(13, 3),
            Move::Play(16, 6),
            Move::Play(15, 4),
            Move::Play(16, 3),
            Move::Play(16, 5),
            Move::Play(14, 4),
            Move::Play(15, 6),
            Move::Play(15, 4),
            Move::Play(17, 2),
            Move::Play(18, 4),
            Move::Play(18, 5),
            Move::Play(18, 2),
            Move::Play(18, 3),
            Move::Play(17, 3),
            Move::Play(15, 0),
            Move::Play(16, 16),
            Move::Play(17, 15),
            Move::Play(17, 16),
            Move::Play(16, 15),
            Move::Play(14, 17),
            Move::Play(13, 17),
            Move::Play(14, 16),
            Move::Play(15, 16),
            Move::Play(15, 17),
            Move::Play(17, 17),
            Move::Play(18, 17),
            Move::Play(17, 18),
            Move::Play(12, 17),
            Move::Play(13, 16),
            Move::Play(13, 18),
            Move::Play(11, 17),
            Move::Play(10, 17),
            Move::Play(11, 18),
            Move::Play(10, 16),
            Move::Play(11, 16),
            Move::Play(8, 17),
            Move::Play(5, 10),
            Move::Play(3, 11),
            Move::Play(4, 0),
            Move::Play(6, 1),
            Move::Play(17, 12),
            Move::Play(17, 11),
            Move::Play(4, 5),
            Move::Play(3, 4),
            Move::Play(8, 7),
            Move::Play(7, 7),
            Move::Play(7, 18),
            Move::Play(7, 17),
            Move::Play(6, 17),
            Move::Play(4, 17),
            Move::Play(5, 17),
            Move::Play(4, 16),
            Move::Play(5, 16),
            Move::Play(2, 16),
            Move::Play(1, 16),
            Move::Play(1, 17),
            Move::Play(1, 15),
            Move::Play(0, 17),
            Move::Play(2, 15),
            Move::Play(3, 16),
            Move::Play(3, 18),
            Move::Play(2, 17),
            Move::Play(4, 18),
            Move::Play(1, 13),
            Move::Play(1, 18),
            Move::Play(2, 18),
            Move::Play(5, 18),
            Move::Play(1, 14),
            Move::Play(0, 16),
            Move::Play(8, 18),
            Move::Play(3, 17),
            Move::Play(6, 18),
            Move::Play(0, 18),
            Move::Play(8, 3),
            Move::Play(4, 3),
            Move::Play(4, 4),
            Move::Play(5, 4),
            Move::Play(2, 0),
            Move::Play(3, 0),
            Move::Play(5, 0),
            Move::Play(5, 3),
            Move::Play(4, 11),
            Move::Play(8, 6),
            Move::Play(7, 6),
            Move::Play(9, 5),
            Move::Play(11, 2),
            Move::Play(12, 0),
            Move::Play(11, 0),
            Move::Play(10, 1),
            Move::Play(10, 2),
            Move::Play(11, 1),
            Move::Play(9, 1),
            Move::Play(10, 0),
            Move::Play(9, 0),
            Move::Play(11, 0),
            Move::Play(16, 12),
            Move::Play(17, 13),
            Move::Play(15, 12),
            Move::Play(18, 11),
            Move::Play(18, 10),
            Move::Play(18, 12),
            Move::Play(17, 10),
            Move::Play(0, 14),
            Move::Play(1, 12),
            Move::Play(10, 18),
            Move::Play(9, 18),
            Move::Play(18, 7),
            Move::Play(18, 8),
            Move::Play(15, 13),
            Move::Play(13, 10),
            Move::Play(12, 10),
            Move::Play(5, 7),
            Move::Play(11, 3),
            Move::Play(10, 3),
            Move::Play(10, 4),
            Move::Play(0, 13),
            Move::Play(6, 12),
            Move::Play(6, 13),
            Move::Play(18, 3),
            Move::Play(7, 18),
            Move::Play(18, 6),
            Move::Play(0, 15),
            Move::Play(1, 5),
            Move::Play(1, 4),
            Move::Play(0, 14),
            Move::Play(5, 6),
            Move::Play(0, 15),
            Move::Play(2, 3),
            Move::Pass,
            Move::Play(15, 14),
            Move::Pass,
            Move::Play(16, 14),
            Move::Play(17, 14),
            Move::Play(15, 16),
            Move::Play(18, 15),
            Move::Play(18, 16),
            Move::Play(14, 18),
            Move::Play(15, 18),
            Move::Play(12, 18),
            Move::Play(14, 18),
            Move::Play(16, 17),
            Move::Play(10, 15),
            Move::Pass,
            Move::Pass,
        ];
        let handicap = vec![(3, 3), (3, 15), (9, 3), (9, 15), (15, 3), (15, 15)];
        let mut g = Game::new(GobanSizes::Nineteen, Rule::Chinese);
        let inv_coord: Vec<usize> = (0..19).rev().collect();
        g.put_handicap(&handicap);
        for m in moves_sgf {
            let to_play = match m {
                Play(x, y) => {
                    let x = x as usize;
                    let y = y as usize;
                    println!("({},{})", x, y);
                    println!("({},{})", inv_coord[x], y);
                    println!("({},{})", inv_coord[x] + 1, y + 1);
                    if inv_coord[x] == 6 && y == 14 && g.turn() == Player::White {
                        println!("bug")
                    }
                    Play(inv_coord[x] as u8, y as u8)
                }
                m => m,
            };
            g.try_play(to_play).unwrap();
            println!("prisoners: {:?}", g.prisoners());
            g.display_goban()
        }
        assert!(g.is_over());
        let (black_score, white_score) = g.calculate_score();
        let (b_prisoners, w_prisoners) = g.prisoners();
        println!("score  b:{} w:{}", black_score, white_score);
        assert_eq!(w_prisoners, 35);
        assert_eq!(b_prisoners, 16);
        assert_eq!(w_prisoners, 35);
    }

    #[test]
    fn atari() {
        let mut goban = Goban::new((9, 9));
        let s = Stone {
            coordinates: (4, 4),
            color: Color::Black,
        };
        goban.push_stone(s);
        println!("{}", goban.pretty_string());
        let cl = goban.clone();
        let x = cl.get_liberties(s.coordinates);

        x.for_each(|s| {
            println!("{:?}", s.coordinates);
            goban.push_stone(Stone {
                coordinates: s.coordinates,
                color: Color::White,
            });
        });

        println!("{}", goban.pretty_string());

        assert_eq!(goban.get_liberties(s.coordinates).count(), 0);
    }

    #[test]
    fn atari_2() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        g.play(Move::Play(1, 0)); // B
        println!("{}", g.goban().pretty_string());
        g.play(Move::Play(0, 0)); // W
        println!("{}", g.goban().pretty_string());
        g.play(Move::Play(0, 1)); // B
        println!("{}", g.goban().pretty_string());
        // Atari
        assert_eq!(g.goban().get_stone((0, 0)), Color::None);
    }

    #[test]
    fn game_finished() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        g.play(Move::Pass);
        g.play(Move::Pass);

        assert_eq!(g.is_over(), true)
    }

    #[test]
    fn score_calcul() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Japanese);
        g.play(Move::Play(4, 4));
        g.play(Move::Pass);
        g.play(Move::Pass);
        let score = g.calculate_score();
        assert_eq!(score.0, 80.); //Black
        assert_eq!(score.1, Rule::Japanese.komi()); //White
    }

    #[test]
    fn score_calcul2() {
        let mut g = Game::new(GobanSizes::Nineteen, Rule::Chinese);
        g.set_komi(0.);
        (0..38).for_each(|x| {
            g.try_play(Play(if x % 2 == 0 { 9 } else { 8 }, x / 2))
                .unwrap();
        });

        g.display_goban();
        let score = g.calculate_score();
        assert_eq!(score, (10. * 19., 9. * 19.));
        let mut goban: Goban = g.goban().clone();
        goban.push_many(
            &{
                let mut vec = vec![];
                (10..19).for_each(|x| vec.push((x, 3)));
                vec
            },
            Color::Black,
        );
        goban.push_many(
            &vec![
                (11, 6),
                (11, 7),
                (11, 8),
                (12, 6),
                (12, 8),
                (13, 6),
                (13, 7),
                (13, 8),
            ],
            Color::White,
        );

        let terr = goban.calculate_territories();
        assert_eq!(terr, (27, 8 * 19 + 1));

        goban.push_many(
            &vec![(17, 18), (18, 17), (18, 15), (17, 16), (16, 17), (15, 18)],
            Black,
        );

        let terr = goban.calculate_territories();
        println!("{}", goban);
        assert_eq!(terr, (27 + 4, 8 * 19 + 1));
    }

    #[test]
    fn score_calcul_chinese() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        g.play(Move::Play(4, 4));
        g.play(Move::Pass);
        g.play(Move::Pass);
        let outcome = match g.outcome() {
            Some(endgame) => Ok(endgame),
            _ => Err("Game not finished"),
        }
            .expect("Game finished");
        let (black, white) = g.calculate_score();
        assert_eq!(black, 81.);
        assert_eq!(white, g.komi());
        assert_eq!(
            outcome,
            EndGame::WinnerByScore(Player::Black, 81. - g.komi())
        )
    }

    #[test]
    fn zobrist_test() {
        let mut set = HashSet::new();
        for i in 0..19 {
            for j in 0..19 {
                for c in vec![Color::Black, Color::White] {
                    let x = ZOBRIST[((i * 19 + j), c)];
                    assert!(!set.contains(&x));
                    set.insert(x);
                }
            }
        }
    }

    #[test]
    fn ko_test() {
        let mut game: Game = Default::default();
        game.play(Move::Play(0, 3)); // black
        game.display_goban();
        game.play(Move::Play(0, 2)); // white
        game.display_goban();
        game.play(Move::Play(1, 4)); // black
        game.display_goban();
        game.play(Move::Play(2, 2)); // white
        game.display_goban();
        game.play(Move::Play(2, 3)); // black
        game.display_goban();
        game.play(Move::Play(1, 1)); // white
        game.display_goban();
        game.play(Move::Play(1, 2)); // black
        game.display_goban();
        game.play(Move::Play(1, 3)); // white takes
        game.display_goban();
        //game.play(Move::Play(1, 2)); // black takes back
        //println!("{}", game);
        // ko
        assert!(game.check_ko(Stone {
            coordinates: (1, 2),
            color: Color::Black,
        }));
        assert!(!game.legals().any(|m| m == (1, 2)));
        assert!(game.try_play(Move::Play(1, 2)).is_err());
        assert!(game.check_superko(Stone {
            coordinates: (1, 2),
            color: Color::Black,
        }));
    }

    #[test]
    fn suicide_test() {
        let mut game: Game = Default::default();
        game.play(Move::Play(0, 2)); // black
        game.display_goban();
        game.play(Move::Play(0, 0)); // white
        game.display_goban();
        game.play(Move::Play(1, 1)); // black
        game.display_goban();
        game.play(Move::Play(1, 0)); // white
        game.display_goban();
        game.play(Move::Play(2, 0)); // black
        game.display_goban();
        //game.play(Move::Play(0, 1)); // white suicide whith
        //println!("{}", game);
        // suicide
        assert!(game.check_suicide(Stone {
            coordinates: (0, 1),
            color: Color::White,
        }));
        assert!(!game.legals().any(|m| m == (0, 1)));
        assert!(game.try_play(Move::Play(0, 1)).is_err());
    }

    #[test]
    fn sgf_test() {
        let game = Game::from_sgf(include_str!("../sgf/ShusakuvsInseki.sgf")).unwrap();
        println!("score : {:?}", game.calculate_score());
        assert_eq!(
            EndGame::WinnerByScore(Player::Black, 2.0),
            game.outcome().unwrap()
        );
        assert_eq!(game.prisoners(), (31, 29));
    }

    #[test]
    fn sgf_test_2_2ha() {
        let game = Game::from_sgf(include_str!("../sgf/sgf_2_2ha.sgf")).unwrap();
        println!("score : {:?}", game.calculate_score());
        assert_eq!(game.prisoners(), (25, 26));
        assert_eq!(
            EndGame::WinnerByScore(Player::Black, 1.0),
            game.outcome().unwrap()
        );
    }

    #[test]
    fn sgf_test_1() {
        let game = Game::from_sgf(include_str!("../sgf/sgf_1.sgf")).unwrap();
        println!("score : {:?}", game.calculate_score());
        println!("prisoners : {:?}", game.prisoners());
        assert_eq!(game.prisoners(), (2, 9));
        assert_eq!(
            EndGame::WinnerByResign(Player::White),
            game.outcome().unwrap()
        )
    }

    #[test]
    fn dead_stones() {
        let game = Game::from_sgf(include_str!("../sgf/ShusakuvsInseki.sgf")).unwrap();
        game.display_goban();
        let mut goban: Goban = game.goban().clone();
        for string in game.dead_stones_wth_simulations(20) {
            goban.remove_go_string(string);
        }
        println!("{}", goban);
    }
}
