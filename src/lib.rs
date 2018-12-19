mod pieces;
mod rules;

#[cfg(test)]
mod tests {
    use crate::pieces::goban::Goban;
    use crate::rules::game::SizeGoban;
    use crate::rules::game::Game;
    use rand::Rng;
    use crate::pieces::stones::Stones;

    #[test]
    fn goban() {
        let mut g = Goban::new(SizeGoban::Nineteen as usize);
        g.play(&(1, 2), true);
        println!("{}", g.pretty_string());
        assert_eq!(true, true)
    }

    #[test]
    fn full_game() {
        let mut g = Game::new(SizeGoban::Nine);
        while !g.legals().is_empty() {
            g.play(rand::thread_rng().choose(&g.legals()).unwrap());
            println!("{}",g.get_goban().pretty_string());
        }
    }

    #[test]
    fn atari(){
        let mut g= Game::new(SizeGoban::Nine);
        g.play(&(1,0)); // B
        println!("{}",g.get_goban().pretty_string());
        g.play(&(0,0)); // W
        println!("{}",g.get_goban().pretty_string());
        g.play(&(1,1)); // B
        println!("{}",g.get_goban().pretty_string());
        g.play(&(8,8)); // W
        println!("{}",g.get_goban().pretty_string());
        g.play(&(0,1)); // B
        println!("{}",g.get_goban().pretty_string());
        // Atari
        assert_eq!(g.get_goban().get(&(0,0)),Stones::Empty);
    }
}
