use std::borrow::Borrow;
use std::cmp::min;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;

macro_rules!vec2d {
    [ $( [ $( $d:expr ),* ] ),* ] => {
        vec![
            $(
                vec![$($d),*],
            )*
        ]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Player{
    P1,
    P2
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string  =
            match self{
                (Player::P1) => "\u{001b}[31mP1\u{001b}[0m",
                (Player::P2) => "\u{001b}[33mP2\u{001b}[0m",
                _ => {""}
            };
        write!(f, "{}", string)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameResult {
    Win(Player),
    Draw
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string  =
            match self{
                GameResult::Draw => "\u{001b}[34mDraw\u{001b}[0m".to_string(),
                GameResult::Win(player) => format!("{}", player)
            };
        write!(f, "{}", string)
    }
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
    pub fn new_from_board(raw_board: Vec<Vec<i8>>) -> Self {
        let board : Vec<Vec<Disc>>  = raw_board.iter().map(|row| row.iter().map(|n| match n {
            1 => Some(Player::P1),
            2 => Some(Player::P2),
            _ => None
        }).collect()).collect();
        Self {
            turn : Player::P1,
            board,
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
                None => "\u{001b}[34m.\u{001b}[0m",
                Some(Player::P1) => "\u{001b}[31mO\u{001b}[0m",
                Some(Player::P2) => "\u{001b}[33mO\u{001b}[0m"
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

pub fn play(mov : Move, gs : &GameState) -> Option<GameState> {
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

pub fn result(gs : &GameState) -> Option<GameResult>{
    for p in vec![Player::P1, Player::P2]{
        match num_wins(gs, p, false){
            0 => {}
            _ => {return Some(GameResult::Win(p))}
        }
    }
    return if is_full(gs) {Some(GameResult::Draw)}else {None};
}

fn is_full(gs : &GameState) -> bool{
    !gs.board.iter().flatten().any(|disc| disc.is_none())
}

fn win_in_row(gs : &GameState, player : Player, possible_wins : bool) -> i32{
    let mut wins = 0;
    for row in 0..gs.rows {
        let mut in_a_row = 0;
        for col in 0..gs.cols {
            match gs.board[row][col] {
                Some(p) if p==player => {in_a_row +=1}
                None if possible_wins => {in_a_row +=1}
                _ => {in_a_row = 0}
            }
            if in_a_row == 4{
                wins+=1;
                in_a_row = 3;
            }
        }
    }
    return wins;
}

fn win_in_col(gs : &GameState, player : Player, possible_wins : bool) -> i32{
    let mut wins = 0;
    for col in 0..gs.cols {
        let mut in_a_row = 0;
        for row in 0..gs.rows {
            match gs.board[row][col] {
                Some(p) if p==player => {in_a_row +=1}
                None if possible_wins => {in_a_row +=1}
                _ => {in_a_row = 0}
            }
            if in_a_row == 4{
                wins+=1;
                in_a_row = 3;
            }
        }
    }
    return wins;
}

fn win_in_diag_tl_to_br(gs : &GameState, player : Player, possible_wins : bool) -> i32{
    let mut wins = 0;
    let starts_side : Vec<(usize, usize)> = (0..gs.rows-3).map(|start_row| (start_row, 0)).collect();
    let starts_top : Vec<(usize, usize)> = (1..gs.cols-3).map(|start_col| (0, start_col)).collect();
    for ( start_row, start_col ) in [starts_side, starts_top].concat() {
        let mut in_a_row = 0;
        for offset in 0..min::<usize>(gs.rows-start_row, gs.cols-start_col) {
            match gs.board[start_row+offset][start_col + offset] {
                Some(p) if p==player => {in_a_row +=1}
                None if possible_wins => {in_a_row +=1}
                _ => {in_a_row = 0}
            }
            if in_a_row == 4{
                wins+=1;
                in_a_row = 3;
            }
        }

    }
    return wins;
}

fn win_in_diag_tr_to_bl(gs : &GameState, player : Player, possible_wins : bool) -> i32{
    let mut wins = 0;
    let starts_side : Vec<(usize, usize)> = (0..gs.rows-3).map(|start_row| (start_row, gs.cols-1)).collect();
    let starts_top : Vec<(usize, usize)> = (3..gs.cols-1).map(|start_col| (0, start_col)).collect();
    for ( start_row, start_col ) in [starts_side, starts_top].concat() {
        let mut in_a_row = 0;
        for offset in 0..min::<usize>(gs.rows-start_row, start_col + 1) {
            match gs.board[start_row+offset][start_col - offset] {
                Some(p) if p==player => {in_a_row +=1}
                None if possible_wins => {in_a_row +=1}
                _ => {in_a_row = 0}
            }
            if in_a_row == 4{
                wins+=1;
                in_a_row = 3;
            }
        }

    }
    return wins;
}

pub fn eval (gs : &GameState) -> f32{
    num_wins(gs, gs.turn, true) as f32
}

fn num_wins(gs : &GameState, player : Player, possible_wins : bool ) -> i32 {
    let tests: Vec<fn(&GameState, Player, bool) -> i32> = vec![win_in_row, win_in_col, win_in_diag_tl_to_br, win_in_diag_tr_to_bl];
    let mut wins = 0;
    for f in &tests{
        wins += f(&gs, player, possible_wins);
    }

    return wins;
}

#[cfg(test)]
mod tests {
    use crate::game_logic::{eval, GameResult, GameState, Player, result};

    #[test]
    fn win_check_horizontal() {
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,1,1,1,1],
                [0,0,1,2,0,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [2,0,0,0,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,0,0,0,0],
                [0,0,1,2,0,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [1,1,1,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,1,1,1,0],
                [1,2,2,2,2,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [0,1,1,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P2)));
    }
    #[test]
    fn win_check_vertical() {
        let gs = GameState::new_from_board(
            vec2d![
                [1,0,0,0,0,0,0],
                [1,0,1,2,0,0,0],
                [1,0,0,0,0,0,0],
                [1,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [2,0,0,0,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,0,0,0,0],
                [0,0,1,2,0,0,0],
                [0,0,0,0,0,0,1],
                [0,0,0,0,0,0,1],
                [0,0,0,0,0,0,1],
                [1,1,0,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,1,1,1,0],
                [1,2,2,0,2,0,0],
                [0,0,2,0,0,0,0],
                [0,0,2,0,0,0,0],
                [0,0,2,0,0,0,0],
                [0,1,1,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P2)));
    }
    #[test]
    fn win_check_diag_tl_to_br() {
        let gs = GameState::new_from_board(
            vec2d![
                [1,0,0,0,0,0,0],
                [0,1,1,2,0,0,0],
                [1,0,1,0,0,0,0],
                [1,0,0,1,0,0,0],
                [0,0,0,0,0,0,0],
                [2,0,0,0,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,0,0,0,0],
                [0,0,2,2,0,0,0],
                [0,0,0,1,0,0,1],
                [0,0,0,0,1,0,1],
                [0,0,0,0,0,1,0],
                [1,1,0,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,1,1,1,0],
                [1,2,2,0,2,0,0],
                [1,0,2,0,0,0,0],
                [0,1,0,0,0,0,0],
                [0,0,1,0,0,0,0],
                [0,1,1,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,1,1,1,0],
                [1,2,2,0,1,0,0],
                [1,0,2,0,0,1,0],
                [0,1,0,0,0,0,1],
                [0,0,0,0,0,0,0],
                [0,1,0,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
    }
    #[test]
    fn win_check_diag_tr_to_bl() {
        let gs = GameState::new_from_board(
            vec2d![
                [1,0,0,0,0,0,1],
                [0,1,1,2,0,1,0],
                [1,0,0,0,1,0,0],
                [1,0,0,1,0,0,0],
                [0,0,0,0,0,0,0],
                [2,0,0,0,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,0,0,0,0],
                [0,0,2,2,0,0,0],
                [0,0,0,1,0,0,1],
                [0,0,1,0,0,0,0],
                [0,1,0,0,0,1,0],
                [1,1,0,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,1,0,1,0],
                [1,2,2,0,2,0,0],
                [1,0,2,0,0,0,1],
                [0,0,0,0,0,1,0],
                [0,0,1,0,1,0,0],
                [0,1,0,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,1,0,1,0],
                [1,2,1,0,0,0,0],
                [1,1,2,0,0,1,0],
                [1,1,0,0,0,0,1],
                [0,0,0,0,0,0,0],
                [0,1,0,1,0,0,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
    }

    #[test]
    fn draw() {
        let gs = GameState::new_from_board(
            vec2d![
                [2,1,2,1,1,2,1],
                [2,1,1,2,1,2,1],
                [1,2,1,2,1,1,2],
                [1,2,1,1,2,1,2],
                [1,2,2,1,2,2,1],
                [2,1,1,1,2,2,1]
            ]
        );
        assert_eq!(result(&gs), Some(GameResult::Draw));
    }
    #[test]
    fn no_result() {
        let gs = GameState::new_from_board(
            vec2d![
                [0,1,2,1,1,2,1],
                [2,1,1,2,1,2,1],
                [1,2,1,2,1,1,2],
                [1,2,1,1,2,1,2],
                [1,2,2,1,2,2,1],
                [2,1,1,1,2,2,1]
            ]
        );
        assert_eq!(result(&gs), None);
    }

    #[test]
    fn eval_function() {
        let gs = GameState::new_from_board(
            vec2d![
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0],
                [0,0,0,0,0,0,0]
            ]
        );
        assert_eq!(eval(&gs), 69.0);
        let gs = GameState::new_from_board(
            vec2d![
                [0,1,2,1,1,2,1],
                [2,1,1,2,1,2,1],
                [1,2,1,2,1,1,2],
                [1,2,1,1,2,1,2],
                [1,2,2,1,2,2,1],
                [2,1,1,1,2,2,1]
            ]
        );
        assert_eq!(eval(&gs), 1.0);
        let gs = GameState::new_from_board(
            vec2d![
                [2,1,2,1,1,2,1],
                [2,1,1,2,1,2,1],
                [1,2,1,2,1,1,2],
                [1,2,1,1,2,1,2],
                [1,2,2,1,2,2,1],
                [2,1,1,1,2,2,1]
            ]
        );
        assert_eq!(eval(&gs), 0.0);
    }
}
