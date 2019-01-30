#[cfg(test)]
mod tests {
    use goban::rules::game::GobanSizes;
    use goban::pieces::goban::Goban;
    use goban::rules::game::Move;
    use goban::pieces::stones::Color;
    use goban::rules::game::Game;
    use goban::rules::JapRule;
    use goban::rules::game::EndGame;
    use goban::pieces::stones::Stone;
    use rand::seq::IteratorRandom;

    #[test]
    fn goban() {
        let mut g = Goban::new(GobanSizes::Nineteen.into());
        g.push(&(1, 2), Color::White).expect("Put the stone in the goban");
        println!("{}", g.pretty_string());
        assert_eq!(true, true)
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
        let mut g = Game::new(GobanSizes::Nine);
        let mut i = 35;
        while !g.legals::<JapRule>().count() != 0 && i != 0 {
            g.play(
                &g.legals::<JapRule>().map(|coord| Move::Play(coord.0, coord.1))
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
        let mut g = Game::new(GobanSizes::Nine);
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
        let mut g = Game::new(GobanSizes::Nine);
        g.play(&Move::Pass);
        g.play(&Move::Pass);

        assert_eq!(g.over::<JapRule>(), true)
    }

    #[test]
    fn score_calcul() {
        let mut g = Game::new(GobanSizes::Nine);
        g.play(&Move::Play(4, 4));
        g.play(&Move::Pass);
        g.play(&Move::Pass);
        let score = match g.end_game::<JapRule>() {
            Some(EndGame::Score(black, white)) => Ok((black, white)),
            _ => Err("Game not finished"),
        }.expect("Game finished");
        assert_eq!(score.0, 80.); //Black
        assert_eq!(score.1, 5.5); //White
    }

    use mcts::*;
    use mcts::tree_policy::TreePolicy;
    use mcts::tree_policy::UCTPolicy;

    #[derive(Clone)]
    struct IGame(Game);

    impl GameState for IGame {
        type Move = Move;
        type Player = bool;
        type MoveList = Vec<Move>;

        fn current_player(&self) -> Self::Player {
            *self.0.turn()
        }

        fn available_moves(&self) -> Self::MoveList {
            let mut r = self.0.legals::<JapRule>()
                .map(|c| Move::Play(c.0, c.1))
                .collect::<Vec<Move>>();
            r.push(Move::Pass);
            //r.push(Move::Resign);
            r
        }

        fn make_move(&mut self, mov: &Self::Move) {
            self.0.play(mov);
        }
    }

    struct GoEval;

    impl Evaluator<GoMCTS> for GoEval {
        type StateEvaluation = i32;

        fn evaluate_new_state<'a, 'b, 'c>(&'a self, state: &IGame, moves: &Vec<Move>, handle:
        Option<SearchHandle<'a, GoMCTS>>) -> (Vec<<<GoMCTS as MCTS>::TreePolicy as TreePolicy<GoMCTS>>::MoveEvaluation>, Self::StateEvaluation) {
            (vec![(); moves.len()],
             if let Some(x) = state.0.end_game::<JapRule>() {
                 match x {
                     EndGame::Score(black, white) => if black > white { 1 } else { -1 },
                     EndGame::WinnerByResign(b) => if b { 1 } else { -1 },
                 }
             } else {
                 0
             })
        }

        fn evaluate_existing_state<'a, 'b, 'c>(&'a self, state: &IGame, existing_evaln: &'c
        Self::StateEvaluation, handle: SearchHandle<'a, GoMCTS>) -> Self::StateEvaluation {
            *existing_evaln
        }

        fn interpret_evaluation_for_player(&self, evaluation: &Self::StateEvaluation, player: &<<GoMCTS as MCTS>::State as GameState>::Player) -> i64 {
            *evaluation as i64
        }
    }


    struct GoMCTS;

    impl MCTS for GoMCTS {
        type State = IGame;
        type Eval = GoEval;
        type TreePolicy = UCTPolicy;
        type NodeData = ();
        type ExtraThreadData = ();
    }

    #[test]
    pub fn playouts_mcts() {
        let game: IGame = IGame(Game::new(GobanSizes::Nineteen));
        let mut mcts = MCTSManager::new(game, GoMCTS, GoEval, UCTPolicy::new(0.5));
        mcts.playout_n_parallel(100000, 4);
        mcts.tree().debug_moves();
    }
}