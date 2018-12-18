mod pieces;
mod rules;

#[cfg(test)]
mod tests {
    use crate::pieces::goban::Goban;
    use crate::rules::game::SizeGoban;
    use crate::rules::game::Game;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn goban() {
        let mut g = Goban::new(SizeGoban::Nineteen as usize);
        g.play(&(1, 2), true);
        println!("{}", g.pretty_string());
        assert_eq!(true, true)
    }

    #[test]
    fn full_game() {
        let g = Game::new(SizeGoban::Nine);
        g
    }
}
