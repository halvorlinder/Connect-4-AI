use crate::game_logic::{
    eval, get_legal, play, result, GameGlobals, GameResult, GameState, Move, PaddedGameState,
    Player,
};
use rand::prelude::*;
use rulinalg::utils;
use rulinalg::utils::{argmax, argmin};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::io;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use cpu_time::ProcessTime;
use std::time::Duration;
use rulinalg::vector::Vector;

use std::sync::atomic::{AtomicUsize, Ordering};

// static CALL_COUNT_TO_MIN_MAX: AtomicUsize = AtomicUsize::new(0);

pub struct Game {
    gs: GameState,
    player_1: Box<dyn Agent>,
    player_2: Box<dyn Agent>,
    game_globals: GameGlobals,
}

impl Game {
    fn generate_agent(player: Player) -> Box<dyn Agent> {
        println!("Please select agent type for {:}", player);
        let agent_types: Vec<Agents> = Agents::iter().collect();
        Game::display_agent_options(&agent_types);
        let index = get_int_in_range_from_user(0, agent_types.len());
        let agent = agent_types[index];
        <dyn Agent>::new(agent, 6, 7)
    }

    fn display_agent_options(options: &Vec<Agents>) {
        for (i, option) in options.into_iter().enumerate() {
            println!("{:}: {:?}", i, option)
        }
    }

    pub fn new(rows: usize, cols: usize) -> Self {
        let game_globals = GameGlobals::new(rows, cols);
        Self {
            gs: GameState::new(&game_globals),
            player_1: Game::generate_agent(Player::P1),
            player_2: Game::generate_agent(Player::P2),
            game_globals,
        }
    }

    fn play(&mut self, mov: Move) -> bool {
        return match play(mov, &self.gs) {
            None => false,
            Some(gs) => {
                self.gs = gs;
                true
            }
        };
    }

    fn next(&mut self) -> Option<GameResult> {
        let mov = match self.gs.turn {
            Player::P1 => &self.player_1,
            Player::P2 => &self.player_2,
        }
        .next_move(&self.gs);
        self.play(mov);
        return result(&self.gs);
    }

    fn game_over(&self, res: GameResult) {
        println!("{:}", self.gs);
        println!("The game ended with the following result: {:}", res)
    }

    fn game_loop(&mut self) {
        println!("{:}", self.gs);
        let res = self.next();
        match res {
            None => {
                self.game_loop();
            }
            Some(r) => self.game_over(r),
        }
    }

    pub fn start_game(&mut self) {
        self.game_loop();
    }
}

pub trait Agent {
    fn next_move(&self, gs: &GameState) -> Move;
}

impl dyn Agent {
    pub fn new(agent_type: Agents, rows: usize, cols: usize) -> Box<dyn Agent> {
        let agent: Box<dyn Agent> = (match agent_type {
            Agents::Human => Box::new(Human::new()),
            Agents::RandomMover => Box::new(RandomMover::new()),
            Agents::MinMaxAgent => Box::new(MinMaxAgent::new(rows, cols)),
        });
        agent
    }
}

#[derive(EnumIter, Debug, Eq, PartialEq, Clone, Copy)]
pub enum Agents {
    Human,
    RandomMover,
    MinMaxAgent,
}

pub struct Human {}

impl Human {
    pub fn new() -> Self {
        Self {}
    }
}
impl Agent for Human {
    fn next_move(&self, gs: &GameState) -> Move {
        let moves = get_legal(&gs);
        println!("{:?}", moves);
        println!("{:} to move. Select a move from the list", gs.turn);
        moves[get_int_in_range_from_user(0, moves.len())]
    }
}

fn print_illegal() {
    println!("Illegal input!");
}

fn get_int_in_range_from_user(lower: usize, upper: usize) -> usize {
    loop {
        let mut input_line = String::new();
        let res = io::stdin().read_line(&mut input_line);
        match res {
            Err(_) => {
                print_illegal();
                continue;
            }
            _ => {}
        }
        let index_res: Result<usize, _> = input_line.trim().parse();
        match index_res {
            Ok(i) if i >= lower && i < upper => return i,
            _ => {
                print_illegal();
                continue;
            }
        };
    }
}

