use std::cmp;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::mpsc::Sender;

use rayon::{iter::ParallelIterator, prelude::*};

use crate::components::pieces::Color;
use crate::moves::generate::RatedMove;
use crate::moves::move_type::{Move, Scenario};

use super::static_eval::StaticEval;
use super::transposition::{Bound, TranspositionTable};

/// Depth reduction used for null move pruning.
const NULL_MOVE_R: i32 = 2;

/// How many additional plies the quiescence search explores beyond the main horizon.
const QUIESCENCE_DEPTH: i32 = 4;

impl Scenario {
    pub fn minimax_alpha_beta(
        &mut self,
        depth: i32,
        mut alpha: i32,
        mut beta: i32,
        tt: &TranspositionTable,
        allow_null_move: bool,
    ) -> i32 {
        // Probe the transposition table. An exact hit lets us return immediately;
        // a bound hit narrows the alpha-beta window and may still cause a cutoff.
        if let Some(result) = tt.probe(self.board.hash, depth) {
            match result.bound {
                Bound::Exact => return result.score,
                Bound::Lower => alpha = alpha.max(result.score),
                Bound::Upper => beta = beta.min(result.score),
            }
            if alpha >= beta {
                return result.score;
            }
        }

        let available_moves = self.board.generate_moves(false);

        if available_moves.is_empty() {
            let score = if self.board.position.is_in_check(Color::White) {
                i32::MIN
            } else if self.board.position.is_in_check(Color::Black) {
                i32::MAX
            } else {
                0
            };
            // Terminal nodes are exact at any depth.
            tt.store(self.board.hash, i32::MAX, score, Bound::Exact);
            return score;
        }

        if depth <= 0 {
            return self.quiescence_search(alpha, beta, QUIESCENCE_DEPTH);
        }

        // Null move pruning: temporarily pass the turn. If the resulting position
        // (one free move for the opponent) still exceeds beta, the branch can be pruned.
        // Disabled when in check or in pawn-only positions (zugzwang risk).
        if allow_null_move && depth > NULL_MOVE_R {
            let in_check = self.board.position.is_in_check(self.board.turn);

            if !in_check && self.board.has_non_pawn_pieces() {
                let null_undo = self.board.make_null_move_mut();
                let null_eval = self.minimax_alpha_beta(
                    depth - 1 - NULL_MOVE_R,
                    alpha,
                    beta,
                    tt,
                    false, // no consecutive null moves
                );
                self.board.unmake_null_move(null_undo);

                match self.board.turn {
                    Color::White => {
                        if null_eval >= beta {
                            tt.store(self.board.hash, depth, beta, Bound::Lower);
                            return beta;
                        }
                    }
                    Color::Black => {
                        if null_eval <= alpha {
                            tt.store(self.board.hash, depth, alpha, Bound::Upper);
                            return alpha;
                        }
                    }
                }
            }
        }

        match self.board.turn {
            Color::White => {
                let mut max_eval = i32::MIN;
                let mut broke_early = false;

                for i in 0..available_moves.len() {
                    let player_move = available_moves.get(i);
                    let undo = self.board.make_move(&player_move);
                    let inner_eval = self.minimax_alpha_beta(depth - 1, alpha, beta, tt, true);
                    self.board.unmake_move(&player_move, undo);

                    if inner_eval > max_eval {
                        max_eval = inner_eval;
                    }
                    alpha = cmp::max(alpha, inner_eval);
                    if alpha >= beta {
                        broke_early = true;
                        break;
                    }
                }

                // Beta cutoff → lower bound (real score may be even higher).
                // All moves explored → exact value.
                let bound = if broke_early {
                    Bound::Lower
                } else {
                    Bound::Exact
                };
                tt.store(self.board.hash, depth, max_eval, bound);
                max_eval
            }
            Color::Black => {
                let mut min_eval = i32::MAX;
                let mut broke_early = false;

                for i in 0..available_moves.len() {
                    let player_move = available_moves.get(i);
                    let undo = self.board.make_move(&player_move);
                    let inner_eval = self.minimax_alpha_beta(depth - 1, alpha, beta, tt, true);
                    self.board.unmake_move(&player_move, undo);

                    if inner_eval < min_eval {
                        min_eval = inner_eval;
                    }

                    beta = cmp::min(beta, inner_eval);
                    if alpha >= beta {
                        broke_early = true;
                        break;
                    }
                }

                // Alpha cutoff → upper bound (real score may be even lower).
                // All moves explored → exact value.
                let bound = if broke_early {
                    Bound::Upper
                } else {
                    Bound::Exact
                };
                tt.store(self.board.hash, depth, min_eval, bound);
                min_eval
            }
        }
    }

