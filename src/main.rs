use std::{io, sync::mpsc, thread};

use types::{
    board::Board,
    moves::{Move, Scenario},
    piece::Color,
};
use utils::evaluation;

mod moves;
mod types;
mod utils;

// 3k4/8/7p/2p1p1pP/1pPpPpP1/1P1P1P2/N7/2K5 w - - 0 1
// r1b1kbnr/pppp1ppp/2n2q2/4p3/2BPP3/5N2/PPP2PPP/RNBQK2R b KQkq - 2 4

/*
TODO:
- Consider en passant moves
- Consider central king
*/

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    let mut principal_variation: Vec<Move> = Vec::new();
    loop {
        let board = loop {
            tracing::info!("Forsyth Edwards position notation:");

            let mut buffer = String::new();
            io::stdin()
                .read_line(&mut buffer)
                .expect("failed to read user input");

            let Ok(position) = Board::from_forsyth_edwards(buffer.trim()) else {
                tracing::error!("error parsing position. Please insert a valid position");
                continue;
            };

            break position;
        };

        let depth = loop {
            tracing::info!("Full evaluation Depth:");

            let mut buffer = String::new();
            io::stdin()
                .read_line(&mut buffer)
                .expect("failed to read user input");

            let Ok(depth) = buffer.trim().parse::<u8>() else {
                tracing::error!("error! please insert a valid depth");
                continue;
            };

            break depth;
        };

        let max_depth = loop {
            tracing::info!("Max evaluation Depth");

            let mut buffer = String::new();
            io::stdin()
                .read_line(&mut buffer)
                .expect("failed to read user input");

            let Ok(depth) = buffer.trim().parse::<u8>() else {
                tracing::error!("error! please insert a valid depth");
                continue;
            };

            break depth;
        };

        tracing::info!("evaluating position: \n{}", board);
        let scenario = Scenario::from_board(&board);

        tracing::info!("start minimax evaluation...");

        //--------------------------------Evaluation-------------------------------------//

        let mut best_eval = match scenario.board.turn {
            Color::White => i32::MIN,
            Color::Black => i32::MAX,
        };
        let mut pv: Vec<Move> = Vec::new();

        let (tx, rx) = mpsc::channel::<(Move, i32, Vec<Move>)>();
        let previous_scenario = scenario.clone();

        thread::spawn(move || {
            evaluation::parallel_minimax_alpha_beta_pv(
                &scenario,
                depth as i32,
                max_depth as i32,
                principal_variation.clone(),
                tx,
            )
        });

        // show the best move we have at the moment while we go on with the elaboration
        for (inner_move, eval, inner_pv) in rx.iter() {
            match previous_scenario.board.turn {
                Color::White => {
                    if eval > best_eval {
                        best_eval = eval;
                        pv = [vec![inner_move], inner_pv].into_iter().flatten().collect();
                        let new_board = previous_scenario.board.make_unchecked_move(&pv[0]);
                        tracing::info!("found new best move:\n{}", new_board);
                        tracing::info!("evaluation: {}", eval);
                    }
                }
                Color::Black => {
                    if eval < best_eval {
                        best_eval = eval;
                        pv = [vec![inner_move], inner_pv].into_iter().flatten().collect();
                        let new_board = previous_scenario.board.make_unchecked_move(&pv[0]);
                        tracing::info!("found new best move:\n{}", new_board);
                        tracing::info!("evaluation: {}", eval);
                    }
                }
            };
        }

        principal_variation = pv;

        let mut new_board = board.clone();
        tracing::info!("Path to best move: \n");
        for (n, m) in principal_variation.iter().enumerate() {
            new_board = new_board.make_unchecked_move(m);
            tracing::info!("Move N. {n} \n");
            tracing::info!("\n{}\n", new_board);
            tracing::info!("------------------------------------------------\n");
        }

        //-----------------------------------------------------------------------//
        tracing::info!("minimax evaluation finished");
    }
}
