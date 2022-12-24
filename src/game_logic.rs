use std::borrow::Borrow;
use std::cmp::{max, min};
use std::{fmt, usize};
use std::fmt::Formatter;
use std::ops::Add;

use num_integer::Integer;

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
pub enum Player {
    P1,
    P2,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = match self {
            (Player::P1) => "\u{001b}[31mP1\u{001b}[0m",
            (Player::P2) => "\u{001b}[33mP2\u{001b}[0m",
            _ => "",
        };
        write!(f, "{}", string)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameResult {
    Win(Player),
    Draw,
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = match self {
            GameResult::Draw => "\u{001b}[34mDraw\u{001b}[0m".to_string(),
            GameResult::Win(player) => format!("{}", player),
        };
        write!(f, "{}", string)
    }
}

type Disc = Option<Player>;

#[derive(Copy, Clone, Debug)]
pub struct Move {
    row: usize,
    col: usize,
}

pub struct PaddedGameState{
    gs : GameState,
    eval: f32,
    placed : usize,
}

impl PaddedGameState {
    pub fn new() -> Self {
        Self {
            gs : GameState::new(),
            eval : 0.0,
            placed : 0
        }
    }
    pub fn new_from_board(raw_board: Vec<Vec<i8>>) -> Self {
        let gs = GameState::new_from_board(raw_board);
        Self::new_from_game_state(gs)
    }
    pub fn new_from_game_state(gs : GameState) -> Self {
        let eval = eval(&gs);
        let placed = placed_discs(&gs);
        Self {
            gs,
            eval,
            placed,
        }
    }
}

#[derive(Clone)]
pub struct GameState {
    pub(crate) turn: Player,
    board: Vec<Vec<Disc>>,
    rows: usize,
    cols: usize,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            turn: Player::P1,
            board: vec![vec![None; 7]; 6],
            rows: 6,
            cols: 7,
        }
    }
    pub fn new_from_board(raw_board: Vec<Vec<i8>>) -> Self {
        let board: Vec<Vec<Disc>> = raw_board
            .iter()
            .map(|row| {
                row.iter()
                    .map(|n| match n {
                        1 => Some(Player::P1),
                        2 => Some(Player::P2),
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        let placed = placed_discs_board(&board);
        Self {
            turn: if placed.is_even() {Player::P1} else {Player::P2},
            board,
            rows: 6,
            cols: 7,
        }
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string: String = format!("To play : {:}\n", self.turn)
            .add("+")
            .add(&"-".repeat(self.cols))
            .add("+")
            .add("\n|")
            + &self
                .board
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|disc| match disc {
                            None => "\u{001b}[34m.\u{001b}[0m",
                            Some(Player::P1) => "\u{001b}[31mO\u{001b}[0m",
                            Some(Player::P2) => "\u{001b}[33mO\u{001b}[0m",
                        })
                        .collect::<Vec<&str>>()
                        .join("")
                })
                .collect::<Vec<String>>()
                .join("|\n|")
                .add("|\n")
                .add("+")
                .add(&"-".repeat(self.cols).add("+").add("\n"));
        write!(f, "{}", string)
    }
}

fn next_turn(p: Player) -> Player {
    match p {
        Player::P1 => Player::P2,
        Player::P2 => Player::P1,
    }
}

pub fn play(mov: Move, gs: &GameState) -> Option<GameState> {
    let Move { row, col } = mov;
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
        _ => None,
    };
}

fn legal_in_col(gs: GameState, col: usize) -> Option<Move> {
    for row in (0..gs.rows).rev() {
        if let None = gs.board[row][col] {
            return Some(Move { row, col });
        }
    }
    None
}

pub fn get_legal(gs: &GameState) -> Vec<Move> {
    //performance issue?
    (0..gs.cols)
        .map(|mov| (legal_in_col(gs.clone(), mov)))
        .flatten()
        .collect()
}

pub fn result(gs: &GameState) -> Option<GameResult> {
    for p in vec![Player::P1, Player::P2] {
        match num_wins(gs, p, false) {
            0 => {}
            _ => return Some(GameResult::Win(p)),
        }
    }
    return if is_full(gs) {
        Some(GameResult::Draw)
    } else {
        None
    };
}

fn is_full(gs: &GameState) -> bool {
    !gs.board.iter().flatten().any(|disc| disc.is_none())
}

