use criterion::{black_box, criterion_group, criterion_main, Criterion};
use four_in_a_row::game::{Agent, Game, MinMaxAgent};
use four_in_a_row::game_logic;
use four_in_a_row::game_logic::test_utils::get_random_position;
use four_in_a_row::game_logic::{eval, fast_eval, get_legal, play, GameGlobals, PaddedGameState};


fn fast_eval_benchmark(c: &mut Criterion) {
    let game_globals = GameGlobals::new(6, 7);
    let gs = get_random_position(42, 1, &game_globals);
    let mov = get_legal(&gs)[0];
    let padded_gs = PaddedGameState::new_from_game_state(&gs);

    c.bench_function("Fast Eval", |b| {
        b.iter(|| fast_eval(&padded_gs, mov, &game_globals))
    });
}

fn eval_benchmark(c: &mut Criterion) {
    let game_globals = GameGlobals::new(6, 7);
    let gs = get_random_position(42, 1, &game_globals);
    let mov = get_legal(&gs)[0];
    let padded_gs = PaddedGameState::new_from_game_state(&gs);

    c.bench_function("Eval", |b| b.iter(|| eval(&play(mov, &padded_gs.gs).unwrap())));
}

criterion_group!(evals, fast_eval_benchmark, eval_benchmark);

fn min_max_next_move_benchmark(c: &mut Criterion) {
    let game_globals = GameGlobals::new(6, 7);
    let gs = get_random_position(0, 1, &game_globals);
    let padded_gs = PaddedGameState::new_from_game_state(&gs);
    let agent = MinMaxAgent::new_with_args(false, 0, 5, 6, 7);

    c.bench_function("Min Max Next Move", |b| b.iter(|| agent.next_move(&padded_gs.gs)));
}
criterion_group!(next_moves, min_max_next_move_benchmark);
criterion_main!(evals, next_moves);
