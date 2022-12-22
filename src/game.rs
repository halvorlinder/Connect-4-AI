use std::io;
use crate::game_logic::{play, GameState, Move, get_legal, Player, result, GameResult, eval};
use rand::prelude::*;
use rulinalg::utils;
use rulinalg::utils::{argmax, argmin};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub struct Game {
    gs: GameState,
    player_1: Box<dyn Agent>,
    player_2: Box<dyn Agent>,
}

impl Game {

    fn generate_agent(player : Player) -> Box<dyn Agent> {
        println!("Please select agent type for {:}", player);
        let agent_types : Vec<Agents> = Agents::iter().collect();
        Game::display_agent_options(&agent_types);
        let index = get_int_in_range_from_user(0, agent_types.len());
        let agent = agent_types[index];
        <dyn Agent>::new(agent)
    }

    fn display_agent_options(options : &Vec<Agents>){
        for (i, option) in options.into_iter().enumerate(){
            println!("{:}: {:?}", i, option)
        }
    }

    pub fn new() -> Self {
        Self {
            gs : GameState::new(),
            player_1 : Game::generate_agent(Player::P1),
            player_2 : Game::generate_agent(Player::P2),
        }
    }

    fn play(&mut self, mov: Move) -> bool {
        return match play(mov, &self.gs){
            None => false,
            Some(gs) => {
                self.gs = gs;
                true
            }
        };
    }

    fn next(&mut self) -> Option<GameResult>{
        let mov = match self.gs.turn{
            Player::P1 => {&self.player_1}
            Player::P2 => {&self.player_2}
        }.next_move(& self.gs);
        self.play(mov);
        return result(&self.gs);
    }

    fn game_over(&self, res : GameResult){
        println!("{:}", self.gs);
        println!("The game ended with the following result: {:}", res)
    }

    fn game_loop(&mut self){
        println!("{:}", self.gs);
        let res = self.next();
        match res {
            None => { self.game_loop(); }
            Some(r) => {self.game_over(r)}
        }
    }

    pub fn start_game(&mut self){
        self.game_loop();
    }
}

pub trait Agent {
    fn next_move(&self, gs: &GameState) -> Move;
}

impl dyn Agent {
    pub fn new(agent_type : Agents) -> Box<dyn Agent> {
        let agent : Box<dyn Agent> = (match agent_type {
            Agents::Human => Box::new(Human::new()),
            Agents::RandomMover => Box::new(RandomMover::new()),
            Agents::MinMaxAgent => Box::new(MinMaxAgent::new()),
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
        Self {
        }
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

fn get_int_in_range_from_user(lower : usize, upper : usize)-> usize{
    fn print_illegal() {
        println!("Illegal input!");
    }
    loop {
        let mut input_line = String::new();
        let res = io::stdin()
            .read_line(&mut input_line);
        match res{
            Err(_) => {
                print_illegal();
                continue
            },
            _ => {}
        }
        let index_res : Result<usize, _> = input_line.trim().parse();
        match index_res {
            Ok(i) if i >=lower && i < upper => { return i }
            _ => {
                print_illegal();
                continue
            },
        };
    }

}

pub struct RandomMover {}

impl RandomMover {
    pub fn new() -> Self {
        Self {
        }
    }
}
impl Agent for RandomMover {
    fn next_move(&self, gs: &GameState) -> Move {
        let moves = get_legal(&gs);
        let mut rng = rand::thread_rng();
        return moves[rng.gen_range(0..moves.len())]
    }
}

pub struct MinMaxAgent {}
impl MinMaxAgent {
    pub fn new() -> Self {
        Self {
        }
    }

    fn min_max(&self, gs: &GameState, depth : i8, max_node : bool) -> f32 {
        let e = eval(gs);
        match e {
            f32::INFINITY => f32::INFINITY,
            f32::NEG_INFINITY => f32::NEG_INFINITY,
            _ =>
                match depth {
                    0 => eval(gs),
                    d => {
                        let moves = get_legal(&gs);
                        let states : Vec<GameState>= moves.iter().map(|mov| play(*mov, gs).unwrap()).collect();
                        let utilities : Vec<f32> = states.iter().map(|state| self.min_max(state, d-1, !max_node)).collect();
                        utilities.iter().cloned().fold(if gs.turn==Player::P1 { f32::NEG_INFINITY } else {f32::INFINITY}, if gs.turn==Player::P1 { f32::max } else {f32::min})
                    }
                }
        }


    }
}

impl Agent for MinMaxAgent {
    fn next_move(&self, gs: &GameState) -> Move {
        let moves = get_legal(&gs);
        let states : Vec<GameState>= moves.iter().map(|mov| play(*mov, gs).unwrap()).collect();
        let utilities : Vec<f32> = states.iter().map(|state| self.min_max(state, 4, gs.turn==Player::P1)).collect();
        // println!("{:?}", moves);
        // println!("{:?}", utilities);
        moves[(if gs.turn == Player::P1 { argmax } else {argmin} )(&utilities).0]
    }
}
