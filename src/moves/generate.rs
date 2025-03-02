use std::{mem, ptr};

use crate::{
    components::{board::Board, castle, pieces::PieceKind},
    evaluator,
};
use strum::IntoEnumIterator;

use super::moves::{Move, MoveKind};

#[derive(Debug, Clone, Copy)]
pub struct RatedMove {
    pub piece_move: Move,
    pub rating: i32,
}

impl RatedMove {
    pub fn new(piece_move: Move, rating: i32) -> Self {
        RatedMove { piece_move, rating }
    }
}

impl Default for RatedMove {
    fn default() -> Self {
        Self {
            piece_move: Move {
                piece: 'K'.try_into().expect("hardcoded piece should not fail"),
                action: MoveKind::Standard { from: 0, to: 0 },
            },
            rating: i32::MIN,
        }
    }
}

pub struct Moves {
    pub list: [RatedMove; 255],
    pub len: u8,
}

impl Moves {
    fn new() -> Self {
        Moves {
            list: [RatedMove::default(); 255],
            len: 0,
        }
    }

    fn push(&mut self, current_move: Move, rating: i32) {
        self.list[self.len as usize] = RatedMove::new(current_move, rating);
        self.len += 1;
    }

    /// puts the best rated next move, left from the required index to the moves len, at the required index.
    fn sort_one(&mut self, index: usize) {
        let mut best_idx = index;
        let mut best_rating = self.list[index].rating;

        for i in index + 1..self.len as usize {
            if self.list[i].rating > best_rating {
                best_idx = i;
                best_rating = self.list[i].rating;
            }
        }

        if best_idx != index {
            self.list.swap(index, best_idx);
        }
    }

    /// Takes the best rated move left from the required index to the moves len.
    pub fn take(&mut self, index: usize) -> Move {
        self.sort_one(index);
        self.list[index].piece_move
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }
}

impl Board {
    /// returns all the possible legal moves order by the rating given to them.
    /// the rating is given according to MVV LVA:
    /// Most Valuable Victim Less Valuable Attacker.
    ///
    /// When only_critical is true only captures and stop-checks get generated.
    /// Discards the moves that leaves the moving side king in check (illegal).
    pub fn generate_moves(&self, only_critical: bool) -> Moves {
        let mut moves = Moves::new();

        for (piece, bitboard) in self.position.into_iter() {
            if piece.color != self.turn {
                continue;
            }

            for piece_position in bitboard.single_squares() {
                let available_moves = self.position.available_moves(*piece, piece_position);

                for to_square in available_moves.single_squares() {
                    let current_move = Move {
                        piece: *piece,
                        action: MoveKind::Standard {
                            from: piece_position,
                            to: to_square,
                        },
                    };

                    let next_position = self.position.inner_make_unchecked_move(&current_move);
                    if next_position.is_in_check(current_move.piece.color) {
                        // the move the player made left the king in check -> not valid
                        continue;
                    }

                    // save promotions only if it's not required to generate only critical moves.
                    // only_critical == true => save only captures and stop-checks
                    if current_move.is_promotion() && !only_critical {
                        for piece_kind in PieceKind::iter() {
                            if piece_kind == PieceKind::Pawn || piece_kind == PieceKind::King {
                                continue;
                            }
                            let promotion = Move {
                                piece: *piece,
                                action: MoveKind::Promote {
                                    from: piece_position,
                                    to: to_square,
                                    to_piece: piece_kind,
                                },
                            };
                            let eval = evaluator::utils::move_score_with_mvv_lva(
                                &promotion,
                                &self.position,
                            );
                            moves.push(promotion, eval);
                        }
                    } else if current_move.is_capture(&self.position)
                        || self.position.is_in_check(self.turn)
                    {
                        // if i'm there it means the move is a capture or the player is in check.
                        // if the player is in check, the move that reached this part is a move that stops the check
                        // or it would have been discarded from the condition at line 39.
                        let eval = evaluator::utils::move_score_with_mvv_lva(
                            &current_move,
                            &self.position,
                        );
                        moves.push(current_move, eval);
                    } else if !only_critical {
                        // if i'm here it means the move is not a promotion, a capture, or a stop-check.
                        // so i add it to the moves Vec only if only_critical is not required
                        let eval = evaluator::utils::move_score_with_mvv_lva(
                            &current_move,
                            &self.position,
                        );
                        moves.push(current_move, eval);
                    }
                }
            }
        }

        // generate castling moves only if the player is not in check and only_critical is not required
        if !self.position.is_in_check(self.turn) && !only_critical {
            let castling_moves = castle::available_castling_moves(
                self,
                self.white_can_castle,
                self.black_can_castle,
            );

            if let Some(m) = castling_moves.0 {
                let eval = evaluator::utils::move_score_with_mvv_lva(&m, &self.position);
                moves.push(m, eval);
            }

            if let Some(m) = castling_moves.1 {
                let eval = evaluator::utils::move_score_with_mvv_lva(&m, &self.position);
                moves.push(m, eval);
            }
        }

        moves
    }
}