fn placed_discs(gs: &GameState) -> usize {
    placed_discs_board(&gs.board)
}

fn placed_discs_board(board : &Vec<Vec<Disc>> ) -> usize {
    board.iter().flatten().filter(|disc| disc.is_some()).count()
}

fn win_in_row(gs: &GameState, player: Player, possible_wins: bool) -> i32 {
    let mut wins = 0;
    for row in 0..gs.rows {
        let mut in_a_row = 0;
        for col in 0..gs.cols {
            match gs.board[row][col] {
                Some(p) if p == player => in_a_row += 1,
                None if possible_wins => in_a_row += 1,
                _ => in_a_row = 0,
            }
            if in_a_row == 4 {
                wins += 1;
                in_a_row = 3;
            }
        }
    }
    return wins;
}

fn win_in_col(gs: &GameState, player: Player, possible_wins: bool) -> i32 {
    let mut wins = 0;
    for col in 0..gs.cols {
        let mut in_a_row = 0;
        for row in 0..gs.rows {
            match gs.board[row][col] {
                Some(p) if p == player => in_a_row += 1,
                None if possible_wins => in_a_row += 1,
                _ => in_a_row = 0,
            }
            if in_a_row == 4 {
                wins += 1;
                in_a_row = 3;
            }
        }
    }
    return wins;
}

fn win_in_diag_tl_to_br(gs: &GameState, player: Player, possible_wins: bool) -> i32 {
    let mut wins = 0;
    let starts_side: Vec<(usize, usize)> =
        (0..gs.rows - 3).map(|start_row| (start_row, 0)).collect();
    let starts_top: Vec<(usize, usize)> =
        (1..gs.cols - 3).map(|start_col| (0, start_col)).collect();
    for (start_row, start_col) in [starts_side, starts_top].concat() {
        let mut in_a_row = 0;
        for offset in 0..min::<usize>(gs.rows - start_row, gs.cols - start_col) {
            match gs.board[start_row + offset][start_col + offset] {
                Some(p) if p == player => in_a_row += 1,
                None if possible_wins => in_a_row += 1,
                _ => in_a_row = 0,
            }
            if in_a_row == 4 {
                wins += 1;
                in_a_row = 3;
            }
        }
    }
    return wins;
}

fn win_in_diag_tr_to_bl(gs: &GameState, player: Player, possible_wins: bool) -> i32 {
    let mut wins = 0;
    let starts_side: Vec<(usize, usize)> = (0..gs.rows - 3)
        .map(|start_row| (start_row, gs.cols - 1))
        .collect();
    let starts_top: Vec<(usize, usize)> =
        (3..gs.cols - 1).map(|start_col| (0, start_col)).collect();
    for (start_row, start_col) in [starts_side, starts_top].concat() {
        let mut in_a_row = 0;
        for offset in 0..min::<usize>(gs.rows - start_row, start_col + 1) {
            match gs.board[start_row + offset][start_col - offset] {
                Some(p) if p == player => in_a_row += 1,
                None if possible_wins => in_a_row += 1,
                _ => in_a_row = 0,
            }
            if in_a_row == 4 {
                wins += 1;
                in_a_row = 3;
            }
        }
    }
    return wins;
}

pub fn eval(gs: &GameState) -> f32 {
    match result(gs) {
        Some(GameResult::Win(p)) if p == Player::P1 => f32::INFINITY,
        Some(GameResult::Win(p)) if p == Player::P2 => f32::NEG_INFINITY,
        Some(GameResult::Draw) => 0.0,
        _ => (num_wins(gs, Player::P1, true) - num_wins(gs, Player::P2, true)) as f32,
    }
}

pub fn fast_eval(padded_gs : &PaddedGameState, mov : Move) -> f32{
    let PaddedGameState{ gs, eval, placed  } = padded_gs;
    match fast_result(padded_gs, mov) {
        Some(GameResult::Win(p)) if p == Player::P1 => f32::INFINITY,
        Some(GameResult::Win(p)) if p == Player::P2 => f32::NEG_INFINITY,
        Some(GameResult::Draw) => 0.0,
        _ => {
            let change = (fast_num_wins(gs, true, mov)) as f32;
            eval + if gs.turn == Player::P1 {change} else {-change}
        },
    }
}
pub fn fast_result(padded_gs : &PaddedGameState, mov : Move) -> Option<GameResult> {
    let PaddedGameState{ gs, eval, placed  } = padded_gs;
    let player = gs.turn;
    match fast_num_wins(gs, false, mov) {
        0 => {}
        _ => return Some(GameResult::Win(player)),
    }
    return if *placed==gs.rows*gs.cols {
        Some(GameResult::Draw)
    } else {
        None
    };
}

