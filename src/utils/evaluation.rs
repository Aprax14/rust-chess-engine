use std::cmp;
use std::sync::mpsc::Sender;

use rayon::ThreadPoolBuilder;

use crate::types::moves::Move;
use crate::types::{moves::Scenario, piece::Color};

use super::static_evaluation::StaticEval;

// not working good at the moment
const MAX_DEPTH: u32 = 6;

fn minimax_alpha_beta_pv(
    scenario: &Scenario,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    depth_counter: u32,
    current_pv: &mut Vec<Move>,
) -> i32 {
    // we keep searching only for captures and stop checks if we reached the required depth.
    // depth <= 0 occours when we already reached the required level but we are keep evaluating captures.
    let only_critical = depth <= 0;
    let available_moves = scenario.generate_moves(only_critical, current_pv);

    if depth <= 0 && (available_moves.is_empty() || depth_counter >= MAX_DEPTH) {
        let static_eval = StaticEval::static_evaluate(&scenario.board);
        return static_eval.white - static_eval.black;
    }

    let mut local_pv = Vec::new();

    let next_scenarios = scenario.apply_moves(available_moves);
    if next_scenarios.is_empty() {
        if scenario.white_in_check() {
            return i32::MIN;
        } else if scenario.black_in_check() {
            return i32::MAX;
        } else {
            return 0;
        }
    }
    match scenario.board.turn {
        Color::White => {
            let mut max_eval = i32::MIN;

            for (piece_move, next_scenario) in next_scenarios {
                let mut child_pv = Vec::new();
                let inner_eval = minimax_alpha_beta_pv(
                    &next_scenario,
                    depth - 1,
                    alpha,
                    beta,
                    depth_counter + 1,
                    &mut child_pv,
                );
                if inner_eval > max_eval {
                    max_eval = inner_eval;
                    local_pv.clear();
                    local_pv.push(piece_move);
                    local_pv.extend(child_pv);
                }
                alpha = cmp::max(alpha, inner_eval);
                if alpha >= beta {
                    break;
                }
            }
            current_pv.clear();
            current_pv.extend(local_pv);
            max_eval
        }
        Color::Black => {
            let mut min_eval = i32::MAX;

            for (piece_move, next_scenario) in next_scenarios {
                let mut child_pv = Vec::new();
                let inner_eval = minimax_alpha_beta_pv(
                    &next_scenario,
                    depth - 1,
                    alpha,
                    beta,
                    depth_counter + 1,
                    &mut child_pv,
                );

                if inner_eval < min_eval {
                    min_eval = inner_eval;
                    local_pv.clear();
                    local_pv.push(piece_move);
                    local_pv.extend(child_pv);
                }

                beta = cmp::min(beta, inner_eval);
                if alpha >= beta {
                    break;
                }
            }
            current_pv.clear();
            current_pv.extend(local_pv);
            min_eval
        }
    }
}

pub fn parallel_minimax_alpha_beta_pv(
    scenario: &Scenario,
    depth: i32,
    alpha: i32,
    beta: i32,
    current_pv: Vec<Move>,
    tx: Sender<(Move, i32, Vec<Move>)>,
) {
    let depth_counter = 0;
    let available_moves = scenario.generate_moves(false, &current_pv);
    let next_scenarios = scenario.apply_moves(available_moves);

    let pool = ThreadPoolBuilder::new()
        .num_threads(16)
        .build()
        .expect("failed to initilize threadpool");

    pool.install(|| {
        for (piece_move, next_scenario) in next_scenarios {
            let mut inner_pv = current_pv.clone();
            let inner_tx = tx.clone();
            pool.spawn(move || {
                // let mut pv = inner_pv;
                let eval = minimax_alpha_beta_pv(
                    &next_scenario,
                    depth - 1,
                    alpha,
                    beta,
                    depth_counter + 1,
                    &mut inner_pv,
                );
                // send evaluations while elaborating them
                inner_tx
                    .send((piece_move, eval, inner_pv))
                    .expect("failed to send to channel");
            });
        }
    });

    drop(tx);
}
