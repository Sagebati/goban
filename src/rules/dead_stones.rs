use std::collections::{HashMap, HashSet};

use oxymcts::{
    DefaultBackProp, DefaultLazyTreePolicy, Evaluator, GameTrait, LazyMcts, LazyMctsNode,
    Num, Playout, uct_value,
};
use rand::prelude::{SliceRandom, ThreadRng};
use rand::thread_rng;

use crate::pieces::goban::ChainIdx;
use crate::pieces::stones::{Color, Stone};
use crate::pieces::util::coord::two_to_1dim;
use crate::rules::{IllegalRules, Move};
use crate::rules::game::Game;

impl GameTrait for Game {
    type Player = Color;
    type Move = Move;

    fn legals_moves(&self) -> Vec<Self::Move> {
        let moves = self
            .legals_by(self.rule.flag_illegal | IllegalRules::FILLEYE)
            .map(Move::from)
            .collect::<Vec<_>>();
        if moves.is_empty() {
            if self.is_over() {
                vec![]
            } else {
                vec![Move::Pass]
            }
        } else {
            moves
        }
    }

    fn player_turn(&self) -> Self::Player {
        self.turn
    }

    fn hash(&self) -> u64 {
        0
    }

    fn is_final(&self) -> bool {
        self.is_over()
    }

    fn do_move(&mut self, m: &Self::Move) {
        self.play(*m);
    }

    fn get_winner(&self) -> Self::Player {
        self.outcome()
            .unwrap()
            .get_winner()
            .expect("The game was ex aequo")
    }
}

struct Eval;
type Reward = u64;

impl Evaluator<Game, Reward, ()> for Eval {
    type Args = f64;
    type EvalResult = Reward;

    fn eval_child(
        child: &LazyMctsNode<Game, Reward, ()>,
        _turn: &<Game as GameTrait>::Player,
        parent_visits: u32,
        args: &Self::Args,
    ) -> Num {
        uct_value(
            parent_visits,
            child.sum_rewards as f64,
            child.n_visits,
            *args,
        )
    }

    fn evaluate_leaf(child: Game, turn: &<Game as GameTrait>::Player) -> Self::EvalResult {
        let winner = child.get_winner();
        if winner == *turn {
            1
        } else {
            0
        }
    }
}

struct PL;

impl Playout<Game> for PL {
    type Args = ();

    fn playout(mut state: Game, _args: ()) -> Game {
        fn fast_play_random(state: &Game, thread_rng: &mut ThreadRng) -> Move {
            let mut v: Vec<_> = state.pseudo_legals().collect();
            v.shuffle(thread_rng);
            for coordinates in v
                .into_iter()
                .filter(|&point| state.check_point(point).is_none())
            {
                if !state.check_eye(Stone {
                    coord: coordinates,
                    color: state.turn,
                }) {
                    return coordinates.into();
                }
            }
            Move::Pass
        }
        let mut thread_rng = thread_rng();
        while !state.is_over() {
            state.play(fast_play_random(&state, &mut thread_rng));
        }
        state
    }
}

type Mcts<'a> = LazyMcts<
    'a,
    Game,
    DefaultLazyTreePolicy<Game, Eval, (), u64>,
    PL,
    DefaultBackProp,
    Eval,
    (),
    u64,
>;

impl Game {
    fn get_floating_stones(&self) -> Vec<ChainIdx> {
        let eyes = self.pseudo_legals().filter(|&p| {
            self.check_eye(Stone {
                coord: p,
                color: Color::Black,
            }) || self.check_eye(Stone {
                coord: p,
                color: Color::White,
            })
        });
        let mut strings_wth_eye = HashMap::new();
        for eye in eyes {
            let string_connected_eye = self
                .goban
                .get_neighbors_chain_indexes(eye)
                .collect::<HashSet<_>>();
            for x in string_connected_eye {
                strings_wth_eye
                    .entry(x)
                    .and_modify(|v| *v += 1)
                    .or_insert(0);
            }
        }
        let all_strings = self
            .goban
            .chains()
            .iter()
            .enumerate()
            .map(|x| x.0)
            .collect::<HashSet<_>>();
        let string_with_2eyes = strings_wth_eye
            .into_iter()
            .filter(|(_, v)| *v >= 2)
            .map(|x| x.0)
            .collect::<HashSet<_>>();

        all_strings
            .difference(&string_with_2eyes)
            .cloned()
            .collect()
    }

    pub fn dead_stones_wth_simulations(&self, nb_simulations: usize) -> HashSet<ChainIdx> {
        let mut game = self.clone();
        let floating_stones = self.get_floating_stones();
        while !game.is_over() {
            let m = {
                let mut mcts = Mcts::with_capacity(&game, nb_simulations);
                for _ in 0..nb_simulations {
                    mcts.execute(&2.0_f64.sqrt(), ());
                }
                mcts.best_move(&2.0_f64.sqrt())
            };
            game.play(m);
        }
        let final_state_raw = game.goban().to_vec();
        let mut dead_ren = HashSet::new();
        for chain_ix in floating_stones {
            let chain = self.goban.chain(chain_ix);
            for stone in self.goban.chain_stones(chain_ix) {
                // If some stones of the string arent in the final goban then it's plausible that
                // this string is dead.
                if final_state_raw[two_to_1dim(self.size(), stone.coord)] != Some(chain.color) {
                    dead_ren.insert(chain_ix);
                    break;
                }
            }
        }
        dead_ren
    }

    /// Return an array of dead stones, works better if the game if ended.
    /// the "dead" stones are only potentially dead.
    #[inline]
    pub fn dead_stones(&self) -> HashSet<ChainIdx> {
        self.dead_stones_wth_simulations(600)
    }
}
