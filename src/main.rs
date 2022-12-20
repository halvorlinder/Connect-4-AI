use crate::game::Game;
use crate::game_logic::{GameState, get_legal, play, result};

mod game_logic;
mod game;

fn main() {
    let mut game = Game::new();
    game.start_game();
}
