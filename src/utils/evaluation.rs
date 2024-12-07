use std::cmp;

use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use crate::types::moves::Move;
use crate::types::{moves::Scenario, piece::Color};

use super::static_evaluation::StaticEval;

fn minimax_alpha_beta_pv(
    scenario: &Scenario,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    only_critical: bool,
    current_pv: &mut Vec<Move>,
) -> i32 {
    if depth == 0 {
        let static_eval = StaticEval::static_evaluate(&scenario.board);
        return static_eval.white - static_eval.black;
    }

    let mut local_pv = Vec::new();
    let available_moves = scenario.generate_moves(only_critical, current_pv);

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
                    only_critical,
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
                    only_critical,
                    &mut child_pv,
                );

                if inner_eval < min_eval {
                    min_eval = inner_eval;
                    local_pv.clear();
                    local_pv.push(piece_move);
                    local_pv.extend(child_pv);
                }

                min_eval = cmp::min(min_eval, inner_eval);
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
    depth: u8,
    alpha: i32,
    beta: i32,
    only_critical: bool,
    current_pv: Vec<Move>,
) -> (i32, Vec<Move>) {
    let available_moves = scenario.generate_moves(only_critical, &current_pv);
    let next_scenarios = scenario.apply_moves(available_moves);

    // for each move we get the branch evaluation and his principal variation
    let evaluation_for_scenario =
        next_scenarios
            .into_par_iter()
            .map(|(piece_move, next_scenario)| {
                let mut pv = current_pv.clone();
                let eval = minimax_alpha_beta_pv(
                    &next_scenario,
                    depth - 1,
                    alpha,
                    beta,
                    only_critical,
                    &mut pv,
                );

                (piece_move, eval, pv)
            });

    // filter the previous result to get only the one associated to the best move
    let (best_move, eval, mut pv) = match scenario.board.turn {
        Color::White => evaluation_for_scenario
            .max_by_key(|(_, eval, _)| *eval)
            .unwrap(),
        Color::Black => evaluation_for_scenario
            .min_by_key(|(_, eval, _)| *eval)
            .unwrap(),
    };

    // update pv
    pv.insert(0, best_move);

    // return the evaluation and the principal variation.
    // the first element of the PV is the move the player should make.
    (eval, pv)
}
