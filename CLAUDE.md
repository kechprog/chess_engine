# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a chess game implementation in Rust with modern WGPU rendering. The project includes comprehensive chess game logic, board visualization, interactive gameplay, and web deployment support.

**Current State**: Production-ready chess application with component-based architecture, complete rule implementation, UI features (menu, overlays, promotion selection), comprehensive testing, and WASM support for web deployment.

**Recent Achievements**:
- Component-based architecture (Orchestrator + Board + Player trait)
- 14.4× performance improvement through bitboards and optimizations
- Full UI system with menu, game end overlays, and promotion selection
- WASM support for browser deployment
- Comprehensive test suite with Perft validation
- Classical chess AI with Negamax algorithm and alpha-beta pruning
- Four difficulty levels from beginner to club-level strength

## Build and Run

```bash
# Build the project
cargo build

# Run the application (opens a window with the chess board)
cargo run

# Build in release mode (highly optimized)
cargo build --release

# Run tests
cargo test

# Run performance benchmarks
cargo bench

# Build for WASM (requires wasm-pack)
wasm-pack build --target web
```

## Architecture

The codebase follows a component-based architecture with clear separation of concerns:

- **Orchestrator**: Root component managing application lifecycle, game modes, and turn coordination
- **Board**: Shared state object encapsulating Position (game logic) and Renderer (graphics)
- **Player Trait**: Abstraction for move providers (Human, AI, Network) with uniform interface
- **Renderer Trait**: Abstraction for rendering implementations (WGPU with native and web targets)

### Module Structure

The codebase is organized into main modules:

1. **game_repr** - Core chess game representation and logic
2. **renderer** - WGPU-based rendering with UI elements
3. **agent** - Player abstraction and input handling
4. **orchestrator** - Application lifecycle and game mode management
5. **board** - Shared board state and interaction handling

### game_repr Module

Located in `src/game_repr/`, this module handles all chess logic:

- **position.rs**: Core `Position` struct representing board state. Includes:
  - FEN string parsing (`from_fen()`)
  - Move execution (`mk_move()` / `unmk_move()`)
  - Legal move generation (`legal_moves()`)
  - Check/checkmate/stalemate detection
  - Castling condition tracking (6-bit array for king/rook movement state)
  - Performance-optimized with bitboards for piece tracking

- **moves.rs**: Compact move representation using 16-bit encoding:
  - 6 bits for source square (from)
  - 6 bits for destination square (to)
  - 4 bits for move type (Normal, EnPassant, Promotion, Castling)
  - Includes promotion piece type for queening moves

- **piece.rs**: `Piece` struct with color and type enums. Includes FEN character parsing and texture loading from `src/assets/`

