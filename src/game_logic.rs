use std::cmp::min;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Player{
    P1,
    P2
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Result {
    Player(Player),
    Draw
}

type Disc = Option<Player>;

#[derive(Copy, Clone, Debug)]
pub struct Move {
    row : usize,
    col : usize
}

#[derive(Clone)]
pub struct GameState {
    pub(crate) turn: Player,
    board: Vec<Vec<Disc>>,
    rows: usize,
    cols: usize
}

impl GameState {
    pub fn new() -> Self {
        Self {
            turn : Player::P1,
            board : vec![vec![None ; 7] ; 6],
            rows : 6,
            cols : 7,
        }
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string : String = "+".to_string().add(&"-".repeat(self.cols)).add("+").add("\n|") + &self.board.iter()
            .map(|row| row.iter().map(|disc|
            match disc{
                None => " ",
                Some(Player::P1) => "O",
                Some(Player::P2) => "X"
            }
        ).collect::<Vec<&str>>().join("")).collect::<Vec<String>>().join("|\n|").add("|\n").add("+").add(&"-".repeat(self.cols).add("+").add("\n"));
        write!(f, "{}", string)
    }
}

fn next_turn(p:Player) -> Player{
    match p {
        Player::P1 => Player::P2,
        Player::P2 => Player::P1
    }
}

pub fn play(mov : Move, gs : GameState) -> Option<GameState> {
    let Move {row, col} = mov;
    return match gs.board[row][col] {
        None => {
            if row != gs.rows - 1 && gs.board[row + 1][col].is_none() {
                return None;
            }
            let mut copy = gs.clone();
            copy.board[row][col] = Some(gs.turn);
            copy.turn = next_turn(gs.turn);
            Some(copy)
        }
        _ => { None }
    }

}

fn legal_in_col(gs: GameState, col : usize ) -> Option<Move> {
    for row in (0..gs.rows).rev() {
        if let None = gs.board[row][col] {
            return Some(Move{row, col});
        }
    }
    None
}

pub fn get_legal (gs : &GameState) -> Vec<Move> {
    //performance issue?
    (0..gs.cols).map(|mov| ( legal_in_col(gs.clone(), mov) )).flatten().collect()
}

pub fn result(gs : &GameState) -> Option<Result>{
    let tests: Vec<fn(&GameState, Player) -> bool> = vec![win_in_row, win_in_col, win_in_diag_tl_to_br, win_in_diag_tr_to_bl];
    for p in vec![Player::P1, Player::P2]{
        for f in &tests{
            if f(&gs, p){
                return Some(Result::Player(p));
            }
        }
    }
    return None
}

fn win_in_row(gs : &GameState, player : Player) -> bool{
    for row in 0..gs.rows {
        let mut in_a_row = 0;
        for col in 0..gs.cols {
            match gs.board[row][col] {
                Some(p) if p==player => {in_a_row +=1}
                _ => {in_a_row = 0}
            }
            if in_a_row == 4{
                return true;
            }
        }
    }
    return false;
}

fn win_in_col(gs : &GameState, player : Player) -> bool{
    for col in 0..gs.cols {
        let mut in_a_row = 0;
        for row in 0..gs.rows {
            match gs.board[row][col] {
                Some(p) if p==player => {in_a_row +=1}
                _ => {in_a_row = 0}
            }
            if in_a_row == 4{
                return true;
            }
        }
    }
    return false;
}

fn win_in_diag_tl_to_br(gs : &GameState, player : Player) -> bool{
    let starts_side : Vec<(usize, usize)> = (0..gs.rows-3).map(|start_row| (start_row, 0)).collect();
    let starts_top : Vec<(usize, usize)> = (0..gs.cols-3).map(|start_col| (0, start_col)).collect();
    for ( start_row, start_col ) in [starts_side, starts_top].concat() {
        let mut in_a_row = 0;
        for offset in 0..min::<usize>(gs.rows-start_row, gs.cols-start_col) {
            match gs.board[start_row+offset][start_col + offset] {
                Some(p) if p==player => {in_a_row +=1}
                _ => {in_a_row = 0}
            }
            if in_a_row == 4{
                return true;
            }
        }

    }
    return false;
}

fn win_in_diag_tr_to_bl(gs : &GameState, player : Player) -> bool{
    let starts_side : Vec<(usize, usize)> = (0..gs.rows-3).map(|start_row| (start_row, gs.cols-1)).collect();
    let starts_top : Vec<(usize, usize)> = (3..gs.cols).map(|start_col| (0, start_col)).collect();
    for ( start_row, start_col ) in [starts_side, starts_top].concat() {
        let mut in_a_row = 0;
        for offset in 0..min::<usize>(gs.rows-start_row, start_col) {
            match gs.board[start_row+offset][start_col - offset] {
                Some(p) if p==player => {in_a_row +=1}
                _ => {in_a_row = 0}
            }
            if in_a_row == 4{
                return true;
            }
        }

    }
    return false;
}

fn eval (gs : GameState) -> f32{
    0.0
}
