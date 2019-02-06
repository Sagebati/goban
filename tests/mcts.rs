use mcts::*;
use mcts::tree_policy::TreePolicy;
use mcts::tree_policy::UCTPolicy;
use goban::rules::game::Game;
use goban::rules::JapRule;
use goban::rules::game::EndGame;
use goban::rules::game::GobanSizes;
use goban::rules::game::Move;
use goban::rules::game::Player;

#[derive(Clone)]
struct IGame(pub Game);

impl GameState for IGame {
    type Move = Move;
    type Player = Player;
    type MoveList = Vec<Move>;

    fn current_player(&self) -> Self::Player {
        self.0.turn()
    }

    fn available_moves(&self) -> Self::MoveList {
        if self.0.over::<JapRule>() {
            Vec::new()
        } else {
            let mut r = self.0.legals::<JapRule>()
                .map(|c| Move::Play(c.0, c.1))
                .collect::<Vec<Move>>();
            //r.push(Move::Pass);
            r.push(Move::Resign);
            r
        }
    }

    fn make_move(&mut self, mov: &Self::Move) {
        self.0.play(mov);
    }
}

struct GoEval;

impl Evaluator<GoMCTS> for GoEval {
    type StateEvaluation = f32;

    fn evaluate_new_state<'a, 'b, 'c>(&'a self, state: &IGame, moves: &Vec<Move>, _handle:
    Option<SearchHandle<'a, GoMCTS>>) -> (Vec<<<GoMCTS as MCTS>::TreePolicy as TreePolicy<GoMCTS>>::MoveEvaluation>, Self::StateEvaluation) {
        (vec![(); moves.len()],
         if let Some(x) = state.0.outcome::<JapRule>() {
             match x {
                 EndGame::Score(black, white) =>
                     if black > white {
                         100.
                     } else {
                         -100.
                     },
                 EndGame::WinnerByResign(player) => match player {
                     Player::Black => 100.,
                     Player::White => -100.,
                 },
             }
         } else {
             let score = state.0.calculate_score::<JapRule>();
             match state.0.turn() {
                 Player::Black => score.0,
                 Player::White => score.1,
             }
         })
    }

    fn evaluate_existing_state<'a, 'b, 'c>(&'a self, _state: &IGame, existing_evaln: &'c
    Self::StateEvaluation, _handle: SearchHandle<'a, GoMCTS>) -> Self::StateEvaluation {
        *existing_evaln
    }

    fn interpret_evaluation_for_player(&self, evaluation: &Self::StateEvaluation, player:
    &<<GoMCTS as MCTS>::State as GameState>::Player) -> i64 {
        match player {
            Player::White =>
                (-*evaluation * 10.) as i64,
            Player::Black =>
                (*evaluation * 10.) as i64
        }
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
    let mut game: IGame = IGame(Game::new(GobanSizes::Custom(5)));
    for _i in 0..150 {
        let mut mcts = MCTSManager::new(game.clone(), GoMCTS, GoEval, UCTPolicy::new(0.6));
        mcts.playout_n_parallel(1000, 12);
        mcts.tree().debug_moves();
        let mov = mcts.best_move().unwrap();
        game.0.display();
        game.0.play(&mov);
    }
}