    pub fn parallel_minimax_alpha_beta(&self, depth: i32, tx: Sender<(Move, i32)>) {
        let available_moves = self.board.generate_moves(false);

        let best_eval = AtomicI32::new(match self.board.turn {
            Color::White => i32::MIN,
            Color::Black => i32::MAX,
        });
        let main_alpha = AtomicI32::new(i32::MIN);
        let main_beta = AtomicI32::new(i32::MAX);
        let stop_signal = AtomicBool::new(false);

        // Single shared TT for all threads. The lockless implementation handles
        // concurrent reads and writes safely via the XOR integrity check.
        let tt = TranspositionTable::new();

        available_moves.list[..available_moves.len()]
            .par_iter()
            .for_each_with(
                tx.clone(),
                |sender,
                 RatedMove {
                     piece_move: player_move,
                     rating: _,
                 }| {
                    let turn = self.board.turn;

                    if stop_signal.load(Ordering::Acquire) {
                        return;
                    }

                    // Clone the board once per for thread isolation.
                    // All deeper recursive calls use make/unmake - no further clones.
                    let mut scenario = Scenario::new(self.board.clone());
                    let _undo = scenario.board.make_move(player_move);

                    let eval = scenario.minimax_alpha_beta(
                        depth - 1,
                        main_alpha.load(Ordering::Acquire),
                        main_beta.load(Ordering::Acquire),
                        &tt,
                        true,
                    );

                    match turn {
                        Color::White => {
                            best_eval.fetch_max(eval, Ordering::AcqRel);
                            main_alpha.fetch_max(eval, Ordering::AcqRel);

                            // send evaluations while elaborating
                            sender
                                .send((*player_move, eval))
                                .expect("failed to send to channel");

                            if main_alpha.load(Ordering::Acquire)
                                >= main_beta.load(Ordering::Acquire)
                            {
                                stop_signal.store(true, Ordering::Release);
                            }
                        }
                        Color::Black => {
                            best_eval.fetch_min(eval, Ordering::AcqRel);
                            main_beta.fetch_min(eval, Ordering::AcqRel);

                            // send evaluations while elaborating
                            sender
                                .send((*player_move, eval))
                                .expect("failed to send to channel");

                            if main_alpha.load(Ordering::Acquire)
                                >= main_beta.load(Ordering::Acquire)
                            {
                                stop_signal.store(true, Ordering::Release);
                            }
                        }
                    }
                },
            );

        drop(tx);
    }

    fn quiescence_search(&mut self, mut alpha: i32, mut beta: i32, qdepth: i32) -> i32 {
        let static_eval = StaticEval::static_evaluate(&self.board);
        let current_eval = static_eval.white - static_eval.black;

        match self.board.turn {
            Color::White => {
                if current_eval >= beta {
                    return beta;
                }
                if current_eval > alpha {
                    alpha = current_eval;
                }
            }
            Color::Black => {
                if current_eval <= alpha {
                    return alpha;
                }
                if current_eval < beta {
                    beta = current_eval;
                }
            }
        }

        if qdepth <= 0 {
            return current_eval;
        }

        let available_moves = self.board.generate_moves(true);
        if available_moves.is_empty() {
            if self.board.position.is_in_check(self.board.turn) {
                return match self.board.turn {
                    Color::White => i32::MIN,
                    Color::Black => i32::MAX,
                };
            }
            // No captures available and not in check: return the standing pat score.
            return match self.board.turn {
                Color::White => alpha,
                Color::Black => beta,
            };
        }

        match self.board.turn {
            Color::White => {
                for i in 0..available_moves.len() {
                    let player_move = available_moves.get(i);
                    let undo = self.board.make_move(&player_move);
                    let eval = self.quiescence_search(alpha, beta, qdepth - 1);
                    self.board.unmake_move(&player_move, undo);
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
                for i in 0..available_moves.len() {
                    let player_move = available_moves.get(i);
                    let undo = self.board.make_move(&player_move);
                    let eval = self.quiescence_search(alpha, beta, qdepth - 1);
                    self.board.unmake_move(&player_move, undo);
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
