use std::collections::{HashMap, HashSet};

use ahash::AHashSet;
use oxymcts::{
    DefaultBackProp, DefaultLazyTreePolicy, Evaluator, GameTrait, LazyMcts, LazyMctsNode,
    Num, Playout, uct_value,
};
use rand::prelude::{SliceRandom, ThreadRng};
use rand::thread_rng;

use crate::pieces::goban::Goban;
use crate::pieces::GoStringPtr;
use crate::pieces::stones::{Color, Stone};
use crate::rules::{IllegalRules, Move, Player};
use crate::rules::game::Game;

impl GameTrait for Game {
    type Player = Player;
    type Move = Move;

    fn legals_moves(&self) -> Vec<Self::Move> {
        let moves = self
            .legals_by(self.rule.illegal_flag() | IllegalRules::FILLEYE)
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
            .expect("Exquo in Go is very rare")
    }
}

struct Eval;

#[derive(Clone)]
struct EvalR {
    reward: u32,
    last_board: Goban,
}

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
                    coordinates,
                    color: state.turn().stone_color(),
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
    fn get_floating_stones(&self) -> Vec<GoStringPtr> {
        let eyes = self.pseudo_legals().filter(|&p| {
            self.check_eye(Stone {
                coordinates: p,
                color: Color::Black,
            }) || self.check_eye(Stone {
                coordinates: p,
                color: Color::White,
            })
        });
        let mut strings_wth_eye = HashMap::new();
        for eye in eyes {
            let string_connected_eye = self
                .goban
                .get_neighbors_strings(eye)
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
            .go_strings()
            .iter()
            .cloned()
            .filter_map(|x| x)
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

    pub fn dead_stones_wth_simulations(&self, nb_simulations: usize) ->
    AHashSet<GoStringPtr> {
        let mut game = self.branch();
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
        let final_state_raw = game.goban().raw();
        let mut dead_ren = AHashSet::new();
        for chain in floating_stones {
            for &stone in chain.stones() {
                if final_state_raw[stone] != chain.color {
                    dead_ren.insert(chain);
                    break;
                }
            }
        }
        dead_ren
    }

    /// Return an array of dead stones, works better if the game if ended.
    /// the "dead" stones are only potentially dead.
    pub fn dead_stones(&self) -> AHashSet<GoStringPtr> {
        self.dead_stones_wth_simulations(600)
    }
}