fn get_bool_from_user() -> bool {
    loop {
        let mut input_line = String::new();
        let res = io::stdin().read_line(&mut input_line);
        match res {
            Err(_) => {
                print_illegal();
                continue;
            }
            _ => {}
        }
        let bool_res: Result<char, _> = input_line.trim().parse();
        match bool_res {
            Ok(c) if c.to_ascii_uppercase() == 'Y' => return true,
            Ok(c) if c.to_ascii_uppercase() == 'N' => return false,
            _ => {
                print_illegal();
                continue;
            }
        };
    }
}

pub struct RandomMover {}

impl RandomMover {
    pub fn new() -> Self {
        Self {}
    }
}

impl Agent for RandomMover {
    fn next_move(&self, gs: &GameState) -> Move {
        let moves = get_legal(&gs);
        let mut rng = rand::thread_rng();
        return moves[rng.gen_range(0..moves.len())];
    }
}

pub struct MinMaxAgent {
    timed: bool,
    time: i32,
    depth: i32,
    game_globals: GameGlobals,
    visited : HashMap<GameState, f32>,
}

impl MinMaxAgent {
    fn get_time_settings() -> (bool, i32) {
        println!("Should the agent use a timer? (Y/N)");
        let timed = get_bool_from_user();
        println!("Maximum number of seconds for a move [1,600]:");
        let time = match timed {
            true => get_int_in_range_from_user(1, 601),
            false => 0,
        } as i32;
        (timed, time)
    }

    fn get_depth_setting() -> i32 {
        println!("Maximum search depth [1,10]:");
        get_int_in_range_from_user(1, 11) as i32
    }

    pub fn new(rows: usize, cols: usize) -> Self {
        let (timed, time) = MinMaxAgent::get_time_settings();
        let depth = match timed {
            true => 0,
            false => MinMaxAgent::get_depth_setting(),
        };
        Self {
            timed,
            time,
            depth,
            game_globals: GameGlobals::new(rows, cols),
            visited : HashMap::new(),
        }
    }

    pub fn new_with_args(timed: bool, time: i32, depth: i32, rows: usize, cols: usize) -> Self {
        Self {
            timed,
            time,
            depth,
            game_globals: GameGlobals::new(rows, cols),
            visited : HashMap::new(),
        }
    }

    fn min_max(
        &self,
        padded_gs: &PaddedGameState,
        depth: i32,
        mut alpha: f32,
        mut beta: f32,
        visited: &mut HashMap<GameState, f32>,
    ) -> f32 {
        // CALL_COUNT_TO_MIN_MAX.fetch_add(1, Ordering::SeqCst);
        let e = padded_gs.eval;
        let (is_max, selector, base_value): (bool, fn(f32, f32) -> (f32), f32) =
            if padded_gs.gs.turn == Player::P1 {
                (true, f32::max, f32::NEG_INFINITY)
            } else {
                (false, f32::min, f32::INFINITY)
            };

        match visited.entry(padded_gs.gs.to_owned()) {
            Entry::Occupied(duplicate) => {return *duplicate.get();}
            Entry::Vacant(_) => {}
        }
        match e {
            f32::INFINITY => f32::INFINITY,
            f32::NEG_INFINITY => f32::NEG_INFINITY,
            _ => match depth {
                0 => padded_gs.eval,
                depth => {
                    let moves = get_legal(&padded_gs.gs);
                    let num_moves = moves.len();
                    let pruned_moves = if padded_gs.is_symmetrical() {
                        moves.into_iter().take((num_moves+1)/2).collect()} else {moves};
                    let mut states: Vec<PaddedGameState> = pruned_moves
                        .iter()
                        .map(|mov| PaddedGameState::next(padded_gs, *mov, &self.game_globals))
                        .collect();
                    states.sort_by(|gs_1, gs_2| match padded_gs.gs.turn {
                        Player::P1 => gs_2.eval.total_cmp(&gs_1.eval),
                        Player::P2 => gs_1.eval.total_cmp(&gs_2.eval),
                    });
                    let mut utilities = Vec::with_capacity(pruned_moves.len());
                    for state in states {
                        let value = self.min_max(&state, depth - 1, alpha, beta, visited );
                        utilities.push(value);
                        if is_max {
                            alpha = f32::max(alpha, value);
                            if alpha > beta {
                                return alpha;
                            }
                        } else {
                            beta = f32::min(beta, value);
                            if beta < alpha {
                                return beta;
                            }
                        }
                    }
                    let value = utilities.iter().cloned().fold(base_value, selector);

                    visited.insert(padded_gs.gs.to_owned(), value);

                    value
                }
            },
        }
    }
}

