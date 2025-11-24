# Chess Engine

A high-performance chess game implementation in Rust with modern WGPU rendering, AI opponent, and web deployment support.

## Features

- **Full Chess Implementation**: Complete chess rules including castling, en passant, pawn promotion, check, checkmate, stalemate, and draw detection
- **Modern Graphics**: Hardware-accelerated rendering using WGPU with custom WGSL shaders
- **AI Opponent**: Negamax search with alpha-beta pruning, quiescence search, and iterative deepening
  - Transposition table for position caching
  - Move ordering with MVV-LVA heuristic
  - Piece-square tables for positional evaluation
  - Multiple difficulty levels (adjustable search depth)
- **Multiple Game Modes**:
  - Player vs Player (PvP)
  - Player vs AI (PvAI)
  - AI vs AI (watch computer play itself)
- **Interactive UI**:
  - Main menu with game mode selection
  - Legal move highlighting
  - Pawn promotion selection overlay
  - Game end overlays with results
- **Web Deployment**: Full WASM support for playing in the browser
- **Cross-Platform**: Runs on Windows, macOS, Linux, and web browsers
- **High Performance**: Optimized move generation (82.8M nodes/sec) with bitboards and lazy evaluation

## Screenshots

See the web version in action by opening `index.html` in a browser after building for WASM.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- For web deployment: [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

## Building and Running

### Desktop Application

```bash
# Clone the repository
git clone <repository-url>
cd chess_engine-claude

# Build the project
cargo build

# Run the application
cargo run

# Build optimized release version (recommended for best performance)
cargo build --release
cargo run --release
```

### Web Application (WASM)

```bash
# Build for WebAssembly
wasm-pack build --target web

# Serve the application (use any static file server)
# For example, with Python:
python -m http.server 8000

# Open http://localhost:8000 in your browser
```

### Testing

```bash
# Run test suite
cargo test

# Run with optimizations (includes long-running Perft validation tests)
cargo test --release

# Run benchmarks
cargo bench
```

## How to Play

### Starting a Game

1. Launch the application
2. Select a game mode from the main menu:
   - **PvP**: Play against another human locally
   - **PvAI**: Play against the computer
   - **AIvAI**: Watch two AI opponents play

### Playing

1. Click on a piece to select it (your color's pieces only)
2. Legal move destinations will be highlighted
3. Click on a highlighted square to move the piece
4. For pawn promotion, select the desired piece (Queen, Rook, Bishop, or Knight) from the overlay
5. The board automatically rotates to show the current player's perspective

### Game End

- The game ends when checkmate, stalemate, or draw by insufficient material occurs
- An overlay displays the result
- Click "Return to Menu" to start a new game

## Architecture

The project follows a component-based architecture with clear separation of concerns:

- **Orchestrator**: Manages application lifecycle and game mode coordination
- **Board**: Shared state encapsulating game logic (Position) and rendering
- **Player Trait**: Abstraction for move providers (Human, AI)
- **Renderer Trait**: Abstraction for rendering implementations (WGPU for native and web)

### Module Structure

- `game_repr/`: Core chess logic (position, moves, pieces, bitboards)
- `renderer/`: WGPU-based rendering with UI elements
- `agent/`: Player implementations (human input, AI search)
- `orchestrator.rs`: Application state management
- `board.rs`: Shared board state and interaction handling

For detailed architecture documentation, see [CLAUDE.md](CLAUDE.md).

## Performance

The engine has been optimized for move generation performance:

**Current Performance:** 1.45s for perft depth 6 (82.8M nodes/sec)
**Baseline:** 20.83s (14.4x improvement)

**Optimizations Applied:**
- Bitboard representation with precomputed lookup tables
- Move list recycling to eliminate heap allocations
- Pin detection for bulk move validation
- Lazy move validation (skip validation for unpinned pieces when not in check)

## AI Implementation

The AI opponent uses a sophisticated search algorithm:

- **Negamax with Alpha-Beta Pruning**: Efficient minimax variant with pruning
- **Iterative Deepening**: Progressively deeper searches with time management
- **Quiescence Search**: Extends search to quiet positions to avoid horizon effect
- **Transposition Table**: Caches evaluated positions to avoid redundant work
- **Move Ordering**: Prioritizes promising moves (captures, checks) for better pruning
- **Evaluation Function**: Considers material, piece positioning, king safety, and pawn structure

## License

This is a hobby project. Feel free to use and modify as needed.

## Credits

- Piece images: Standard chess piece set
- Font: Roboto (embedded)
- Built with Rust, WGPU, and winit
