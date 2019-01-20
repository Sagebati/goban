#[cfg(test)]
mod tests {
    use goban::rules::game::GobanSizes;
    use goban::pieces::goban::Goban;
    use goban::rules::game::Move;
    use goban::pieces::stones::StoneColor;
    use goban::rules::game::Game;
    use rand::seq::SliceRandom;
    use goban::rules::JapRule;
    use goban::rules::game::EndGame;

    #[test]
    fn goban() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push(&(1, 2), StoneColor::White).expect("Put the stone in the goban");
        println!("{}", g.pretty_string());
        assert_eq!(true, true)
    }

    #[test]
    fn some_plays() {
        let mut g = Game::new(GobanSizes::Nine);
        let mut i = 35;
        while !g.legals().is_empty() && i != 0 {
            g.play::<JapRule>(g.legals().choose(&mut rand::thread_rng()).unwrap());
            i -= 1;
            println!("{}", g.goban().pretty_string());
        }
    }

    #[test]
    fn atari() {
        let mut g = Game::new(GobanSizes::Nine);
        g.play::<JapRule>(&Move::Play(1, 0)); // B
        println!("{}", g.goban().pretty_string());
        g.play::<JapRule>(&Move::Play(0, 0)); // W
        println!("{}", g.goban().pretty_string());
        g.play::<JapRule>(&Move::Play(1, 1)); // B
        println!("{}", g.goban().pretty_string());
        g.play::<JapRule>(&Move::Play(8, 8)); // W
        println!("{}", g.goban().pretty_string());
        g.play::<JapRule>(&Move::Play(0, 1)); // B
        println!("{}", g.goban().pretty_string());
        // Atari
        assert_eq!(g.goban().get(&(0, 0)), StoneColor::Empty);
    }

    #[test]
    fn game_finished() {
        let mut g = Game::new(GobanSizes::Nine);
        g.play::<JapRule>(&Move::Pass);
        g.play::<JapRule>(&Move::Pass);

        assert_eq!(g.gameover(),true)
    }

    #[test]
    fn score_calcul() {
        let mut g = Game::new(GobanSizes::Nine);
        g.play::<JapRule>(&Move::Play(4, 4));
        g.play::<JapRule>(&Move::Pass);
        g.play::<JapRule>(&Move::Pass);
        let score = match g.end_game::<JapRule>() {
            EndGame::Score(black, white) => Ok((black, white)),
            EndGame::GameNotFinish => Err("Game not finished"),
        }.expect("Game finished");
        assert_eq!(score.0, 80.); //Black
        assert_eq!(score.1, 5.5); //White
    }
}