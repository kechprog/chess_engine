# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a chess game implementation in Rust with modern WGPU rendering. The project includes comprehensive chess game logic, board visualization, and interactive gameplay.

**Current State**: Functional two-player chess game with complete rule implementation and testing.

**Architecture Refactor in Progress**: Transitioning to a flexible component-based architecture supporting multiple game modes (Player vs Player, Player vs AI, etc.). See [ARCHITECTURE.md](./ARCHITECTURE.md) for complete architectural documentation and refactor plan.

## Build and Run

```bash
# Build the project
cargo build

# Run the application (opens a window with the chess board)
cargo run

# Build in release mode
cargo build --release

# Run tests
cargo test
```

## Architecture

**For comprehensive architectural documentation, see [ARCHITECTURE.md](./ARCHITECTURE.md)**

The architecture is currently being refactored from a monolithic design to a component-based system:
- **Orchestrator**: Root component managing game modes and lifecycle
- **Board**: Shared state object for game logic and rendering
- **Player Trait**: Abstraction for move providers (Human, AI, Network)

### Current Module Structure

The codebase is organized into main modules:

1. **game_repr** - Core chess game representation and logic (stable, not changing)
2. **renderer** - WGPU-based rendering (replacing old board_drawer)
3. **agent** - Event handling and game state management (being refactored)
4. **orchestrator** - Application lifecycle management (new, to be implemented)
5. **board** - Shared board state (new, to be implemented)

### game_repr Module

Located in `src/game_repr/`, this module handles all chess logic:

- **position.rs**: Core `Position` struct that represents the board state as a 64-element array of `Piece`. Includes:
  - FEN string parsing (`from_fen()`)
  - Move execution (`mk_move()`)
  - Legal move generation (`legal_moves()`)
  - Castling condition tracking (6-bit array for king/rook movement state)

- **moves.rs**: Compact move representation using 16-bit encoding:
  - 6 bits for source square (from)
  - 6 bits for destination square (to)
  - 4 bits for move type (Normal, EnPassant, Promotion, Castling)

- **piece.rs**: `Piece` struct with color and type enums. Includes FEN character parsing and OpenGL texture loading from `src/assets/`

- **piece_moves/**: Individual files for each piece type's move generation logic (pawn.rs, rook.rs, knight.rs, bishop.rs, queen.rs, king.rs)

### renderer Module

Located in `src/renderer/`, handles all graphics rendering:

- **wgpu_renderer.rs**: Main `WgpuRenderer` struct implementing the Renderer trait
  - `draw_position()`: Renders the complete board state with optional selected tile highlight
  - `coord_to_tile()`: Converts mouse coordinates to board square indices
  - Supports both White and Black POV (reverses board when pov is Black)
  - Maintains aspect ratio for the board regardless of window dimensions
  - Uses WGPU (modern graphics API) with custom shaders for tiles, pieces, and legal move indicators
  - Implements texture caching for piece images

### agent Module

Located in `src/agent/`, handles game flow and user interaction:

- **agent.rs**: Trait definition for game agents
- **two_player.rs**: `TwoPlayerAgent` implementation for local two-player games
  - Manages game state (`Position` and `turn`)
  - Handles mouse input for piece selection and movement
  - Validates moves against legal move list before executing
  - Switches turn and POV after each valid move

### Board Representation

- Board is a flat 64-element array indexed 0-63
- Index 0 = a1 (bottom-left from White's perspective)
- Index 63 = h8 (top-right from White's perspective)
- Row calculation: `row = idx / 8`, Column: `col = idx % 8`

### Rendering Flow (Current - Being Refactored)

1. Main event loop in `src/main.rs` creates a window and `TwoPlayerAgent`
2. Agent handles input events (mouse clicks, window resize, etc.)
3. On redraw events, calls `WgpuRenderer.draw_position()`
4. Renderer uses WGPU to render:
   - Colored tiles (light/dark squares, selection highlight)
   - Textured pieces (with alpha blending)
   - Legal move indicators (semi-transparent dots)
5. When POV is Black, indices are reversed (63 - idx) for rendering

**Future**: Rendering will be managed by the Board component, which owns the Renderer

### Move Execution

1. User clicks a piece (sets `selected_tile`)
2. Legal moves calculated via `Position.legal_moves()`
3. User clicks destination square
4. If move is in legal moves list, execute via `Position.mk_move()`
5. Switch turn to opposite color
6. Update POV to show board from current player's perspective

## Key Implementation Details

- **FEN Parsing**: The default position uses FEN notation without the full FEN (no castling rights, en passant, etc. in the string). Default castling conditions are set to `[true; 6]`.

- **Move Types**: Four move types are currently implemented:
  - Normal: Standard piece movement
  - EnPassant: Pawn captures pawn
  - Promotion: Pawn reaches back rank (auto-promotes to Queen)
  - Castling: King and rook special move (TODO: not fully implemented)

- **POV Handling**: The board can be viewed from either player's perspective. When `pov` is Black, all board indices are reversed for both rendering and coordinate conversion.

- **Asset Loading**: Piece textures are loaded from `src/assets/` with naming convention `{color}_{piece}_png_128px.png` (e.g., `w_pawn_png_128px.png`)

## Known TODOs and Limitations

### Architecture Refactor
- **In Progress**: Transitioning to Orchestrator + Board + Player architecture (see ARCHITECTURE.md)
- TwoPlayerAgent will be replaced by Orchestrator + HumanPlayer
- Legacy `chess_repr/` module needs to be deleted

### Code Quality
- TwoPlayerAgent.mouse_click() needs refactoring (will be replaced in new architecture)
- Move struct has unused methods `from_u8()` and `to_u8()` (moves.rs)
- Asset paths are hardcoded (should use `include_bytes!()` or proper asset system)

### Features to Add
- Game mode selection menu
- AI opponent (Player trait designed to support this)
- Undo/Redo functionality
- Save/Load games (PGN format)
- Time controls
