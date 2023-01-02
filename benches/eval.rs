use criterion::{black_box, criterion_group, criterion_main, Criterion};
use four_in_a_row::game::Game;
use four_in_a_row::game_logic;
use four_in_a_row::game_logic::test_utils::get_random_position;
use four_in_a_row::game_logic::{eval, fast_eval, get_legal, play, GameGlobals, PaddedGameState};

fn criterion_benchmark(c: &mut Criterion) {
    let game_globals = GameGlobals::new(6, 7);
    let gs = get_random_position(42, 1, &game_globals);
    let mov = get_legal(&gs)[0];
    let padded_gs = PaddedGameState::new_from_game_state(gs);

    // c.bench_function("Eval", |b| b.iter(|| eval(&play(mov, &padded_gs.gs).unwrap())));
    c.bench_function("Fast Eval", |b| {
        b.iter(|| fast_eval(&padded_gs, mov, &game_globals))
    });
    // c.bench_function("Test", |b| b.iter(|| { let a = 0; }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