pub fn fast_num_wins(pre_gs : &GameState, possible_wins : bool, mov : Move) -> i32{
    let player = pre_gs.turn;
    let mut wins = 0;
    let base_dirs = vec![ 1,-1 ];
    let dirs  = vec![(0,1), (1,0), (1,1), (1,-1)];
    let forward_limits : Vec<(i32, i32)> = vec![(100, pre_gs.cols as i32 -1), (pre_gs.rows as i32 -1, 100), (pre_gs.rows as i32 -1, pre_gs.cols as i32 -1), (pre_gs.rows as i32 -1, 0)];
    let backward_limits : Vec<(i32, i32)> = vec![(100, 0), (0, 100), (0, 0), (0, pre_gs.cols as i32 -1)];
    let limits : Vec<(&(i32,i32),(i32,i32))> = forward_limits.iter().zip(backward_limits).collect();

    let Move{row: start_row, col: start_col} = mov;
    for ( (row_dir, col_dir), ( limit_1, limit_2 ) ) in dirs.iter().zip(limits){
        let mut ranges = Vec::with_capacity(2);
        for (base_dir, (row_limit, col_limit) ) in base_dirs.iter().zip(vec![*limit_1, limit_2]){
            let mut range = 0;
            for offset in 1..(1+min( i32::abs( row_limit - (start_row as i32 * base_dir * row_dir) ), i32::abs(col_limit - (start_col as i32 * base_dir * col_dir)))){
                // println!("Start : ({:},{:}) Limit : ({:},{:}) Row : {:} + {:} * {:} * {:}, Col : {:} + {:} * {:} * {:}", start_row, start_col, row_limit, col_limit, start_row, offset, row_dir, base_dir, start_col, offset, col_dir, base_dir);
                match pre_gs.board[( start_row as i32 + row_dir*offset*base_dir ) as usize][( start_col as i32 + col_dir*offset*base_dir ) as usize] {
                    Some(p) if p == player && !possible_wins || p!=player && possible_wins => range += 1,
                    None if possible_wins => range += 1,
                    _ => {break},
                }
            }
            ranges.push(range);
        }
        // println!("{:?}",ranges);
        wins += if possible_wins {max(ranges[0]+ranges[1]-2,0)-max(ranges[0]-3, 0)-max(ranges[1]-3, 0)} else {max(ranges.iter().sum::<i32>()-2, 0)}
    }
    wins
}

