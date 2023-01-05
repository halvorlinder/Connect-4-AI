use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use four_in_a_row::game::{Agent, Game, MinMaxAgent};
use four_in_a_row::game_logic;
use four_in_a_row::game_logic::test_utils::{get_random_position, get_random_positions};
use four_in_a_row::game_logic::{eval, fast_eval, get_legal, play, GameGlobals, PaddedGameState};

use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

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

    c.bench_function("Eval", |b| {
        b.iter(|| eval(&play(mov, &padded_gs.gs).unwrap()))
    });
}

criterion_group!(evals, fast_eval_benchmark, );

fn min_max_next_move_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("min_max_next_move_benchmark");
    let game_globals = GameGlobals::new(6, 7);
    let gs = get_random_position(0, 1, &game_globals);
    let padded_gs = PaddedGameState::new_from_game_state(&gs);

    for depth in [5]{
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            let agent = MinMaxAgent::new_with_args(false, 0, depth, 6, 7);
            b.iter(|| agent.next_move(&padded_gs.gs))
        });
    }
}
criterion_group!(next_moves, min_max_next_move_benchmark);

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn game_state_hash_benchmark(c: &mut Criterion) {
    let game_globals = GameGlobals::new(6, 7);
    // let gs = get_random_position(42, 1, &game_globals);
    let states = get_random_positions(42, 1000, &game_globals);

    c.bench_function("GameState Hash", |b| {
        b.iter(|| {
            for gs in states.iter(){
                let mut s = DefaultHasher::new();
                gs.hash(&mut s);
                s.finish();
            }
        })
    });
}

fn game_state_hash_insert_benchmark(c: &mut Criterion) {
    let game_globals = GameGlobals::new(6, 7);
    let states = get_random_positions(42, 1000, &game_globals);


    c.bench_function("GameState Hash", |b| {
        b.iter(|| {
            let mut hm = HashSet::new();
            for gs in states.iter(){
                hm.insert(gs);
            }
        })
    });
}

fn u128_hash_insert_benchmark(c: &mut Criterion) {
    let game_globals = GameGlobals::new(6, 7);
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    let hashes : Vec<u128> = (0..1000000u128).map(|_| rng.gen_range(0..u128::MAX)).collect();

    c.bench_function("GameState Hash", |b| {
        b.iter(|| {
            let mut hm = HashSet::new();
            for hash in hashes.iter(){
                hm.insert(hash);
            }
        })
    });
}

criterion_group!(hashes, game_state_hash_benchmark, game_state_hash_insert_benchmark, u128_hash_insert_benchmark);

criterion_main!(evals, next_moves, hashes);
