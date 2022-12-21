use std::io;
use crate::game_logic::{play, GameState, Move, get_legal, Player, result, GameResult, eval};
use rand::prelude::*;
use rulinalg::utils;
use rulinalg::utils::argmax;

pub struct Game {
    gs: GameState,
    player_1: Box<dyn Agent>,
    player_2: Box<dyn Agent>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            gs : GameState::new(),
            player_1 : Box::new(Human::new()),
            player_2 : Box::new(MinMaxAgent::new()),
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

trait Agent {
    fn next_move(&self, gs: &GameState) -> Move;
}

struct Human {}

impl Human {
    pub fn new() -> Self {
        Self {
        }
    }
}
impl Agent for Human {
    fn next_move(&self, gs: &GameState) -> Move {
        fn print_illegal() {
            println!("Illegal input!");
        }
        let moves = get_legal(&gs);
        println!("{:?}", moves);
        println!("{:} to move. Select a move from the list", gs.turn);
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
            let mut index = 0;
            match index_res {
                Err(_) => {
                    print_illegal();
                    continue
                },
                Ok(i) => { index = i }
            }
            if index >=0 && index < moves.len(){
                return moves[index]
            }
            print_illegal();
        }
    }
}

struct RandomMover {}
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

struct MinMaxAgent {}
impl MinMaxAgent {
    pub fn new() -> Self {
        Self {
        }
    }
}
impl Agent for MinMaxAgent {
    fn next_move(&self, gs: &GameState) -> Move {
        let moves = get_legal(&gs);
        let states : Vec<GameState>= moves.iter().map(|mov| play(*mov, gs).unwrap()).collect();
        let utilities : Vec<f32> = states.iter().map(|state| eval(state)).collect();
        moves[argmax(&utilities).0]
    }
}
