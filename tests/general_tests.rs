#[cfg(test)]
mod tests {
    use goban::rules::game::GobanSizes;
    use goban::pieces::goban::Goban;
    use goban::rules::game::Move;
    use goban::pieces::stones::StoneColor;
    use goban::rules::game::Game;
    use goban::rules::JapRule;
    use goban::rules::game::EndGame;
    use goban::pieces::stones::Stone;
    use rand::seq::IteratorRandom;

    #[test]
    fn goban() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push(&(1, 2), StoneColor::White).expect("Put the stone in the goban");
        println!("{}", g.pretty_string());
        assert_eq!(true, true)
    }

    #[test]
    fn get_all_stones() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push(&(1, 2), StoneColor::White).expect("Put the stone in the goban");
        g.push(&(0, 0), StoneColor::Black).expect("Put the stone in the goban");

        let expected = vec![
            Stone { coord: (0, 0), color: StoneColor::Black },
            Stone { coord: (1, 2), color: StoneColor::White }
        ];
        let vec: Vec<Stone> = g.get_stones().collect();
        assert_eq!(expected, vec)
    }

    #[test]
    fn some_plays() {
        let mut g = Game::new(GobanSizes::Nine);
        let mut i = 35;
        while !g.legals::<JapRule>().count() != 0 && i != 0 {
            g.play(&g.legals::<JapRule>().choose(&mut rand::thread_rng()).unwrap());
            i -= 1;
            println!("{}", g.goban().pretty_string());
        }
    }

    #[test]
    fn atari() {
        let mut g = Game::new(GobanSizes::Nine);
        g.play(&Move::Play(1, 0)); // B
        println!("{}", g.goban().pretty_string());
        g.play(&Move::Play(0, 0)); // W
        println!("{}", g.goban().pretty_string());
        g.play(&Move::Play(1, 1)); // B
        println!("{}", g.goban().pretty_string());
        g.play(&Move::Play(8, 8)); // W
        println!("{}", g.goban().pretty_string());
        g.play(&Move::Play(0, 1)); // B
        println!("{}", g.goban().pretty_string());
        // Atari
        assert_eq!(g.goban().get(&(0, 0)), StoneColor::Empty);
    }

    #[test]
    fn game_finished() {
        let mut g = Game::new(GobanSizes::Nine);
        g.play(&Move::Pass);
        g.play(&Move::Pass);

        assert_eq!(g.game_over(), true)
    }

    #[test]
    fn score_calcul() {
        let mut g = Game::new(GobanSizes::Nine);
        g.play(&Move::Play(4, 4));
        g.play(&Move::Pass);
        g.play(&Move::Pass);
        let score = match g.end_game::<JapRule>() {
            EndGame::Score(black, white) => Ok((black, white)),
            EndGame::GameNotFinish => Err("Game not finished"),
        }.expect("Game finished");
        assert_eq!(score.0, 80.); //Black
        assert_eq!(score.1, 5.5); //White
    }
}