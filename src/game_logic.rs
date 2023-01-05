use std::borrow::{Borrow, BorrowMut};
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::ops::Add;
use std::{fmt, usize};
use std::hash::{Hash, Hasher};

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

pub struct GameGlobals {
    rows: usize,
    cols: usize,
    win_tests: HashMap<Move, Vec<Vec<Vec<(usize, usize)>>>>,
}

impl GameGlobals {
    pub fn new(rows: usize, cols: usize) -> Self {
        let win_tests = Self::get_win_tests(rows, cols);
        Self {
            rows,
            cols,
            win_tests,
        }
    }
    fn get_win_tests(rows: usize, cols: usize) -> HashMap<Move, Vec<Vec<Vec<(usize, usize)>>>> {
        let mut hm = HashMap::new();
        for row in 0..rows {
            for col in 0..cols {
                let mov = Move { row, col };
                hm.insert(mov, Self::get_win_tests_for_move(rows, cols, mov));
            }
        }
        hm
    }
    fn get_win_tests_for_move(
        rows: usize,
        cols: usize,
        mov: Move,
    ) -> Vec<Vec<Vec<(usize, usize)>>> {
        let mut win_squares = Vec::with_capacity(4);
        let base_dirs = vec![1, -1];
        let dirs = vec![(0, 1), (1, 0), (1, 1), (1, -1)];
        let forward_limits: Vec<(i32, i32)> = vec![
            (100, cols as i32 - 1),
            (rows as i32 - 1, 100),
            (rows as i32 - 1, cols as i32 - 1),
            (rows as i32 - 1, 0),
        ];
        let backward_limits: Vec<(i32, i32)> =
            vec![(100, 0), (0, 100), (0, 0), (0, cols as i32 - 1)];
        let limits: Vec<(&(i32, i32), (i32, i32))> =
            forward_limits.iter().zip(backward_limits).collect();

        let Move {
            row: start_row,
            col: start_col,
        } = mov;
        for ((row_dir, col_dir), (limit_1, limit_2)) in dirs.iter().zip(limits) {
            let mut win_squares_dir = Vec::with_capacity(2);
            for (base_dir, (row_limit, col_limit)) in base_dirs.iter().zip(vec![*limit_1, limit_2])
            {
                win_squares_dir.push(
                    (1..(1 + min(
                        i32::abs(row_limit - (start_row as i32 * base_dir * row_dir)),
                        i32::abs(col_limit - (start_col as i32 * base_dir * col_dir)),
                    )))
                        .map(|offset| {
                            (
                                (start_row as i32 + row_dir * offset * base_dir) as usize,
                                (start_col as i32 + col_dir * offset * base_dir) as usize,
                            )
                        })
                        .collect(),
                )
            }
            win_squares.push(win_squares_dir)
        }
        win_squares
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameResult {
    Win(Player),
    Draw,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    row: usize,
    col: usize,
}

pub struct PaddedGameState {
    pub gs: GameState,
    pub eval: f32,
    placed: usize,
    pub unsymmetrical_count : i32
}

impl PaddedGameState {
    pub fn new(game_globals: &GameGlobals) -> Self {
        Self {
            gs: GameState::new(game_globals),
            eval: 0.0,
            placed: 0,
            unsymmetrical_count: 0,
        }
    }
    pub fn new_from_board(raw_board: Vec<Vec<i8>>) -> Self {
        let gs = GameState::new_from_board(raw_board);
        Self::new_from_game_state(&gs)
    }
    pub fn new_from_game_state(gs_ref: &GameState) -> Self {
        let eval = eval(gs_ref);
        let placed = placed_discs(&gs_ref);
        Self {
            gs: gs_ref.clone(),
            eval,
            placed,
            unsymmetrical_count : Self::get_unsymmetrical_count(gs_ref),
        }
    }
    pub fn next(
        old_gs: &PaddedGameState,
        mov: Move,
        game_globals: &GameGlobals,
    ) -> PaddedGameState {
        PaddedGameState {
            gs: play(mov, &old_gs.gs).unwrap(),
            eval: fast_eval(old_gs, mov, &game_globals),
            placed: old_gs.placed + 1,
            unsymmetrical_count : old_gs.unsymmetrical_count + Self::get_unsymmetrical_count_diff(&old_gs.gs, mov)
        }
    }

    fn get_unsymmetrical_count(gs : &GameState) -> i32 {
        let mut count = 0;
        for row in gs.board.iter() {
            for col in 0..(row.len()/2){
                if row[col]!=row[row.len()-col-1]{
                    count+=1;
                }
            }
        }
        count
    }

    fn get_unsymmetrical_count_diff(gs : &GameState, mov : Move) -> i32 {
        if gs.cols.is_odd() && mov.col==gs.cols/2 {0} else if Some(gs.turn) == gs.board[mov.row][gs.cols-mov.col-1] {-1} else {1}
    }

    pub fn is_symmetrical(&self) -> bool {
        self.unsymmetrical_count == 0
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct GameState {
    pub(crate) turn: Player,
    board: Vec<Vec<Disc>>,
    rows: usize,
    cols: usize,
}

impl GameState {
    pub fn new(game_globals: &GameGlobals) -> Self {
        let rows = game_globals.rows;
        let cols = game_globals.cols;
        Self {
            turn: Player::P1,
            board: vec![vec![None; 7]; 6],
            rows,
            cols,
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
        let rows = board.len();
        let cols = board[0].len();
        let placed = placed_discs_board(&board);
        Self {
            turn: if placed.is_even() {
                Player::P1
            } else {
                Player::P2
            },
            board,
            rows,
            cols,
        }
    }
}

impl Hash for GameState {
    fn hash<H: Hasher>(&self, state: &mut H)
    where H: std::hash::Hasher{
        let v : Vec<u8>= self.board.iter().flatten().map(|disc|{
            match disc {
                None => {0},
                Some(p) => if *p==Player::P1{1} else {2}
            }
        }).collect();
        state.write(&v[..])
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

fn placed_discs_board(board: &Vec<Vec<Disc>>) -> usize {
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

pub fn fast_eval(padded_gs: &PaddedGameState, mov: Move, game_globals: &GameGlobals) -> f32 {
    let PaddedGameState { gs, eval, placed, unsymmetrical_count} = padded_gs;
    match fast_result(padded_gs, mov, game_globals) {
        Some(GameResult::Win(p)) if p == Player::P1 => f32::INFINITY,
        Some(GameResult::Win(p)) if p == Player::P2 => f32::NEG_INFINITY,
        Some(GameResult::Draw) => 0.0,
        _ => {
            let change = (fast_num_wins(gs, true, mov, game_globals)) as f32;
            eval + if gs.turn == Player::P1 {
                change
            } else {
                -change
            }
        }
    }
}
pub fn fast_result(
    padded_gs: &PaddedGameState,
    mov: Move,
    game_globals: &GameGlobals,
) -> Option<GameResult> {
    let PaddedGameState { gs, eval, placed, unsymmetrical_count } = padded_gs;
    let player = gs.turn;
    match fast_num_wins(gs, false, mov, game_globals) {
        0 => {}
        _ => return Some(GameResult::Win(player)),
    }
    return if *placed == gs.rows * gs.cols {
        Some(GameResult::Draw)
    } else {
        None
    };
}

pub fn fast_num_wins(
    pre_gs: &GameState,
    possible_wins: bool,
    mov: Move,
    game_globals: &GameGlobals,
) -> i32 {
    let player = pre_gs.turn;
    let mut wins = 0;
    let limits = game_globals.win_tests.get(&mov).unwrap();
    for main_dir in limits {
        let mut ranges = Vec::with_capacity(2);
        for base_dir in main_dir {
            let mut range = 0;
            for (row, col) in base_dir {
                // println!("Start : ({:},{:}) Limit : ({:},{:}) Row : {:} + {:} * {:} * {:}, Col : {:} + {:} * {:} * {:}", start_row, start_col, row_limit, col_limit, start_row, offset, row_dir, base_dir, start_col, offset, col_dir, base_dir);
                match pre_gs.board[*row][*col] {
                    Some(p) if p == player && !possible_wins || p != player && possible_wins => {
                        range += 1
                    }
                    None if possible_wins => range += 1,
                    _ => break,
                }
            }
            ranges.push(range);
        }
        // println!("{:?}",ranges);
        wins += if possible_wins {
            max(ranges[0] + ranges[1] - 2, 0) - max(ranges[0] - 3, 0) - max(ranges[1] - 3, 0)
        } else {
            max(ranges.iter().sum::<i32>() - 2, 0)
        }
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

pub mod test_utils {
    use crate::game_logic::{
        eval, fast_eval, fast_num_wins, fast_result, get_legal, next_turn, num_wins, play, result,
        GameGlobals, GameResult, GameState, Move, PaddedGameState, Player,
    };
    use rand::Rng;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    pub fn get_random_positions(
        depth: i32,
        n: usize,
        game_globals: &GameGlobals,
    ) -> Vec<GameState> {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let mut positions = Vec::with_capacity(n);
        for i in 0..n {
            positions.push(get_random_position(
                rng.gen_range(1..depth),
                i,
                game_globals,
            ));
        }
        positions
    }

    pub fn get_random_position(depth: i32, seed: usize, game_globals: &GameGlobals) -> GameState {
        let mut rng = ChaCha8Rng::seed_from_u64(seed as u64);
        let mut gs = GameState::new(game_globals);
        for _ in 0..depth {
            let moves = get_legal(&gs);
            let next_gs = play(moves[rng.gen_range(0..moves.len())], &gs).unwrap();
            if let Some(_) = result(&next_gs) {
                return gs;
            }
            gs = next_gs;
        }
        gs
    }
}

#[cfg(test)]
mod tests {
    use crate::game_logic::test_utils::get_random_positions;
    use crate::game_logic::{
        eval, fast_eval, fast_num_wins, fast_result, get_legal, next_turn, num_wins, play, result,
        GameGlobals, GameResult, GameState, Move, PaddedGameState, Player,
    };
    use rand::Rng;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;

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
        assert_eq!(eval(&gs), 0.0);
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
    #[test]
    fn fast_eval_function() {
        let game_globals = &GameGlobals::new(6, 7);
        let padded_gs = PaddedGameState::new(&GameGlobals::new(6, 7));
        assert_eq!(eval(&padded_gs.gs), 0.0);
        assert_eq!(
            fast_eval(&padded_gs, Move { row: 5, col: 0 }, game_globals),
            eval(&play(Move { row: 5, col: 0 }, &padded_gs.gs).unwrap())
        );
        let padded_gs = PaddedGameState::new_from_board(vec2d![
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 2, 0, 0, 0, 0, 0],
            [1, 2, 0, 0, 0, 0, 0],
            [1, 2, 0, 0, 0, 0, 0]
        ]);
        assert_eq!(
            fast_eval(&padded_gs, Move { row: 2, col: 0 }, game_globals),
            f32::INFINITY
        );
        assert_eq!(
            fast_result(&padded_gs, Move { row: 2, col: 0 }, game_globals),
            Some(GameResult::Win(Player::P1))
        );
        for mov in get_legal(&padded_gs.gs) {
            assert_eq!(
                fast_eval(&padded_gs, mov, game_globals),
                eval(&play(mov, &padded_gs.gs).unwrap())
            );
        }
    }

    #[test]
    fn fast_eval_function_loop() {
        let game_globals = &GameGlobals::new(6, 7);
        let states = get_random_positions(42, 1000, &GameGlobals::new(6, 7));
        for gs in states {
            let padded_gs = PaddedGameState::new_from_game_state(&gs);
            for mov in get_legal(&padded_gs.gs) {
                assert_eq!(
                    fast_eval(&padded_gs, mov, game_globals),
                    eval(&play(mov, &padded_gs.gs).unwrap())
                );
            }
        }
    }

    #[test]
    fn win_squares() {
        let game_globals = GameGlobals::new(2, 2);
        println!("{:?}", game_globals.win_tests);
    }
}
