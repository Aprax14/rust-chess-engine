use std::io::{self, BufRead, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use components::{
    board::Board,
    castle::CastleSide,
    pieces::{Color, PieceKind},
};
use moves::move_type::{Move, MoveKind, Scenario};

mod components;
mod evaluator;
mod moves;

const ENGINE_NAME: &str = "corman"; // my cats: Cornelia and Norman
const ENGINE_AUTHOR: &str = "Damiano Scarpellini";

/// Sends a UCI response line and flushes stdout immediately.
macro_rules! uci_send {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        let _ = io::stdout().flush();
    }};
}

// ---------------------------------------------------------------------------
// Square index <-> UCI notation
// ---------------------------------------------------------------------------

/// Square index (0 - 63) -> UCI square string (e.g. 3 -> e1).
fn sq_to_uci(sq: u8) -> String {
    let file = 7 - (sq % 8);
    let rank = sq / 8;

    format!("{}{}", (b'a' + file) as char, (b'1' + rank) as char)
}

/// UCI square string -> square index (e.g. "e1" → 3). Returns None on invalid input.
fn uci_to_sq(s: &str) -> Option<u8> {
    let b = s.as_bytes();
    if b.len() < 2 {
        return None;
    }
    let file = b[0].checked_sub(b'a').filter(|&f| f < 8)?;
    let rank = b[1].checked_sub(b'1').filter(|&r| r < 8)?;

    Some(rank * 8 + (7 - file))
}

// ---------------------------------------------------------------------------
// Move <-> UCI notation
// ---------------------------------------------------------------------------

/// Converts a Move to its UCI string (e.g. "e2e4", "e7e8q", "e1g1").
fn move_to_uci(m: &Move) -> String {
    match &m.action {
        MoveKind::Standard { from, to } | MoveKind::EnPassant { from, to } => {
            format!("{}{}", sq_to_uci(*from), sq_to_uci(*to))
        }
        MoveKind::Promote { from, to, to_piece } => {
            let promo = match to_piece {
                PieceKind::Queen => 'q',
                PieceKind::Rook => 'r',
                PieceKind::Bishop => 'b',
                PieceKind::Knight => 'n',
                _ => 'q',
            };
            format!("{}{}{}", sq_to_uci(*from), sq_to_uci(*to), promo)
        }
        MoveKind::Castle(side) => match (m.piece.color, side) {
            (Color::White, CastleSide::King) => "e1g1",
            (Color::White, CastleSide::Queen) => "e1c1",
            (Color::Black, CastleSide::King) => "e8g8",
            (Color::Black, CastleSide::Queen) => "e8c8",
        }
        .to_string(),
    }
}

/// Expected from/to square indices for each castling move (used when matching
/// a UCI string like "e1g1" back to a Castle move).
fn castle_squares(color: Color, side: &CastleSide) -> (u8, u8) {
    match (color, side) {
        (Color::White, CastleSide::King) => (3, 1),    // e1 -> g1
        (Color::White, CastleSide::Queen) => (3, 5),   // e1 -> c1
        (Color::Black, CastleSide::King) => (59, 57),  // e8 -> g8
        (Color::Black, CastleSide::Queen) => (59, 61), // e8 -> c8
    }
}

/// Parses a UCI move string into a legal Move for the given board position.
/// Returns None if the move string is malformed or the move is not legal.
fn parse_uci_move(board: &Board, uci: &str) -> Option<Move> {
    if uci.len() < 4 {
        return None;
    }
    let from = uci_to_sq(&uci[0..2])?;
    let to = uci_to_sq(&uci[2..4])?;
    let promo = uci.as_bytes().get(4).and_then(|&b| match b {
        b'q' => Some(PieceKind::Queen),
        b'r' => Some(PieceKind::Rook),
        b'b' => Some(PieceKind::Bishop),
        b'n' => Some(PieceKind::Knight),
        _ => None,
    });

    let moves = board.generate_moves(false);
    moves.list[..moves.len()]
        .iter()
        .find(|rm| {
            let m = &rm.piece_move;
            match &m.action {
                MoveKind::Standard { from: f, to: t } => *f == from && *t == to && promo.is_none(),
                MoveKind::EnPassant { from: f, to: t } => *f == from && *t == to,
                MoveKind::Promote {
                    from: f,
                    to: t,
                    to_piece,
                } => *f == from && *t == to && promo.is_none_or(|p| p == *to_piece),
                MoveKind::Castle(side) => {
                    let (cf, ct) = castle_squares(m.piece.color, side);
                    from == cf && to == ct
                }
            }
        })
        .map(|rm| rm.piece_move)
}

// ---------------------------------------------------------------------------
// Search helpers
// ---------------------------------------------------------------------------

/// Runs the engine search at the given depth and returns the best (Move, score)
/// pair, or None if there are no legal moves (checkmate / stalemate).
fn search_at_depth(board: &Board, depth: i32, max_depth: i32) -> Option<(Move, i32)> {
    let scenario = Scenario::new(board.clone());
    let (tx, rx) = mpsc::channel::<(Move, i32)>();
    let is_white = board.turn == Color::White;

    thread::spawn(move || {
        scenario.parallel_minimax_alpha_beta(depth, max_depth, tx);
    });

    let mut best: Option<(Move, i32)> = None;
    for (m, eval) in rx {
        let better = match best {
            None => true,
            Some((_, prev)) => {
                if is_white {
                    eval > prev
                } else {
                    eval < prev
                }
            }
        };
        if better {
            best = Some((m, eval));
        }
    }

    best
}

