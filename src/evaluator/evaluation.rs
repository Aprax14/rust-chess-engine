#![warn(clippy::pedantic)]
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc::Sender;
use std::{cmp, i32};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::components::pieces::Color;
use crate::moves::moves::{Move, Scenario};

use super::static_eval::StaticEval;

impl Scenario {
    fn minimax_alpha_beta(
        &self,
        depth: i32,
        max_depth: i32,
        mut alpha: i32,
        mut beta: i32,
        depth_counter: i32,
    ) -> i32 {
        let available_moves = self.board.generate_moves_ordered(false);

        if available_moves.is_empty() {
            if self.board.position.is_in_check(Color::White) {
                return i32::MIN;
            } else if self.board.position.is_in_check(Color::Black) {
                return i32::MAX;
            } else {
                return 0;
            }
        }

        if depth <= 0 {
            return self.quiescence_search(alpha, beta, depth_counter, max_depth);
        }

        match self.board.turn {
            Color::White => {
                let mut max_eval = i32::MIN;

                for player_move in available_moves {
                    let next_scenario = Scenario::new(self.board.make_unchecked_move(player_move));
                    let inner_eval = next_scenario.minimax_alpha_beta(
                        depth - 1,
                        max_depth,
                        alpha,
                        beta,
                        depth_counter + 1,
                    );
                    if inner_eval > max_eval {
                        max_eval = inner_eval;
                    }
                    alpha = cmp::max(alpha, inner_eval);
                    if alpha >= beta {
                        break;
                    }
                }
                max_eval
            }
            Color::Black => {
                let mut min_eval = i32::MAX;

                for player_move in available_moves {
                    let next_scenario = Scenario::new(self.board.make_unchecked_move(player_move));
                    let inner_eval = next_scenario.minimax_alpha_beta(
                        depth - 1,
                        max_depth,
                        alpha,
                        beta,
                        depth_counter + 1,
                    );

                    if inner_eval < min_eval {
                        min_eval = inner_eval;
                    }

                    beta = cmp::min(beta, inner_eval);
                    if alpha >= beta {
                        break;
                    }
                }
                min_eval
            }
        }
    }

    pub fn parallel_minimax_alpha_beta(&self, depth: i32, max_depth: i32, tx: Sender<(Move, i32)>) {
        let depth_counter = 0;
        let available_moves = self.board.generate_moves_ordered(false);

        let best_eval = AtomicI32::new(match self.board.turn {
            Color::White => i32::MIN,
            Color::Black => i32::MAX,
        });
        let main_alpha = AtomicI32::new(i32::MIN);
        let main_beta = AtomicI32::new(i32::MAX);
        let stop_signal = AtomicBool::new(false);

        available_moves
            .into_par_iter()
            .for_each_with(tx.clone(), |sender, player_move| {
                let next_scenario = Scenario::new(self.board.make_unchecked_move(player_move));
                let turn = self.board.turn;

                if stop_signal.load(Ordering::Acquire) {
                    return;
                }

                let eval = next_scenario.minimax_alpha_beta(
                    depth - 1,
                    max_depth,
                    main_alpha.load(Ordering::Acquire),
                    main_beta.load(Ordering::Acquire),
                    depth_counter + 1,
                );

                match turn {
                    Color::White => {
                        best_eval.fetch_max(eval, Ordering::AcqRel);
                        main_alpha.fetch_max(eval, Ordering::AcqRel);

                        // send evaluations while elaborating
                        sender
                            .send((player_move.clone(), eval))
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
                            .send((player_move.clone(), eval))
                            .expect("failed to send to channel");

                        if main_alpha.load(Ordering::Acquire) >= main_beta.load(Ordering::Acquire) {
                            stop_signal.store(true, Ordering::Release);
                            return;
                        }
                    }
                }
            });

        drop(tx);
    }

    fn quiescence_search(
        &self,
        mut alpha: i32,
        mut beta: i32,
        depth_counter: i32,
        max_depth: i32,
    ) -> i32 {
        let static_eval = StaticEval::static_evaluate(&self.board);
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

        let available_moves = self.board.generate_moves_ordered(true);
        // At this point generate_moves should have already discarded the moves that left the king in check
        if available_moves.is_empty() {
            if self.board.position.is_in_check(Color::White) {
                return i32::MIN;
            } else if self.board.position.is_in_check(Color::Black) {
                return i32::MAX;
            } else {
                return 0;
            }
        }

        match self.board.turn {
            Color::White => {
                for player_move in available_moves {
                    let next_scenario = Scenario::new(self.board.make_unchecked_move(player_move));
                    let eval =
                        next_scenario.quiescence_search(alpha, beta, depth_counter + 1, max_depth);
                    if eval >= beta {
                        return beta;
                    }
                    if eval > alpha {
                        alpha = eval;
                    }
                }
                alpha
            }
            Color::Black => {
                for player_move in available_moves {
                    let next_scenario = Scenario::new(self.board.make_unchecked_move(player_move));
                    let eval =
                        next_scenario.quiescence_search(alpha, beta, depth_counter + 1, max_depth);
                    if eval <= alpha {
                        return alpha;
                    }
                    if eval < beta {
                        beta = eval;
                    }
                }
                beta
            }
        }
    }
}
