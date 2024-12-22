#![warn(clippy::pedantic)]
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc::Sender;
use std::{cmp, i32};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::types::moves::Move;
use crate::types::{moves::Scenario, piece::Color};

use super::static_evaluation::StaticEval;

fn minimax_alpha_beta_pv(
    scenario: &Scenario,
    depth: i32,
    max_depth: i32,
    mut alpha: i32,
    mut beta: i32,
    depth_counter: i32,
    current_pv: &mut Vec<Move>,
) -> i32 {
    let available_moves = scenario.generate_moves(false, current_pv);

    if available_moves.is_empty() {
        if scenario.white_in_check() {
            return i32::MIN;
        } else if scenario.black_in_check() {
            return i32::MAX;
        } else {
            return 0;
        }
    }

    if depth <= 0 {
        return quiescence_search(scenario, alpha, beta, depth_counter, max_depth, current_pv);
    }

    let mut local_pv = Vec::new();

    match scenario.board.turn {
        Color::White => {
            let mut max_eval = i32::MIN;

            for player_move in available_moves {
                if let Some(next_scenario) = scenario.apply_move(&player_move) {
                    let mut child_pv = Vec::new();
                    let inner_eval = minimax_alpha_beta_pv(
                        &next_scenario,
                        depth - 1,
                        max_depth,
                        alpha,
                        beta,
                        depth_counter + 1,
                        &mut child_pv,
                    );
                    if inner_eval > max_eval {
                        max_eval = inner_eval;
                        local_pv.clear();
                        local_pv.push(player_move.clone());
                        local_pv.extend(child_pv);
                    }
                    alpha = cmp::max(alpha, inner_eval);
                    if alpha >= beta {
                        break;
                    }
                }
            }
            current_pv.clear();
            current_pv.extend(local_pv);
            max_eval
        }
        Color::Black => {
            let mut min_eval = i32::MAX;

            for player_move in available_moves {
                if let Some(next_scenario) = scenario.apply_move(&player_move) {
                    let mut child_pv = Vec::new();
                    let inner_eval = minimax_alpha_beta_pv(
                        &next_scenario,
                        depth - 1,
                        max_depth,
                        alpha,
                        beta,
                        depth_counter + 1,
                        &mut child_pv,
                    );

                    if inner_eval < min_eval {
                        min_eval = inner_eval;
                        local_pv.clear();
                        local_pv.push(player_move.clone());
                        local_pv.extend(child_pv);
                    }

                    beta = cmp::min(beta, inner_eval);
                    if alpha >= beta {
                        break;
                    }
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
    max_depth: i32,
    current_pv: Vec<Move>,
    tx: Sender<(Move, i32, Vec<Move>)>,
) {
    let depth_counter = 0;
    let available_moves = scenario.generate_moves(false, &current_pv);

    let best_eval = AtomicI32::new(match scenario.board.turn {
        Color::White => i32::MIN,
        Color::Black => i32::MAX,
    });
    let main_alpha = AtomicI32::new(i32::MIN);
    let main_beta = AtomicI32::new(i32::MAX);
    let stop_signal = AtomicBool::new(false);

    available_moves
        .into_par_iter()
        .for_each_with(tx.clone(), |sender, player_move| {
            if let Some(next_scenario) = scenario.apply_move(&player_move) {
                let mut pv = current_pv.clone();
                let turn = scenario.board.turn;

                if stop_signal.load(Ordering::Acquire) {
                    return;
                }

                let eval = minimax_alpha_beta_pv(
                    &next_scenario,
                    depth - 1,
                    max_depth,
                    main_alpha.load(Ordering::Acquire),
                    main_beta.load(Ordering::Acquire),
                    depth_counter + 1,
                    &mut pv,
                );

                match turn {
                    Color::White => {
                        best_eval.fetch_max(eval, Ordering::AcqRel);
                        main_alpha.fetch_max(eval, Ordering::AcqRel);

                        // send evaluations while elaborating
                        sender
                            .send((player_move.clone(), eval, pv))
                            .expect("failed to send to channel");

                        if main_alpha.load(Ordering::Acquire) >= main_beta.load(Ordering::Acquire) {
                            stop_signal.store(true, Ordering::Release);
                            return;
                        }
                    }
                    Color::Black => {
                        best_eval.fetch_min(eval, Ordering::AcqRel);
                        main_beta.fetch_min(eval, Ordering::AcqRel);

                        // send evaluations while elaborating
                        sender
                            .send((player_move.clone(), eval, pv))
                            .expect("failed to send to channel");

                        if main_alpha.load(Ordering::Acquire) >= main_beta.load(Ordering::Acquire) {
                            stop_signal.store(true, Ordering::Release);
                            return;
                        }
                    }
                }
            }
        });

    drop(tx);
}

// non sto aggiornando la PV
fn quiescence_search(
    scenario: &Scenario,
    mut alpha: i32,
    mut beta: i32,
    depth_counter: i32,
    max_depth: i32,
    current_pv: &mut Vec<Move>,
) -> i32 {
    let static_eval = StaticEval::static_evaluate(&scenario.board);
    let current_eval = static_eval.white - static_eval.black;

    if current_eval >= beta {
        return beta;
    }

    if depth_counter >= max_depth {
        return current_eval;
    }

    if current_eval > alpha {
        alpha = current_eval;
    }

    let available_moves = scenario.generate_moves(true, &Vec::new());

    if available_moves.is_empty() {
        if scenario.white_in_check() {
            return i32::MIN;
        } else if scenario.black_in_check() {
            return i32::MAX;
        } else {
            return 0;
        }
    }

    let mut local_pv = Vec::new();

    match scenario.board.turn {
        Color::White => {
            for player_move in available_moves {
                if let Some(next_scenario) = scenario.apply_move(&player_move) {
                    let mut child_pv = Vec::new();
                    let eval = quiescence_search(
                        &next_scenario,
                        alpha,
                        beta,
                        depth_counter + 1,
                        max_depth,
                        &mut child_pv,
                    );
                    if eval >= beta {
                        return beta;
                    }
                    if eval > alpha {
                        alpha = eval;
                        local_pv.clear();
                        local_pv.push(player_move.clone());
                        local_pv.extend(child_pv);
                    }
                } else {
                    continue;
                }
            }
            current_pv.clear();
            current_pv.extend(local_pv);
            alpha
        }
        Color::Black => {
            for player_move in available_moves {
                if let Some(next_scenario) = scenario.apply_move(&player_move) {
                    let mut child_pv = Vec::new();
                    let eval = quiescence_search(
                        &next_scenario,
                        alpha,
                        beta,
                        depth_counter + 1,
                        max_depth,
                        &mut child_pv,
                    );
                    if eval <= alpha {
                        return alpha;
                    }
                    if eval < beta {
                        beta = eval;
                        local_pv.clear();
                        local_pv.push(player_move.clone());
                        local_pv.extend(child_pv);
                    }
                } else {
                    continue;
                }
            }
            current_pv.clear();
            current_pv.extend(local_pv);
            beta
        }
    }
}
