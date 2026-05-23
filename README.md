# Solance

Local chess analysis platform. No cloud. No data sent anywhere.

## Requirements

- Rust (stable)
- Node.js + bun
- Stockfish (`pacman -S stockfish` on Arch)

## Running

```bash
# Terminal 1 — analysis backend
cargo run --bin solance-web

# Terminal 2 — web frontend
cd web && node build
```

Open http://localhost:3000

## Features

- PGN game review with move-by-move analysis
- Accuracy scores using Lichess win% model
- FEN position evaluation
- Multi-game PGN support
- Keyboard navigation (← →)
- Best move highlighting on board

## CLI usage

```bash
cargo run --bin solance-cli -- game.pgn --depth 16
```
