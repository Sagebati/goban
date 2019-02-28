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

    #[test]
    fn goban() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push(&(1, 2), Color::White).expect("Put the stone in the goban");
        println!("{}", g.pretty_string());
        assert_eq!(true, true)
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
}