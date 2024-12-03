use std::cmp;

use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use crate::types::{moves::Scenario, piece::Color};

use super::static_evaluation::StaticEval;

fn minimax_alpha_beta(
    scenario: &Scenario,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    only_captures: bool,
    captures_first: bool,
) -> i32 {
    if scenario.white_lost() {
        return i32::MIN;
    } else if scenario.black_lost() {
        return i32::MAX;
    } else if depth == 0 {
        let static_eval = StaticEval::static_evaluate(&scenario.board);
        return static_eval.white - static_eval.black;
    }

    let available_moves = scenario.generate_moves(only_captures, captures_first);

    let next_scenarios = scenario.apply_moves(available_moves);
    match scenario.board.turn {
        Color::White => {
            let mut max_eval = i32::MIN;

            for next_scenario in next_scenarios {
                let inner_eval = minimax_alpha_beta(
                    &next_scenario,
                    depth - 1,
                    alpha,
                    beta,
                    only_captures,
                    captures_first,
                );
                max_eval = cmp::max(max_eval, inner_eval);
                alpha = cmp::max(alpha, inner_eval);
                if alpha >= beta {
                    break;
                }
            }
            max_eval
        }
        Color::Black => {
            let mut min_eval = i32::MAX;

            for next_scenario in next_scenarios {
                let inner_eval = minimax_alpha_beta(
                    &next_scenario,
                    depth - 1,
                    alpha,
                    beta,
                    only_captures,
                    captures_first,
                );
                min_eval = cmp::min(min_eval, inner_eval);
                beta = cmp::min(beta, inner_eval);
                if alpha >= beta {
                    break;
                }
            }
            min_eval
        }
    }
}

pub fn parallel_minimax_alpha_beta(
    scenario: &Scenario,
    depth: u8,
    alpha: i32,
    beta: i32,
    only_captures: bool,
    captures_first: bool,
) -> (i32, Scenario) {
    let available_moves = scenario.generate_moves(only_captures, captures_first);
    let next_scenarios = scenario.apply_moves(available_moves);

    let evaluation_for_scenario = next_scenarios.into_par_iter().map(|next_scenario| {
        (
            minimax_alpha_beta(
                &next_scenario,
                depth - 1,
                alpha,
                beta,
                only_captures,
                captures_first,
            ),
            next_scenario,
        )
    });

    match scenario.board.turn {
        Color::White => evaluation_for_scenario
            .max_by_key(|(eval, _)| *eval)
            .unwrap(),
        Color::Black => evaluation_for_scenario
            .min_by_key(|(eval, _)| *eval)
            .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{types::board::Board, utils::evaluation};

    /// 30.79 seconds on my pc
    #[test]
    fn moves_unordered() {
        let board = Board::from_forsyth_edwards(
            "r1b1kbnr/pppp1ppp/2n2q2/4p3/2BPP3/5N2/PPP2PPP/RNBQK2R b KQkq - 2 4",
        )
        .unwrap();
        let scenario = Scenario::from_board(board);
        let eval =
            evaluation::parallel_minimax_alpha_beta(&scenario, 6, i32::MIN, i32::MAX, false, false);
        tracing::info!("suggested move: \n{}", eval.1.board);
    }

    /// 1.89 seconds on my pc
    #[test]
    fn captures_first() {
        let board = Board::from_forsyth_edwards(
            "r1b1kbnr/pppp1ppp/2n2q2/4p3/2BPP3/5N2/PPP2PPP/RNBQK2R b KQkq - 2 4",
        )
        .unwrap();
        let scenario = Scenario::from_board(board);
        let eval =
            evaluation::parallel_minimax_alpha_beta(&scenario, 6, i32::MIN, i32::MAX, false, true);
        tracing::info!("suggested move: \n{}", eval.1.board);
    }
}
