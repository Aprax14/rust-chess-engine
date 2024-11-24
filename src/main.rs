use std::env;

use anyhow::Context;
use types::{board::Board, moves::Scenario};
use utils::evaluation;

mod moves;
mod types;
mod utils;

// 3k4/8/7p/2p1p1pP/1pPpPpP1/1P1P1P2/N7/2K5 w - - 0 1
// r1b1kbnr/pppp1ppp/2n2q2/4p3/2BPP3/5N2/PPP2PPP/RNBQK2R b KQkq - 2 4

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let args = env::args();
    let position = utils::position_from_args(args);

    let board = Board::from_forsyth_edwards(
        &position
    ).context("failed to convert arguments to a valid chess position")?;

    tracing::info!("evaluating position: \n{}", board);

    let scenario = Scenario::from_board(board);
    tracing::info!("start minimax evaluation...");
    let eval = evaluation::parallel_minimax_alpha_beta(&scenario, 6, i32::MIN, i32::MAX, false);
    tracing::info!("suggested move: \n{}", eval.1.board);
    tracing::info!("evaluation: {}", eval.0);
    tracing::info!("minimax evaluation finished");

    Ok(())
}
