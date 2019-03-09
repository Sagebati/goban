#[cfg(test)]
mod tests {
    use goban::rules::game::GobanSizes;
    use goban::pieces::goban::Goban;
    use goban::rules::game::Move;
    use goban::pieces::stones::Color;
    use goban::rules::game::Game;
    use goban::pieces::stones::Stone;
    use rand::seq::IteratorRandom;
    use goban::rules::Rule;
    use goban::rules::EndGame;
    use goban::pieces::util::coord::Order;
    use goban::rules::game::Move::Play;

    #[test]
    fn goban() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push(&(1, 2), Color::White).expect("Put the stone in the goban");
        println!("{}", g.pretty_string());
        assert!(true)
    }

    #[test]
    fn goban_new_array() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push(&(1, 2), Color::White).expect("Put the stone in the goban");
        g.push(&(1, 3), Color::Black).expect("Put the stone in the goabn");
        let tab = g.tab();
        let g2 = Goban::from_array(&tab, Order::RowMajor);
        assert_eq!(g, g2)
    }

    #[test]
    fn passes() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        g.play(&Move::Play(3, 3));
        g.play(&Move::Pass);
        g.play(&Move::Play(4, 3));
        let goban: &Goban = g.goban();
        assert_eq!(goban.get(&(4, 3)), Color::Black);
    }

    #[test]
    fn get_all_stones() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push(&(1, 2), Color::White).expect("Put the stone in the goban");
        g.push(&(0, 0), Color::Black).expect("Put the stone in the goban");

        let expected = vec![
            Stone { coord: (0, 0), color: Color::Black },
            Stone { coord: (1, 2), color: Color::White }
        ];
        let vec: Vec<Stone> = g.get_stones().collect();
        assert_eq!(expected, vec)
    }

    #[test]
    fn some_plays() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        let mut i = 35;
        while !g.legals().count() != 0 && i != 0 {
            g.play(
                &g.legals().map(|coord| Move::Play(coord.0, coord.1))
                    .choose(&mut rand::thread_rng())
                    .unwrap());
            i -= 1;
            println!("{}", g.goban().pretty_string());
        }
    }

    fn vec_bool_to_vec_u8(w_stones: &Vec<bool>, b_stones: &Vec<bool>) -> Vec<u8> {
        let mut res: Vec<u8> = vec![Color::None.into(); w_stones.len()];
        for i in 0..w_stones.len() {
            if w_stones[i] && b_stones[i] {
                panic!("Error");
            }
            if w_stones[i] {
                res[i] = Color::White.into();
            }
            if b_stones[i] {
                res[i] = Color::Black.into();
            }
        }
        res
    }

    #[test]
    fn some_plays_integrity_boolean_vecs() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        let mut i = 40;
        while !g.legals().count() != 0 && i != 0 {
            g.play(
                &g.legals().map(|coord| Move::Play(coord.0, coord.1))
                    .choose(&mut rand::thread_rng())
                    .unwrap());
            i -= 1;
            assert_eq!(g.goban().tab(), &vec_bool_to_vec_u8(g.goban().w_stones(), g.goban()
                .b_stones()));
            println!("{}", g.goban().pretty_string());
        }
    }

    #[test]
    fn some_plays_from_sgf() {
        let moves_sgf = vec![
            Move::Play(16, 13), Move::Play(16, 11), Move::Play(14, 12), Move::Play(14, 11), Move::Play(13, 11), Move::Play(13, 12), Move::Play(14, 13), Move::Play(14, 10), Move::Play(12, 12), Move::Play(13, 13), Move::Play(14, 14),
            Move::Play(12, 13), Move::Play(14, 15), Move::Play(11, 12), Move::Play(12, 11), Move::Play
                (11, 13), Move::Play(12, 9), Move::Play(13, 8), Move::Play(13, 9), Move::Play(14, 9),
            Move::Play(14, 8), Move::Play(12, 8), Move::Play(11, 11), Move::Play(15, 8), Move::Play(14, 7), Move::Play(13, 6), Move::Play(15, 7), Move::Play(16, 8), Move::Play(16, 7), Move::Play(17, 7), Move::Play(17, 6), Move::Play(17, 8), Move::Play(14, 5), Move::Play(16, 4), Move::Play(12, 6), Move::Play(11, 8), Move::Play(13, 5), Move::Play(10, 11), Move::Play(10, 10), Move::Play(11, 10), Move::Play(13, 2), Move::Play(11, 9), Move::Play(16, 2), Move::Play(15, 2), Move::Play(15, 1), Move::Play(14, 1), Move::Play(16, 1), Move::Play(13, 3), Move::Play(12, 3), Move::Play(12, 2), Move::Play(13, 1), Move::Play(14, 2), Move::Play(13, 4), Move::Play(12, 1), Move::Play(13, 0), Move::Play(14, 0), Move::Play(14, 3), Move::Play(17, 4), Move::Play(17, 5), Move::Play(18, 1), Move::Play(2, 13), Move::Play(2, 11), Move::Play(4, 12), Move::Play(3, 13), Move::Play(3, 12), Move::Play(2, 12), Move::Play(3, 14), Move::Play(4, 13), Move::Play(4, 14), Move::Play(5, 13), Move::Play(2, 14), Move::Play(5, 14), Move::Play(4, 15), Move::Play(2, 6), Move::Play(5, 15), Move::Play(6, 15), Move::Play(6, 16), Move::Play(7, 15), Move::Play(7, 16), Move::Play(8, 16), Move::Play(6, 14), Move::Play(5, 12), Move::Play(3, 10), Move::Play(2, 10), Move::Play(4, 10), Move::Play(6, 10), Move::Play(5, 11), Move::Play(7, 13), Move::Play(6, 11), Move::Play(7, 12), Move::Play(7, 11), Move::Play(3, 8), Move::Play(7, 14), Move::Play(8, 15), Move::Play(8, 14), Move::Play(9, 13), Move::Play(9, 12), Move::Play(8, 13), Move::Play(10, 14), Move::Play(9, 14), Move::Play(10, 12), Move::Play(9, 11), Move::Play(8, 11), Move::Play(9, 10), Move::Play(10, 13), Move::Play(12, 15), Move::Play(12, 14), Move::Play(13, 14), Move::Play(13, 15), Move::Play(11, 14), Move::Play(11, 15), Move::Play(8, 12), Move::Play(12, 14), Move::Play(8, 9), Move::Play(4, 8), Move::Play(3, 9), Move::Play(4, 9), Move::Play(6, 8), Move::Play(4, 6), Move::Play(3, 7), Move::Play(4, 7), Move::Play(6, 6), Move::Play(2, 5), Move::Play(3, 5), Move::Play(3, 6), Move::Play(1, 7), Move::Play(7, 9), Move::Play(7, 8), Move::Play(1, 6), Move::Play(2, 7), Move::Play(5, 5), Move::Play(2, 4), Move::Play(6, 5), Move::Play(7, 5), Move::Play(6, 9), Move::Play(5, 9), Move::Play(7, 10), Move::Play(5, 8), Move::Play(7, 4), Move::Play(8, 5), Move::Play(5, 2), Move::Play(7, 2), Move::Play(7, 3), Move::Play(6, 2), Move::Play(6, 3), Move::Play(5, 1), Move::Play(4, 2), Move::Play(4, 1), Move::Play(3, 2), Move::Play(2, 2), Move::Play(3, 1), Move::Play(2, 1), Move::Play(8, 2), Move::Play(8, 1), Move::Play(9, 4), Move::Play(9, 2), Move::Play(9, 9), Move::Play(10, 9), Move::Play(9, 8), Move::Play(8, 8), Move::Play(8, 10), Move::Play(10, 10), Move::Play(10, 7), Move::Play(11, 6), Move::Play(10, 6), Move::Play(12, 5), Move::Play(11, 5), Move::Play(12, 7), Move::Play(11, 4), Move::Play(8, 4), Move::Play(15, 4), Move::Play(15, 5), Move::Play(14, 6), Move::Play(14, 4), Move::Play(13, 3), Move::Play(16, 6), Move::Play(15, 4), Move::Play(16, 3), Move::Play(16, 5), Move::Play(14, 4), Move::Play(15, 6), Move::Play(15, 4), Move::Play(17, 2), Move::Play(18, 4), Move::Play(18, 5), Move::Play(18, 2), Move::Play(18, 3), Move::Play(17, 3), Move::Play(15, 0), Move::Play(16, 16), Move::Play(17, 15), Move::Play(17, 16), Move::Play(16, 15), Move::Play(14, 17), Move::Play(13, 17), Move::Play(14, 16), Move::Play(15, 16), Move::Play(15, 17), Move::Play(17, 17), Move::Play(18, 17), Move::Play(17, 18), Move::Play(12, 17), Move::Play(13, 16), Move::Play(13, 18), Move::Play(11, 17), Move::Play(10, 17), Move::Play(11, 18), Move::Play(10, 16), Move::Play(11, 16), Move::Play(8, 17), Move::Play(5, 10), Move::Play(3, 11), Move::Play(4, 0), Move::Play(6, 1), Move::Play(17, 12), Move::Play(17, 11), Move::Play(4, 5), Move::Play(3, 4), Move::Play(8, 7), Move::Play(7, 7), Move::Play(7, 18), Move::Play(7, 17), Move::Play(6, 17), Move::Play(4, 17), Move::Play(5, 17), Move::Play(4, 16), Move::Play(5, 16), Move::Play(2, 16), Move::Play(1, 16), Move::Play(1, 17), Move::Play(1, 15), Move::Play(0, 17), Move::Play(2, 15), Move::Play(3, 16), Move::Play(3, 18), Move::Play(2, 17), Move::Play(4, 18), Move::Play(1, 13), Move::Play(1, 18), Move::Play(2, 18), Move::Play(5, 18), Move::Play(1, 14), Move::Play(0, 16), Move::Play(8, 18), Move::Play(3, 17), Move::Play(6, 18), Move::Play(0, 18), Move::Play(8, 3), Move::Play(4, 3), Move::Play(4, 4), Move::Play(5, 4), Move::Play(2, 0), Move::Play(3, 0), Move::Play(5, 0), Move::Play(5, 3), Move::Play(4, 11), Move::Play(8, 6), Move::Play(7, 6), Move::Play(9, 5), Move::Play(11, 2), Move::Play(12, 0), Move::Play(11, 0), Move::Play(10, 1), Move::Play(10, 2), Move::Play(11, 1), Move::Play(9, 1), Move::Play(10, 0), Move::Play(9, 0), Move::Play(11, 0), Move::Play(16, 12), Move::Play(17, 13), Move::Play(15, 12), Move::Play(18, 11), Move::Play(18, 10), Move::Play(18, 12), Move::Play(17, 10), Move::Play(0, 14), Move::Play(1, 12), Move::Play(10, 18), Move::Play(9, 18), Move::Play(18, 7), Move::Play(18, 8), Move::Play(15, 13), Move::Play(13, 10), Move::Play(12, 10), Move::Play(5, 7), Move::Play(11, 3), Move::Play(10, 3), Move::Play(10, 4), Move::Play(0, 13), Move::Play(6, 12), Move::Play(6, 13), Move::Play(18, 3), Move::Play(7, 18), Move::Play(18, 6), Move::Play(0, 15), Move::Play(1, 5), Move::Play(1, 4), Move::Play(0, 14), Move::Play(5, 6), Move::Play(0, 15), Move::Play(2, 3), Move::Pass, Move::Play(15, 14), Move::Pass, Move::Play(16, 14), Move::Play(17, 14), Move::Play(15, 16), Move::Play(18, 15), Move::Play(18, 16), Move::Play(14, 18), Move::Play(15, 18), Move::Play(12, 18), Move::Play(14, 18), Move::Play(16, 17), Move::Play(10, 15), Move::Pass, Move::Pass, ];

        let handicap = vec![(3, 3), (3, 15), (9, 3), (9, 15), (15, 3), (15, 15)];
        let mut g = Game::new(GobanSizes::Nineteen, Rule::Japanese);
        let inv_coord: Vec<usize> = (0..19).rev().collect();
        g.put_handicap(&handicap);
        let mut c = 0;
        for m in moves_sgf {
            let to_play = match m {
                Play(x, y) => {
                    println!("({},{})", x, y);
                    println!("({},{})", inv_coord[x], y);
                    Play(inv_coord[x], y)
                }
                Move::Pass => Move::Pass,
                _ => unreachable!()
            };
            if c == 292 {
                println!();
                let mut  gg: Goban = g.goban().clone();
                gg.push(&(0, 3), Color::White);
                println!("{}", gg);
            }
            println!("count : {}", c - 6);
            g.play_with_verifications(&to_play).unwrap();
            g.display();
            c += 1;
        }
    }

    #[test]
    fn atari() {
        let mut goban = Goban::new(9);
        let s = Stone { coord: (4, 4), color: Color::Black };
        goban.push_stone(&s).expect("Put the stone");
        println!("{}", goban.pretty_string());
        let cl = goban.clone();
        let x = cl.get_liberties(&s);

        x.for_each(|s| {
            println!("{:?}", s.coord);
            goban.push_stone(&Stone { coord: s.coord, color: Color::White })
                .expect("Put the stone");
        });

        println!("{}", goban.pretty_string());

        assert_eq!(goban.get_liberties(&s).count(), 0);
    }

    #[test]
    fn atari_2() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        g.play(&Move::Play(1, 0)); // B
        println!("{}", g.goban().pretty_string());
        g.play(&Move::Play(0, 0)); // W
        println!("{}", g.goban().pretty_string());
        g.play(&Move::Play(0, 1)); // B
        println!("{}", g.goban().pretty_string());
        // Atari
        assert_eq!(g.goban().get(&(0, 0)), Color::None);
    }

    #[test]
    fn game_finished() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        g.play(&Move::Pass);
        g.play(&Move::Pass);

        assert_eq!(g.over(), true)
    }

    #[test]
    fn score_calcul() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Japanese);
        g.play(&Move::Play(4, 4));
        g.play(&Move::Pass);
        g.play(&Move::Pass);
        let score = match g.outcome() {
            Some(EndGame::Score(black, white)) => Ok((black, white)),
            _ => Err("Game not finished"),
        }.expect("Game finished");
        assert_eq!(score.0, 80.); //Black
        assert_eq!(score.1, 5.5); //White
    }

    #[test]
    fn score_calcul_chinesse() {
        let mut g = Game::new(GobanSizes::Nine, Rule::Chinese);
        g.play(&Move::Play(4, 4));
        g.play(&Move::Pass);
        g.play(&Move::Pass);
        let score = match g.outcome() {
            Some(EndGame::Score(black, white)) => Ok((black, white)),
            _ => Err("Game not finished"),
        }.expect("Game finished");
        assert_eq!(score.0, 81.); //Black
        assert_eq!(score.1, 5.5); //White
    }
}