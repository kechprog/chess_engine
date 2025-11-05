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
  - `request_move()`: Async method to get next move from player
  - `handle_event()`: Process window events (mouse, keyboard, etc.)
  - Supports different player types: Human, AI (future), Network (future)

- **human_player.rs**: `HumanPlayer` implementation for local gameplay
  - Manages piece selection and legal move display
  - Handles mouse input for piece selection and movement
  - Validates moves against legal move list before executing
  - Supports promotion piece selection through UI overlay

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
2. User selects mode (currently only PvP is implemented)
3. Orchestrator creates appropriate Player instances (two HumanPlayers for PvP)
4. Game loop begins:
   - Request move from current player
   - Player handles input events until move is made
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

These optimizations are particularly effective for deep position analysis (Perft testing) and will benefit future AI implementations.

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
- AI opponent implementation (Player trait designed to support this)
  - Consider using the performance optimizations for tree search
  - Implement minimax or alpha-beta pruning
  - Add evaluation function for position scoring
- Network multiplayer (Online game mode)
  - Implement network Player type
  - Add lobby system and matchmaking
- Time controls and chess clock
- Undo/Redo functionality (mk_move/unmk_move infrastructure exists)
- Save/Load games (PGN format)
- Move history display
- Opening book and endgame tablebase support

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
- **Adding new player types**: Implement `Player` trait (see `HumanPlayer` as example)
- **Adding new UI elements**: Extend `Renderer` trait and implement in `WgpuRenderer`
- **Cross-platform support**: WGPU abstracts graphics API (works on Windows, macOS, Linux, Web)

The use of `Arc<RefCell<Board>>` for shared state is safe because:
- All access happens on the main thread (winit event loop)
- RefCell provides runtime borrow checking
- Borrows are kept short-lived to avoid panics
