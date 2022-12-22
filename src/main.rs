use crate::game::Game;
use crate::game_logic::{get_legal, play, result, GameState};

mod game;
mod game_logic;

fn main() {
    let mut game = Game::new();
    game.start_game();
}
