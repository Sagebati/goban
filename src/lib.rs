pub mod pieces;
pub mod rules;

#[cfg(test)]
mod tests {
    use crate::pieces::goban::Goban;
    use crate::rules::game::GobanSizes;
    use crate::rules::game::Game;
    use rand::seq::SliceRandom;
    use crate::pieces::stones::StoneColor;
    use crate::rules::game::Move;

    #[test]
    fn goban() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.play(&(1, 2), true);
        println!("{}", g.pretty_string());
        assert_eq!(true, true)
    }

    #[test]
    fn some_plays() {
        let mut g = Game::new(GobanSizes::Nine);
        let mut i = 35;
        while !g.legals().is_empty() && i != 0 {
            g.play(g.legals().choose(&mut rand::thread_rng()).unwrap());
            i -= 1;
            println!("{}", g.get_goban().pretty_string());
        }
    }

    #[test]
    fn atari() {
        let mut g = Game::new(GobanSizes::Nine);
        g.play(&Move::Play(1, 0)); // B
        println!("{}", g.get_goban().pretty_string());
        g.play(&Move::Play(0, 0)); // W
        println!("{}", g.get_goban().pretty_string());
        g.play(&Move::Play(1, 1)); // B
        println!("{}", g.get_goban().pretty_string());
        g.play(&Move::Play(8, 8)); // W
        println!("{}", g.get_goban().pretty_string());
        g.play(&Move::Play(0, 1)); // B
        println!("{}", g.get_goban().pretty_string());
        // Atari
        assert_eq!(g.get_goban().get(&(0, 0)), StoneColor::Empty);
    }

    #[test]
    fn score_calcul() {
        let mut g = Game::new(GobanSizes::Nine);
        g.play(&Move::Play(4, 4));
        let score = g.calculate_pseudo_score();
        assert_eq!(score.1, 81.); //Black
        assert_eq!(score.0, 5.5); //White
    }
}
