#[cgf(test)]
mod benchs {
    mod mcts {
        use mcts::*;
        use goban::rules::game::Game;
        use goban::rules::game::Move;
        use goban::pieces::util::Coord;
        use mcts::tree_policy::TreePolicy;
        use goban::rules::game::EndGame;

        impl GameState for Game {
            type Move = Move;
            type Player = bool;
            type MoveList = Vec<Move>;

            fn current_player(&self) -> Self::Player {
                self.turn()
            }

            fn available_moves(&self) -> Self::MoveList {
                let mut r = self.legals()
                    .map(|c| Move::Play(c.0, c.1))
                    .collect::<Vec<Move>>();
                r.push(Move::Pass);
                r.push(Move::Resign);
                r
            }

            fn make_move(&mut self, mov: &Self::Move) {}
        }

        struct GoEval;

        impl Evaluator<GoMCTS> for GoEval {
            type StateEvaluation = i32;

            fn evaluate_new_state<'a, 'b, 'c>(&'a self, state: &Game, moves: &Vec<Move>, handle:
            Option<SearchHandle<'a, GoMCTS>>) -> (Vec<<<GoMCTS as MCTS>::TreePolicy as TreePolicy<GoMCTS>>::MoveEvaluation>, Self::StateEvaluation) {
                moves.iter()
                    .map()
                if let Some(x) = state.end_game() {
                    match x {
                        EndGame::Score(black, white) => if black > white { 1 },
                        EndGame::WinnerByResign(b) => if b { 1 } else { -1 },
                    }
                } else {
                    0
                }
            }

            fn evaluate_existing_state<'a, 'b, 'c>(&'a self, state: &Game, existing_evaln: &'c
            Self::StateEvaluation, handle: SearchHandle<'a, GoMCTS>) -> Self::StateEvaluation {
                *existing_evaln
            }

            fn interpret_evaluation_for_player(&self, evaluation: &Self::StateEvaluation, player: &<<GoMCTS as MCTS>::State as GameState>::Player) -> i64 {
                *evaluation as i64
            }
        }


        struct GoMCTS;

        #[bench]
        pub fn playouts_mcts() {}
    }
}