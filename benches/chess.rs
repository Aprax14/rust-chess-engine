use corman::{
    components::board::Board,
    evaluator::{static_eval::StaticEval, transposition::TranspositionTable},
    moves::{magic, move_type::Scenario},
};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::sync::Once;
use std::time::Duration;

static INIT: Once = Once::new();

fn init_magic() {
    INIT.call_once(|| magic::init());
}

// A mix of positions to benchmark against.
const POSITIONS: &[(&str, &str)] = &[
    (
        "start",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    (
        "mid_game",
        "r1bqk2r/pp2bppp/2n1pn2/3p4/3P4/2NBPN2/PPP2PPP/R1BQK2R w KQkq - 0 8",
    ),
    ("endgame", "8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P2/8/1K6 w - - 0 1"),
    (
        "tactics",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ),
];

fn bench_move_generation(c: &mut Criterion) {
    init_magic();
    let mut group = c.benchmark_group("move_generation");
    for (name, fen) in POSITIONS {
        let board = Board::from_forsyth_edwards(fen).unwrap();
        group.bench_function(*name, |b| b.iter(|| black_box(board.generate_moves(false))));
    }
    group.finish();
}

fn bench_static_eval(c: &mut Criterion) {
    init_magic();
    let mut group = c.benchmark_group("static_eval");
    for (name, fen) in POSITIONS {
        let board = Board::from_forsyth_edwards(fen).unwrap();
        group.bench_function(*name, |b| {
            b.iter(|| black_box(StaticEval::static_evaluate(&board)))
        });
    }
    group.finish();
}

fn bench_search(c: &mut Criterion) {
    init_magic();
    let mut group = c.benchmark_group("search_depth_4");
    // Search is expensive: keep sample count low and give a longer time window.
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(60));
    for (name, fen) in POSITIONS {
        let board = Board::from_forsyth_edwards(fen).unwrap();
        group.bench_function(*name, |b| {
            b.iter(|| {
                let mut scenario = Scenario::new(board.clone());
                let tt = TranspositionTable::new();
                black_box(scenario.minimax_alpha_beta(4, i32::MIN + 1, i32::MAX - 1, &tt, true))
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_move_generation,
    bench_static_eval,
    bench_search
);

criterion_main!(benches);
