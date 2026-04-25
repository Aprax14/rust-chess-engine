# corman

A chess engine written in Rust, built for fun in my free time.

## How it works

The engine uses a bitboard-based board representation and searches with alpha-beta minimax. A few things it does:

- **Move ordering** via MVV-LVA to get better alpha-beta cutoffs
- **Quiescence search** to avoid the horizon effect on captures
- **Null move pruning** to speed up the search
- **Transposition table** with Zobrist hashing to avoid re-evaluating the same position
- **Parallel search** at the root using rayon
- Static evaluation based on material, piece-square tables, and attacked squares

It uses UCI protocol, so you can plug it into any UCI-compatible GUI or lichess-bot.

## Building

```bash
cargo build --release
```

## Benchmarks

`bench.sh` runs the benchmarks, compares against the previous baseline, and saves the results to `benches/results.md`. 
It's used to keep a minimal record of performance across commits.

```bash
./bench.sh
```

For more detailed analysis, HTML reports with graphs are written to `target/criterion/` after any bench run.

## Testing against Stockfish

Requires [cutechess-cli](https://github.com/cutechess/cutechess) and Stockfish installed.

Build the engine first:

```bash
cargo build --release
```

Then run:

```bash
cutechess-cli \
  -engine name=corman cmd=./target/release/corman \
  -engine name=stockfish cmd=stockfish option.UCI_LimitStrength=true option.UCI_Elo=1800 \
  -each proto=uci tc=40/10 \
  -openings format=epd file=2moves_v1.epd order=random \
  -games 10 -repeat -concurrency 1 \
  -pgnout results.pgn
```

- `tc=40/10` -> 40 moves in 10 seconds per side
- `UCI_Elo=1800` -> limits Stockfish strength (raise to make it harder)
- `-games 10 -repeat` -> plays each opening twice (once per color)
- `-pgnout results.pgn` -> saves the games for review

-----------

Btw: Cornelia 🐈 + Norman 🐈‍⬛ = Corman. They're my cats.
