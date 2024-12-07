use std::io;

use types::{board::Board, moves::Scenario};
use utils::evaluation;

mod moves;
mod types;
mod utils;

// 3k4/8/7p/2p1p1pP/1pPpPpP1/1P1P1P2/N7/2K5 w - - 0 1
// r1b1kbnr/pppp1ppp/2n2q2/4p3/2BPP3/5N2/PPP2PPP/RNBQK2R b KQkq - 2 4

/*
TODO:
- Consider castling move
- Consider en passant moves
- Consider central king
- Maybe personalized maps for pieces
*/

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    loop {
        let board = loop {
            tracing::info!("insert a new position in Forsyth Edwards notation:");

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
            tracing::info!("insert evaluation depth");

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

        let scenario = Scenario::from_board(board);
        tracing::info!("start minimax evaluation...");
        let eval = evaluation::parallel_minimax_alpha_beta(
            &scenario,
            depth,
            i32::MIN,
            i32::MAX,
            false,
            true,
        );
        tracing::info!("suggested move: \n{}", eval.1.board);
        tracing::info!("evaluation: {}", eval.0);
        tracing::info!("minimax evaluation finished");
    }
}