/// Iterative-deepening search within `budget`.
/// Starts from depth 1, increases depth as long as time allows, and returns
/// the best move found in the last completed iteration.
///
/// The engine has no mid-search stop mechanism, so each iteration runs to
/// completion. The heuristic is: skip the next iteration if we have already
/// used more than half the budget (the next depth usually takes ~5× longer).
fn iterative_deepening(board: &Board, budget: Duration) -> Option<Move> {
    let start = Instant::now();
    let mut best_move: Option<Move> = None;

    for depth in 1i32..=20 {
        let elapsed = start.elapsed();
        if elapsed >= budget {
            break;
        }
        // If over half the budget is gone, the next depth will likely exceed it.
        if best_move.is_some() && elapsed.as_millis() * 2 > budget.as_millis() {
            break;
        }

        match search_at_depth(board, depth, depth + 3) {
            Some((m, eval)) => {
                // Engine uses 1000 per pawn; UCI expects centipawns (100/pawn).
                let cp = eval / 10;
                uci_send!(
                    "info depth {} score cp {} time {}",
                    depth,
                    cp,
                    start.elapsed().as_millis()
                );
                best_move = Some(m);
            }
            None => break, // no legal moves
        }
    }

    best_move
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

fn handle_position(tokens: &[&str], current_board: &mut Board) {
    if tokens.len() < 2 {
        return;
    }

    let moves_idx = tokens.iter().position(|&t| t == "moves");

    let board_result = match tokens[1] {
        "startpos" => Ok(Board::new_game()),
        "fen" => {
            let fen_end = moves_idx.unwrap_or(tokens.len());
            if fen_end <= 2 {
                return;
            }
            Board::from_forsyth_edwards(&tokens[2..fen_end].join(" "))
        }
        _ => return,
    };

    let mut board = match board_result {
        Ok(b) => b,
        Err(_) => return,
    };

    if let Some(mi) = moves_idx {
        for uci_move in &tokens[mi + 1..] {
            match parse_uci_move(&board, uci_move) {
                Some(m) => board = board.make_unchecked_move(&m),
                None => break, // malformed move list, stop applying
            }
        }
    }

    *current_board = board;
}

fn handle_go(board: &Board, tokens: &[&str]) {
    let mut fixed_depth: Option<i32> = None;
    let mut movetime_ms: Option<u64> = None;
    let mut wtime: Option<u64> = None; // white clock time left
    let mut btime: Option<u64> = None; // black clock time left
    let mut winc: Option<u64> = None; // white clock increment
    let mut binc: Option<u64> = None; // black clock increment
    let mut infinite = false;

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "depth" => {
                fixed_depth = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "movetime" => {
                movetime_ms = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "wtime" => {
                wtime = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "btime" => {
                btime = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "winc" => {
                winc = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "binc" => {
                binc = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "infinite" => {
                infinite = true;
            }
            _ => {}
        }
        i += 1;
    }

    let best_move = if let Some(d) = fixed_depth {
        // Fixed-depth search: run once, no time management.
        search_at_depth(board, d, d + 3).map(|(m, eval)| {
            let cp = eval / 10;
            uci_send!("info depth {} score cp {}", d, cp);
            m
        })
    } else {
        let budget_ms = if infinite {
            // No stop support yet: cap at 30 s so we don't hang forever.
            30_000
        } else if let Some(mt) = movetime_ms {
            mt
        } else {
            // Time-control mode: use ~1/30 of remaining time + half increment.
            let my_time = match board.turn {
                Color::White => wtime.unwrap_or(30_000),
                Color::Black => btime.unwrap_or(30_000),
            };
            let my_inc = match board.turn {
                Color::White => winc.unwrap_or(0),
                Color::Black => binc.unwrap_or(0),
            };
            (my_time / 30).max(100) + my_inc / 2
        };

        iterative_deepening(board, Duration::from_millis(budget_ms))
    };

    match best_move {
        Some(m) => uci_send!("bestmove {}", move_to_uci(&m)),
        None => uci_send!("bestmove 0000"), // no legal moves (checkmate / stalemate)
    }
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

fn main() {
    let stdin = io::stdin();
    let mut current_board = Board::new_game();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        match tokens[0] {
            "uci" => {
                uci_send!("id name {}", ENGINE_NAME);
                uci_send!("id author {}", ENGINE_AUTHOR);
                uci_send!("uciok");
            }
            "isready" => {
                uci_send!("readyok");
            }
            "ucinewgame" => {
                current_board = Board::new_game();
            }
            "position" => {
                handle_position(&tokens, &mut current_board);
            }
            "go" => {
                handle_go(&current_board, &tokens[1..]);
            }
            // "stop" is not supported yet: the search runs to completion.
            // Ignore it silently so the GUI does not hang.
            "stop" => {}
            "quit" => break,
            _ => {}
        }
    }
}
