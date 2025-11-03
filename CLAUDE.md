# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a chess game implementation in Rust with OpenGL rendering using the Glium library. The project includes chess game logic, board visualization, and interactive gameplay for two players.

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

### Module Structure

The codebase is organized into three main modules:

1. **game_repr** - Core chess game representation and logic
2. **board_drawer** - OpenGL rendering of the chess board
3. **agent** - Event handling and game state management

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

### board_drawer Module

Located in `src/board_drawer/`, handles all rendering:

- **board.rs**: Main `BoardDrawer` struct that orchestrates rendering
  - `draw_position()`: Renders the complete board state with optional selected tile highlight
  - `coord_to_tile()`: Converts mouse coordinates to board square indices
  - Supports both White and Black POV (reverses board when pov is Black)
  - Maintains aspect ratio for the board regardless of window dimensions

- **tile_drawer.rs**: Renders individual chess pieces on squares
- **dot_drawer.rs**: Renders dots indicating legal moves for selected piece

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

### Rendering Flow

1. Main event loop in `src/main.rs` creates a Glium display and `TwoPlayerAgent`
2. Agent handles input events (mouse clicks, window resize, etc.)
3. On relevant events, calls `BoardDrawer.draw_position()`
4. BoardDrawer renders tiles, pieces, and legal move indicators
5. When POV is Black, indices are reversed (63 - idx) for rendering

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

From code comments:
- Castling move type is not fully implemented in `mk_move()` (position.rs:83)
- FEN parsing doesn't handle castling rights notation (position.rs:27)
- TwoPlayerAgent.mouse_click() needs refactoring (two_player.rs:27-28)
- Move struct has unused methods `from_u8()` and `to_u8()` (moves.rs:59-64)