- **piece_moves/**: Individual files for each piece type's move generation logic (pawn.rs, rook.rs, knight.rs, bishop.rs, queen.rs, king.rs)

- **bitboards/**: Bitboard-based optimizations for piece tracking and move generation
  - Pre-computed attack tables for knights and kings
  - Efficient piece location tracking using 64-bit bitmasks

### renderer Module

Located in `src/renderer/`, handles all graphics rendering:

- **mod.rs**: `Renderer` trait defining the rendering interface
  - `draw_position()`: Renders the complete board state with optional selected tile
  - `draw_menu()`: Renders the main menu with game mode selection
  - `draw_game_end()`: Renders game end overlay (checkmate, stalemate, draw)
  - `draw_promotion_selection()`: Renders pawn promotion piece selection overlay
  - `coord_to_tile()`: Converts mouse coordinates to board square indices
  - `coord_to_promotion_choice()`: Converts click to promotion piece selection

- **wgpu_renderer.rs**: `WgpuRenderer` implementation
  - Supports both White and Black POV (reverses board when pov is Black)
  - Maintains aspect ratio for the board regardless of window dimensions
  - Uses WGPU (modern graphics API) with custom WGSL shaders
  - Implements texture caching for piece images
  - Text rendering using glyphon library for UI elements
  - Works on both native (desktop) and WASM (web) targets

### agent Module

Located in `src/agent/`, handles player abstraction and input:

- **player.rs**: `Player` trait definition for move providers
  - `get_move()`: Method to get next move from player
  - `handle_event()`: Process window events (mouse, keyboard, etc.)
  - Supports different player types: Human, AI, Network (future)

- **human_player.rs**: `HumanPlayer` implementation for local gameplay
  - Manages piece selection and legal move display
  - Handles mouse input for piece selection and movement
  - Validates moves against legal move list before executing
  - Supports promotion piece selection through UI overlay

- **ai/**: AI module implementing Negamax-based chess engine
  - See "AI Implementation" section for full details
  - Provides `NegamaxPlayer` with four difficulty levels
  - Classical minimax search with modern optimizations

### orchestrator Module

Located in `src/orchestrator.rs`, manages the application:

- **GameMode**: Enum for application states (Menu, PvP, PvAI, AIvAI, Online)
- **Orchestrator**: Root component coordinating game flow
  - Creates and manages Player instances based on game mode
  - Executes game loop: request move → process move → check end → switch turn
  - Routes events to appropriate handlers (menu, players, overlays)
  - Manages transitions between game modes
  - Handles game end detection and display

### board Module

Located in `src/board.rs`, the central state object:

- **Board**: Wraps Position and Renderer into unified interface
  - Encapsulates game state (`Position`) and rendering (`Renderer`)
  - Provides query methods (`piece_at()`, `legal_moves_for()`, etc.)
  - Handles user interaction (`handle_click()`, `update_mouse_pos()`)
  - Caches legal moves when piece is selected for performance
  - Manages POV (point of view) for board orientation
  - Shared between Orchestrator and Players via `Arc<RefCell<Board>>`

### Board Representation

- Board is a flat 64-element array indexed 0-63
- Index 0 = a1 (bottom-left from White's perspective)
- Index 63 = h8 (top-right from White's perspective)
- Row calculation: `row = idx / 8`, Column: `col = idx % 8`

### Rendering Flow

1. Main event loop in `src/main.rs` creates window and `Orchestrator`
2. Orchestrator determines current mode (Menu, PvP, etc.) and renders accordingly
3. In Menu mode: calls `Board.draw_menu()` to show game mode selection
4. In active game: calls `Board.draw()` to render board with pieces and legal moves
5. Renderer uses WGPU to draw:
   - Colored tiles (light/dark squares, selection highlight)
   - Textured pieces (with alpha blending)
   - Legal move indicators (semi-transparent dots)
   - UI overlays (menu, promotion selection, game end)
   - Text elements (titles, buttons, game results)
6. When POV is Black, indices are reversed (63 - idx) for rendering

### Game Flow

1. Application starts in Menu mode showing game mode selection
2. User selects mode (PvP, PvAI with difficulty selection, or other modes)
3. Orchestrator creates appropriate Player instances:
   - PvP: Two HumanPlayers
   - PvAI: One HumanPlayer and one NegamaxPlayer
   - AIvAI: Two NegamaxPlayers (for testing/demonstration)
4. Game loop begins:
   - Request move from current player
   - Human players handle input events until move is made
   - AI players perform search and return best move
   - Process move (execute, check for game end)
   - If game ends, show result overlay with "Return to Menu" option
   - Otherwise, switch turn and repeat
5. User can return to menu after game ends

### Move Execution

1. User clicks a piece (sets `selected_tile`)
2. Legal moves calculated via `Position.legal_moves()` and cached
3. Legal move indicators displayed on destination squares
4. User clicks destination square
5. If move is promotion, show promotion selection overlay
6. If move is legal, execute via `Position.mk_move()`
7. Check for checkmate, stalemate, or draw
8. Switch turn to opposite color
9. Update POV to show board from current player's perspective

## Performance Optimizations

The engine has undergone significant performance optimization achieving **14.4× overall improvement**:

1. **Bitboards (1.30×)**: Use 64-bit bitmasks for piece tracking instead of scanning arrays
2. **Move List Recycling (2.34×)**: Reuse allocated move vectors to reduce allocations
3. **Bulk Move Validation (1.22×)**: Detect pinned pieces once per move generation
4. **Lazy Validation (3.0×)**: Defer expensive legality checks until needed

These optimizations are particularly effective for deep position analysis (Perft testing) and benefit the AI implementation.

## AI Implementation

Located in `src/agent/ai/`, this module implements a classical chess AI based on the Negamax algorithm with alpha-beta pruning. The AI provides four difficulty levels and plays at club-level strength.

### Architecture Overview

The AI follows a clean separation of concerns:

- **negamax_player.rs**: `NegamaxPlayer` - Implements the `Player` trait, provides difficulty levels
- **search.rs**: Iterative deepening search orchestrator with time management
- **negamax.rs**: Core Negamax algorithm with alpha-beta pruning
- **quiescence.rs**: Quiescence search to avoid horizon effect
- **evaluation.rs**: Position evaluation with tapered eval (middlegame/endgame)
- **move_ordering.rs**: Move ordering heuristics (killer moves, history table)
- **piece_square_tables.rs**: Positional evaluation tables for each piece type
- **transposition_table.rs**: Zobrist hash-based position caching

### Negamax Algorithm

The core search algorithm is **Negamax with alpha-beta pruning**, a variant of Minimax that simplifies implementation by exploiting the zero-sum property of chess. Key features:

- **Alpha-Beta Pruning**: Eliminates branches that cannot improve the best line, dramatically reducing search space
- **Principal Variation Search (PVS)**: Searches first move with full window, subsequent moves with null window for efficiency
- **Null Move Pruning**: If passing the turn still gives a winning position, prunes the branch early
- **Mate Distance Pruning**: Optimizes checkmate detection by preferring shorter mating sequences
- **Checkmate Score**: Uses `MATE_SCORE = 30000` adjusted by depth to prefer faster mates

The search recursively explores the game tree up to a specified depth, calling quiescence search at leaf nodes.

### Iterative Deepening

The search uses **iterative deepening** (depth 1, 2, 3, ..., max_depth) which provides several benefits:

- **Better Move Ordering**: Results from shallower searches improve ordering for deeper searches
- **Time Management**: Can stop at any depth and return the best move found so far
- **Principal Variation**: Tracks best move sequence for improved alpha-beta efficiency
- **Aspiration Windows**: Optional optimization that narrows the search window based on previous iteration's score

Typical search times:
- Depth 2: ~0.1 seconds (Easy)
- Depth 4: ~1 second (Medium)
- Depth 6: ~5 seconds (Hard)
- Depth 8: up to 5 seconds (Expert, time-limited)

### Quiescence Search

Addresses the **horizon effect** where static evaluation in the middle of tactical sequences gives inaccurate scores. Instead of stopping at fixed depth, quiescence search extends tactical lines (captures, promotions) until the position is "quiet."

Optimizations implemented:
- **Stand-pat**: Current evaluation can cause beta cutoff without searching any moves
- **Delta Pruning**: Skips captures that cannot possibly improve alpha (even with perfect play)
- **MVV-LVA Ordering**: Most Valuable Victim - Least Valuable Attacker (e.g., pawn takes queen before queen takes pawn)
- **Depth Limiting**: Maximum 16 plies to prevent infinite recursion in complex positions

This prevents evaluating positions like "I just captured the opponent's queen" when in reality "they immediately recapture."

### Position Evaluation

The evaluation function returns a score in **centipawns** (1/100th of a pawn, so 100 = 1 pawn advantage) from the perspective of the side to move. Components:

**Material Values**:
- Pawn: 100
- Knight: 300
- Bishop: 320
- Rook: 500
- Queen: 900
- King: 0 (cannot be captured)

**Piece-Square Tables (PST)**: Positional bonuses/penalties based on piece placement
- Different tables for middlegame and endgame
- Encourages central control, king safety, piece activity
- Tapered evaluation smoothly interpolates based on game phase

**Pawn Structure** (tapered middlegame/endgame):
- Doubled Pawns: -15/-20 (pawns on same file)
- Isolated Pawns: -20/-25 (no friendly pawns on adjacent files)
- Passed Pawns: +40/+70 (no enemy pawns can block)
- Pawn Shield: +15/+5 (pawns protecting king)

**Piece Mobility** (tapered):
- Knight: 4/4 per square
- Bishop: 5/5 per square
- Rook: 2/4 per square
- Queen: 1/2 per square
- King: 0/3 per square (mobility valuable in endgame)

**Piece Coordination** (tapered):
- Bishop Pair: +40/+50 (two bishops are powerful together)
- Rook on Open File: +25/+25 (no pawns blocking)
- Rook on Semi-Open File: +12/+12 (no own pawns)
- Rook on Seventh Rank: +18/+25 (attacking opponent's pawns)
- Connected Rooks: +15/+15 (clear path between rooks)

**King Safety**: Pawn shield bonus for pawns near king (important in middlegame)

**Game Phase Detection**: Calculates phase (0-256) based on remaining material:
- 256 = Opening (all pieces present)
- 0 = Endgame (only pawns and kings remain)
- Tapered evaluation interpolates smoothly between middlegame and endgame scores

### Move Ordering

Critical for alpha-beta efficiency. Searches moves in this priority order:

1. **Hash Move**: Best move from transposition table (from previous search or deeper iteration)
2. **Captures (MVV-LVA)**: Most valuable victim, least valuable attacker
   - Queen takes pawn: score 10 × 100 - 900 = 100
   - Pawn takes queen: score 10 × 900 - 100 = 8900 (much higher priority!)
3. **Promotions**: Pawn promotion (typically to Queen)
4. **Killer Moves**: Non-captures that caused beta cutoffs at this depth (stores 2 per depth)
5. **History Heuristic**: Non-captures that have historically been good
6. **Other Moves**: Remaining quiet moves

Good move ordering can improve alpha-beta efficiency by 5-10× by finding beta cutoffs early.

### Transposition Table

Uses **Zobrist hashing** to identify positions that have been evaluated before. Key features:

- **Hash Computation**: XOR-based hashing of all position features (pieces, castling rights, en passant, side to move)
- **Incremental Updates**: Efficiently update hash after moves without full recomputation
- **Cache Size**: Default 1M entries (~40-80MB memory usage)
- **Node Types**:
  - **Exact**: Fully searched position with exact score
  - **LowerBound**: Beta cutoff occurred, score is at least this value
  - **UpperBound**: All moves failed low, score is at most this value
- **Replacement Strategy**: Prefers deeper searches and exact scores over bounds
- **Statistics Tracking**: Hit rate monitoring for tuning

Typical hit rates during search: 60-80% on repeated searches, providing significant speedup.

### Difficulty Levels

The `NegamaxPlayer` provides four difficulty levels through the `Difficulty` enum:

- **Easy**: Depth 2, no time limit (~0.1s per move)
  - Suitable for beginners, makes basic tactical moves
- **Medium**: Depth 4, no time limit (~1s per move)
  - Suitable for intermediate players, sees 2 moves ahead
- **Hard**: Depth 6, no time limit (~5s per move)
  - Suitable for advanced players, sees 3 moves ahead
- **Expert**: Depth 8, 5 second time limit (up to 5s per move)
  - Very strong play with deep calculation and time control

### Usage

The AI integrates seamlessly with the Player trait abstraction:

```rust
use chess_engine::agent::ai::{NegamaxPlayer, Difficulty};

// Create AI with desired difficulty
let ai_player = NegamaxPlayer::with_difficulty(board.clone(), Difficulty::Hard);

// Or with custom name
let ai_player = NegamaxPlayer::new(
    board.clone(),
    Difficulty::Expert,
    "Stockfish Lite".to_string()
);

// AI automatically promotes to Queen (can be overridden)
// Uses get_move() to return best move after search
```

### Search Statistics

In debug builds, the AI logs detailed search information:
- Depth reached
- Nodes searched (positions evaluated)
- Best move score in centipawns
- Principal variation length
- Nodes per second (NPS)

Example output:
```
[AI (Hard)] Searched to depth 6, evaluated 124532 positions, best move score: 45
  Principal variation: 7 moves
info depth 6 score cp 45 nodes 124532 time 3421 nps 36401 pv Some(e2e4)
```

### Performance Characteristics

- **Nodes per Second**: ~30,000-50,000 NPS (varies by position complexity)
- **Branching Factor**: Effective branching factor ~3-5 with good move ordering
- **Memory Usage**: ~40-80MB for transposition table (1M entries)
- **Deterministic**: Same position always produces same move (no randomness)
- **Scalability**: Each depth doubles approximately triples search time

### Future Enhancements

Potential improvements to the AI:
- Parallel search (multi-threading for faster move calculation)
- Opening book (precomputed best opening moves)
- Endgame tablebases (perfect play in endgame positions)
- Pondering (thinking during opponent's time)
- Underpromotion support (currently always promotes to Queen)
- Enhanced pruning (Late Move Reductions, Futility Pruning)
- Selectivity techniques (Singular Extensions, Check Extensions)

## Key Implementation Details

- **FEN Parsing**: Supports full FEN notation including castling rights, en passant, and move counters

- **Move Types**: Four move types are fully implemented:
  - Normal: Standard piece movement
  - EnPassant: Pawn captures pawn en passant
  - Promotion: Pawn reaches back rank (player selects piece via UI)
  - Castling: King and rook special move (both kingside and queenside)

- **POV Handling**: The board can be viewed from either player's perspective. When `pov` is Black, all board indices are reversed for both rendering and coordinate conversion.

- **Asset Loading**: Piece textures are embedded at compile time using `include_bytes!()` for WASM compatibility

- **Text Rendering**: Uses glyphon library with embedded Roboto font for menu and overlay text

- **Game End Detection**:
  - Checkmate: King in check with no legal moves
  - Stalemate: No legal moves but not in check
  - Draw by insufficient material: K vs K, KB vs K, KN vs K, KB vs KB (same color bishops)

## WASM Support

The application can be compiled to WebAssembly for browser deployment:

1. Build with `wasm-pack build --target web`
2. Serve `index.html` with a local web server
3. The application runs in the browser with full functionality
4. Uses WebGL backend for WGPU rendering

## Testing

Comprehensive test suite covering all chess rules:

- **Unit Tests**: Individual piece movement, special moves (castling, en passant, promotion)
- **Integration Tests**: Checkmate, stalemate, check detection
- **Perft Tests**: Move generation correctness verified against known positions
  - Starting position: 119,060,324 positions at depth 6
  - Kiwipete: 8,031,647,685 positions at depth 5
  - Various complex positions testing edge cases
- **Regression Tests**: Prevent previously fixed bugs from reoccurring

Run tests with `cargo test` (standard suite) or `cargo test --release` (includes long-running Perft tests).

## Known TODOs and Next Steps

### Features to Add
- Network multiplayer (Online game mode)
  - Implement network Player type
  - Add lobby system and matchmaking
  - Real-time move synchronization
- Time controls and chess clock
  - Per-player time tracking
  - Time control formats (bullet, blitz, rapid, classical)
  - Increment/delay support
- Undo/Redo functionality (mk_move/unmk_move infrastructure exists)
- Save/Load games (PGN format)
  - Export games to PGN
  - Import and replay PGN games
- Move history display
  - Scrollable move list in algebraic notation
  - Click to jump to position
- Game analysis features
  - Show best move hint
  - Post-game analysis with AI evaluation

### AI Enhancements
The AI is functional but could be improved:
- Opening book integration (precomputed best opening moves)
- Endgame tablebases (perfect play in 3-5 piece endgames)
- Parallel search (multi-threading for faster move calculation)
- Pondering (thinking during opponent's time)
- Enhanced pruning (Late Move Reductions, Futility Pruning)
- Selectivity techniques (Singular Extensions, Check Extensions)
- Underpromotion support (currently always promotes to Queen, though engine evaluates all options)
- Tuned evaluation weights (currently hand-tuned, could use automated tuning)
- Neural network evaluation (modern alternative to classical eval)

### Code Quality
- Move struct has unused methods `from_u8()` and `to_u8()` (moves.rs) - consider removing
- Consider refactoring large methods in wgpu_renderer.rs for better maintainability
- Add more inline documentation for complex algorithms

### Performance
- Current optimizations focused on move generation
- Future work: Investigate SIMD for bitboard operations
- Profile and optimize rendering pipeline for large screens

## Architecture Notes

The current architecture is designed for extensibility:

- **Adding new game modes**: Implement in `GameMode` enum and add handling in `Orchestrator`
- **Adding new player types**: Implement `Player` trait (see `HumanPlayer` and `NegamaxPlayer` as examples)
- **Adding new UI elements**: Extend `Renderer` trait and implement in `WgpuRenderer`
- **Cross-platform support**: WGPU abstracts graphics API (works on Windows, macOS, Linux, Web)
- **AI customization**: Modify evaluation weights, pruning parameters, or search heuristics in AI module

The use of `Arc<RefCell<Board>>` for shared state is safe because:
- All access happens on the main thread (winit event loop)
- RefCell provides runtime borrow checking
- Borrows are kept short-lived to avoid panics

### Player Trait Design

The `Player` trait abstraction allows seamless integration of different player types:

```rust
pub trait Player {
    fn get_move(&mut self, color: Color) -> Option<Move>;
    fn handle_event(&mut self, event: &WindowEvent);
    fn opponent_moved(&mut self, mv: Move);
    fn game_ended(&mut self, result: GameResult);
    fn name(&self) -> &str;
    fn get_promotion_choice(&self) -> Option<Type>;
}
```

This design enables:
- Human players that wait for mouse input
- AI players that compute moves algorithmically
- Network players (future) that receive moves over the network
- Hybrid players (future) with AI assistance

The Orchestrator remains agnostic to player implementation, simply calling `get_move()` regardless of player type.