fn num_wins(gs: &GameState, player: Player, possible_wins: bool) -> i32 {
    let tests: Vec<fn(&GameState, Player, bool) -> i32> = vec![
        win_in_row,
        win_in_col,
        win_in_diag_tl_to_br,
        win_in_diag_tr_to_bl,
    ];
    let mut wins = 0;
    for f in &tests {
        wins += f(&gs, player, possible_wins);
    }

    return wins;
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::game_logic::{eval, result, GameResult, GameState, Player, fast_eval, Move, fast_num_wins, play, num_wins, PaddedGameState, fast_result, get_legal, next_turn};
    use rand_chacha::ChaCha8Rng;
    use rand_chacha::rand_core::SeedableRng;

    fn get_random_positions(depth : i32, n : usize) -> Vec<GameState> {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let mut positions = Vec::with_capacity(n);
        for i in 0..n {
            positions.push(get_random_position(rng.gen_range(1..depth), i));
        }
        positions
    }

    fn get_random_position(depth : i32, seed : usize) -> GameState {
        let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
        let mut gs = GameState::new();
        for _ in 0..depth{
            let moves = get_legal(&gs);
            let next_gs = play(moves[rng.gen_range(0..moves.len())], &gs).unwrap();
            if let Some(_) = result(&next_gs){
                return gs;
            }
            gs = next_gs;
        }
        gs
    }

    #[test]
    fn win_check_horizontal() {
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 1, 1, 1, 1],
            [0, 0, 1, 2, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 1, 2, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 1, 1, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 1, 1, 1, 0],
            [1, 2, 2, 2, 2, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P2)));
    }
    #[test]
    fn win_check_vertical() {
        let gs = GameState::new_from_board(vec2d![
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 1, 2, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 1, 2, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1],
            [1, 1, 0, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 1, 1, 1, 0],
            [1, 2, 2, 0, 2, 0, 0],
            [0, 0, 2, 0, 0, 0, 0],
            [0, 0, 2, 0, 0, 0, 0],
            [0, 0, 2, 0, 0, 0, 0],
            [0, 1, 1, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P2)));
    }
    #[test]
    fn win_check_diag_tl_to_br() {
        let gs = GameState::new_from_board(vec2d![
            [1, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 2, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 0],
            [1, 0, 0, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 2, 2, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 1],
            [0, 0, 0, 0, 1, 0, 1],
            [0, 0, 0, 0, 0, 1, 0],
            [1, 1, 0, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 1, 1, 1, 0],
            [1, 2, 2, 0, 2, 0, 0],
            [1, 0, 2, 0, 0, 0, 0],
            [0, 1, 0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0, 0, 0],
            [0, 1, 1, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 1, 1, 1, 0],
            [1, 2, 2, 0, 1, 0, 0],
            [1, 0, 2, 0, 0, 1, 0],
            [0, 1, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
    }
    #[test]
    fn win_check_diag_tr_to_bl() {
        let gs = GameState::new_from_board(vec2d![
            [1, 0, 0, 0, 0, 0, 1],
            [0, 1, 1, 2, 0, 1, 0],
            [1, 0, 0, 0, 1, 0, 0],
            [1, 0, 0, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 2, 2, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 1],
            [0, 0, 1, 0, 0, 0, 0],
            [0, 1, 0, 0, 0, 1, 0],
            [1, 1, 0, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 1, 0, 1, 0],
            [1, 2, 2, 0, 2, 0, 0],
            [1, 0, 2, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 1, 0],
            [0, 0, 1, 0, 1, 0, 0],
            [0, 1, 0, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 1, 0, 1, 0],
            [1, 2, 1, 0, 0, 0, 0],
            [1, 1, 2, 0, 0, 1, 0],
            [1, 1, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 1, 0, 0, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Win(Player::P1)));
    }

    #[test]
    fn draw() {
        let gs = GameState::new_from_board(vec2d![
            [2, 1, 2, 1, 1, 2, 1],
            [2, 1, 1, 2, 1, 2, 1],
            [1, 2, 1, 2, 1, 1, 2],
            [1, 2, 1, 1, 2, 1, 2],
            [1, 2, 2, 1, 2, 2, 1],
            [2, 1, 1, 1, 2, 2, 1]
        ]);
        assert_eq!(result(&gs), Some(GameResult::Draw));
    }
    #[test]
    fn no_result() {
        let gs = GameState::new_from_board(vec2d![
            [0, 1, 2, 1, 1, 2, 1],
            [2, 1, 1, 2, 1, 2, 1],
            [1, 2, 1, 2, 1, 1, 2],
            [1, 2, 1, 1, 2, 1, 2],
            [1, 2, 2, 1, 2, 2, 1],
            [2, 1, 1, 1, 2, 2, 1]
        ]);
        assert_eq!(result(&gs), None);
    }

    #[test]
    fn eval_function() {
        let gs = GameState::new_from_board(vec2d![
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0]
        ]);
        assert_eq!(eval(&gs), 69.0);
        let gs = GameState::new_from_board(vec2d![
            [0, 1, 2, 1, 1, 2, 1],
            [2, 1, 1, 2, 1, 2, 1],
            [1, 2, 1, 2, 1, 1, 2],
            [1, 2, 1, 1, 2, 1, 2],
            [1, 2, 2, 1, 2, 2, 1],
            [2, 1, 1, 1, 2, 2, 1]
        ]);
        assert_eq!(eval(&gs), 1.0);
        let gs = GameState::new_from_board(vec2d![
            [2, 1, 2, 1, 1, 2, 1],
            [2, 1, 1, 2, 1, 2, 1],
            [1, 2, 1, 2, 1, 1, 2],
            [1, 2, 1, 1, 2, 1, 2],
            [1, 2, 2, 1, 2, 2, 1],
            [2, 1, 1, 1, 2, 2, 1]
        ]);
        assert_eq!(eval(&gs), 0.0);
    }
}