impl Agent for MinMaxAgent {
    //TODO stop search when a winning move is found
    //TODO keep calculations from last move
    //TODO prune non promising branches

    fn next_move(&self, gs: &GameState) -> Move {
        // CALL_COUNT_TO_MIN_MAX.store(0, Ordering::SeqCst);
        let next_move_internal = |depth: i32| -> Move {
            let (arg_select, base_value): (fn(&[f32]) -> (usize, f32), f32) =
                if gs.turn == Player::P1 {
                    (argmax, f32::NEG_INFINITY)
                } else {
                    (argmin, f32::NEG_INFINITY)
                };
            let mut alpha: f32 = f32::NEG_INFINITY;
            let mut beta: f32 = f32::INFINITY;

            let padded_gs = PaddedGameState::new_from_game_state(gs);

            let mut visited : HashMap<GameState, f32> = HashMap::new();

            let mut moves = get_legal(&gs);

            let num_moves = moves.len();
            let pruned_moves = if padded_gs.is_symmetrical() {moves.into_iter().take((num_moves+1)/2).collect()} else {moves};
            let mut states: Vec<PaddedGameState> = pruned_moves
                .iter()
                .map(|mov| PaddedGameState::next(&padded_gs, *mov, &self.game_globals))
                .collect();


            let mut utilities = Vec::with_capacity(pruned_moves.len());

            let mut zipped_states : Vec<(&PaddedGameState, Move)>= states.iter().zip(pruned_moves).collect();
            zipped_states.sort_by(|(gs_1, _), (gs_2, _)| match gs.turn {
                Player::P1 => gs_2.eval.total_cmp(&gs_1.eval),
                Player::P2 => gs_1.eval.total_cmp(&gs_2.eval),
            });

            for (state, _) in zipped_states.iter() {
                let value = self.min_max(&state, depth, alpha, beta, &mut visited);
                utilities.push(value);
                alpha = f32::min(alpha, value);
                beta = f32::max(beta, value);
            }
            // println!("{:?}", moves);
            // println!("{:?}", utilities);
            // println!("{:?}", utilities);
            // println!("min_max called {} times.", CALL_COUNT_TO_MIN_MAX.load(Ordering::SeqCst));
            // println!("Utilities {:?}", utilities);
            zipped_states[(arg_select)(&utilities).0].1
        };
        if !self.timed {
            return next_move_internal(self.depth);
        }

        let mut depth = 1;
        let mut mov: Move = next_move_internal(0);

        let start = ProcessTime::try_now().expect("Getting process time failed");

        while start
            .try_elapsed()
            .expect("Getting process time failed")
            .as_millis()
            < ((self.time * 1000) / 7) as u128
        {
            mov = next_move_internal(depth);
            depth += 1;
        }
        println!("Depth: {:?}", depth);
        mov
    }
}
#[cfg(test)]
mod tests {
    use crate::game::{Agent, MinMaxAgent};
    use crate::game_logic::{GameGlobals, PaddedGameState};
    use crate::game_logic::test_utils::{get_random_position, get_random_positions};

    #[test]
    fn min_max_call_count() {
        let gg = GameGlobals::new(6,7);
        // let state = get_random_position(42, 0, &gg);
        let states = get_random_positions(42, 10, &gg);
        // let padded = PaddedGameState::new_from_game_state(&state);
        for state in states.iter(){
            // println!("{}", state);
            for depth in 7..8 {
                let agent = MinMaxAgent::new_with_args(false, 0, depth, 6, 7);
                agent.next_move(&state);
            }
        }

    }
}
