# Chess Engine Architecture Documentation

## Table of Contents
1. [Executive Summary](#executive-summary)
2. [Current Architecture](#current-architecture)
3. [New Architecture Vision](#new-architecture-vision)
4. [Component Specifications](#component-specifications)
5. [Data Flow & Interactions](#data-flow--interactions)
6. [Concurrency & Shared State](#concurrency--shared-state)
7. [Implementation Phases](#implementation-phases)
8. [Migration Strategy](#migration-strategy)
9. [Design Decisions & Rationale](#design-decisions--rationale)

---

## Executive Summary

This document outlines a major architectural refactor of the chess engine, transitioning from a monolithic two-player agent design to a flexible, component-based architecture that supports multiple game modes (PvP, PvAI, online multiplayer) through a clean separation of concerns.

### Key Changes

**Before**: Monolithic `TwoPlayerAgent` owns everything (game state, renderer, input handling)

**After**:
- **Orchestrator** - Root component managing application lifecycle and mode selection
- **Board** - Shared state object handling both game logic and rendering
- **Player Trait** - Abstraction for move providers (Human, AI, Network)
- Clean separation enabling easy addition of new game modes and player types

---

## Current Architecture

### Component Hierarchy (Before Refactor)

```
main.rs
  └── App (winit ApplicationHandler)
      └── TwoPlayerAgent
          ├── Position (game state)
          ├── Box<dyn Renderer> (graphics)
          ├── Arc<Window> (window handle)
          ├── turn: Color
          ├── selected_tile: Option<u8>
          └── game_over: bool
```

### Limitations of Current Design

1. **Tight Coupling**: Agent mixes game logic, UI state, and input handling
2. **Not Extensible**: Adding AI requires duplicating UI logic in new agent type
3. **No Mode Selection**: Hardcoded to two-player mode
4. **State Ownership**: Position and Renderer owned by agent, can't be shared
5. **Complex Methods**: `mouse_click()` is 150+ lines handling all game flow

### Current Data Flow

```
User Input → TwoPlayerAgent::mouse_click()
  → coord_to_tile()
  → position.legal_moves()
  → position.mk_move()
  → switch turn
  → check game end
  → window.request_redraw()
  → renderer.draw_position()
```

---

## New Architecture Vision

### Component Hierarchy (After Refactor)

```
main.rs
  └── App (winit ApplicationHandler)
      └── Orchestrator
          ├── Window
          ├── Board (Arc<RefCell<Board>>)
          │   ├── Position (game state)
          │   ├── Renderer (graphics)
          │   ├── selected_tile: Option<u8>
          │   └── legal_moves_cache: Vec<Move>
          ├── GameMode (enum: Menu, PvP, PvAI)
          └── Players: Option<(Box<dyn Player>, Box<dyn Player>)>
              ├── HumanPlayer
              │   └── board_ref: Arc<RefCell<Board>>
              └── AIPlayer (future)
                  └── board_ref: Arc<RefCell<Board>>
```

### Architectural Principles

1. **Single Responsibility**: Each component has one clear purpose
2. **Dependency Inversion**: Players depend on Board abstraction
3. **Shared State**: Board is shared via Arc<RefCell<>> for flexible access
4. **Extensibility**: New player types implement Player trait
5. **Composition**: Orchestrator composes components based on selected mode

---

## Component Specifications

### 1. Orchestrator

**Responsibility**: Root component managing application lifecycle and game mode coordination.

#### Data Structure

```rust
pub struct Orchestrator {
    window: Arc<Window>,
    board: Arc<RefCell<Board>>,
    game_mode: GameMode,
    players: Option<(Box<dyn Player>, Box<dyn Player>)>,
    current_turn: Color,
    game_active: bool,
}

pub enum GameMode {
    Menu,           // Mode selection screen
    PvP,            // Player vs Player
    PvAI,           // Player vs AI (future)
    AIvAI,          // AI vs AI (future)
    Online,         // Network multiplayer (future)
}
```

#### Public API

```rust
impl Orchestrator {
    /// Create new orchestrator with window
    pub fn new(window: Arc<Window>) -> Self;

    /// Handle window events (delegates based on current mode)
    pub fn handle_event(&mut self, event: WindowEvent);

    /// Switch to a new game mode
    pub fn set_game_mode(&mut self, mode: GameMode);

    /// Start a game with the current mode
    pub fn start_game(&mut self);

    /// Request next move from current player
    pub fn request_move(&mut self);

    /// Process move received from player
    pub fn process_move(&mut self, mv: Move);

    /// Check and handle game end conditions
    pub fn check_game_end(&mut self);

    /// Return to mode selection menu
    pub fn return_to_menu(&mut self);
}
```

#### Responsibilities

- **Lifecycle Management**: Create/destroy game sessions
- **Mode Selection**: Display UI for picking game mode
- **Player Creation**: Instantiate correct player types for selected mode
- **Turn Management**: Track whose turn it is, request moves from current player
- **Game Flow**: Handle move execution, turn switching, game end detection
- **Event Routing**: Delegate events to Board or handle mode-specific logic

#### State Transitions

```
[Application Start]
    ↓
[Menu Mode] ← ← ← ← ← ← ← ← ←
    ↓ (user selects mode)        ↑
[Create Players]                 ↑
    ↓                            ↑
[Active Game]                    ↑
    ↓ (game loop)                ↑
[Request Move] → [Player provides move]
    ↓                            ↑
[Execute Move]                   ↑
    ↓                            ↑
[Check Game End] ─(if ended)─→ [Menu Mode]
    ↓ (not ended)
[Switch Turn] ──→ [Request Move]
```

---

### 2. Board

**Responsibility**: Shared state object managing chess position, rendering, and UI interactions.

#### Data Structure

```rust
pub struct Board {
    // Game state
    position: Position,

    // Rendering
    renderer: Box<dyn Renderer>,

    // UI state
    selected_tile: Option<u8>,
    legal_moves_cache: Vec<Move>,

    // Display settings
    pov: Color,  // Point of view for rendering

    // Event handling
    mouse_pos: PhysicalPosition<f64>,
}
```

#### Public API

```rust
impl Board {
    /// Create new board with starting position
    pub fn new(renderer: Box<dyn Renderer>) -> Self;

    /// Create board from FEN string
    pub fn from_fen(fen: &str, renderer: Box<dyn Renderer>) -> Self;

    // === Game State Access ===

    /// Get current position (read-only)
    pub fn position(&self) -> &Position;

    /// Get piece at square
    pub fn piece_at(&self, square: u8) -> Piece;

    /// Get all legal moves for current player
    pub fn legal_moves_for(&self, color: Color) -> Vec<Move>;

    /// Check if move is legal
    pub fn is_legal_move(&self, mv: Move) -> bool;

    /// Execute a move (modifies position)
    pub fn execute_move(&mut self, mv: Move);

    /// Check if position is checkmate
    pub fn is_checkmate(&self, color: Color) -> bool;

    /// Check if position is stalemate
    pub fn is_stalemate(&self, color: Color) -> bool;

    /// Check if position is in check
    pub fn is_in_check(&self, color: Color) -> bool;

    // === UI Interaction ===

    /// Handle mouse click, returns selected square
    pub fn handle_click(&mut self, pos: PhysicalPosition<f64>) -> Option<u8>;

    /// Get currently selected tile
    pub fn selected_tile(&self) -> Option<u8>;

    /// Set selected tile and update legal moves cache
    pub fn set_selected_tile(&mut self, tile: Option<u8>);

    /// Get legal moves for selected piece
    pub fn legal_moves_for_selection(&self) -> &[Move];

    /// Update cached mouse position
    pub fn update_mouse_pos(&mut self, pos: PhysicalPosition<f64>);

    // === Rendering ===

    /// Draw current board state
    pub fn draw(&mut self);

    /// Set point of view (which side is at bottom)
    pub fn set_pov(&mut self, pov: Color);

    /// Get current POV
    pub fn pov(&self) -> Color;

    /// Handle window resize
    pub fn resize(&mut self, new_size: (u32, u32));
}
```

#### Responsibilities

- **State Management**: Owns and manages the Position struct
- **Move Validation**: Provides legal move queries and validation
- **Move Execution**: Applies moves to the position
- **UI State**: Tracks selected tile and legal moves for display
- **Rendering**: Draws the board using the renderer
- **Input Processing**: Converts mouse clicks to board squares
- **Caching**: Stores legal moves for selected piece to avoid recomputation

#### Key Design Points

1. **Shared Ownership**: Wrapped in Arc<RefCell<>> for multiple borrows
2. **Encapsulation**: Position is private, accessed through methods
3. **Caching**: Legal moves cached when tile selected (performance)
4. **Immutable Queries**: Many methods take &self for concurrent reads
5. **Mutable State Changes**: execute_move takes &mut self

---

### 3. Player Trait

**Responsibility**: Abstraction for entities that can provide chess moves.

#### Trait Definition

```rust
pub trait Player {
    /// Get the next move for this player
    /// This method may block (e.g., waiting for user input)
    /// Returns None if player cancels/resigns
    fn get_move(&mut self, color: Color) -> Option<Move>;

    /// Notify player that opponent made a move (optional)
    fn opponent_moved(&mut self, mv: Move) {
        // Default: do nothing
    }

    /// Notify player of game end (optional)
    fn game_ended(&mut self, result: GameResult) {
        // Default: do nothing
    }

    /// Get player name for display
    fn name(&self) -> &str {
        "Player"
    }
}

pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
    Stalemate,
}
```

#### Design Rationale

**Why no constructor in trait?**
- Different player types need different construction parameters
- HumanPlayer needs Board reference
- AIPlayer needs engine configuration, difficulty level
- NetworkPlayer needs connection details
- Trait focuses on behavior, not construction

**Why blocking get_move()?**
- Simplifies control flow (no callbacks/futures needed initially)
- HumanPlayer can block until user clicks
- AIPlayer can block during search
- Future: Can make async if needed

---

### 4. HumanPlayer

**Responsibility**: Player implementation that gets moves via UI interaction with Board.

#### Data Structure

```rust
pub struct HumanPlayer {
    board: Arc<RefCell<Board>>,
    name: String,
    pending_move: Option<Move>,
}
```

#### Implementation

```rust
impl HumanPlayer {
    /// Construct human player with reference to board
    pub fn new(board: Arc<RefCell<Board>>, name: String) -> Self {
        Self {
            board,
            name,
            pending_move: None,
        }
    }

    /// Handle window event (called by Orchestrator)
    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput { state: Pressed, button: Left, .. } => {
                self.handle_click();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.board.borrow_mut().update_mouse_pos(*position);
            }
            _ => {}
        }
    }

    /// Process mouse click, potentially creating a move
    fn handle_click(&mut self) {
        let mut board = self.board.borrow_mut();
        let mouse_pos = board.mouse_pos;

        // Get clicked square
        let clicked_tile = match board.handle_click(mouse_pos) {
            Some(tile) => tile,
            None => {
                board.set_selected_tile(None);
                return;
            }
        };

        // If nothing selected, select this tile (if has piece of current color)
        if board.selected_tile().is_none() {
            let piece = board.piece_at(clicked_tile);
            if !piece.is_none() {
                board.set_selected_tile(Some(clicked_tile));
            }
            return;
        }

        // Something selected, try to create move
        let from = board.selected_tile().unwrap();
        let legal_moves = board.legal_moves_for_selection();

        // Check if click creates a legal move
        for mv in legal_moves {
            if mv._from() == from as usize && mv._to() == clicked_tile as usize {
                self.pending_move = Some(*mv);
                board.set_selected_tile(None);
                return;
            }
        }

        // Click didn't create legal move, reselect if friendly piece
        let piece = board.piece_at(clicked_tile);
        if !piece.is_none() {
            board.set_selected_tile(Some(clicked_tile));
        } else {
            board.set_selected_tile(None);
        }
    }
}

impl Player for HumanPlayer {
    fn get_move(&mut self, color: Color) -> Option<Move> {
        // Set POV to current player's perspective
        self.board.borrow_mut().set_pov(color);

        // Clear any pending move from previous call
        self.pending_move = None;

        // This method blocks until user provides a move
        // In practice, called from event loop which processes events
        // Events call handle_event() which sets pending_move
        // Orchestrator polls pending_move after each event

        // For now, return pending move (will be Some when user clicks)
        self.pending_move
    }

    fn name(&self) -> &str {
        &self.name
    }
}
```

#### Control Flow

```
Orchestrator::request_move()
    ↓
player.get_move(color)
    ↓
[Wait for user interaction]
    ↓
WindowEvent::MouseInput
    ↓
Orchestrator::handle_event()
    ↓
player.handle_event(event)
    ↓
player.handle_click()
    ↓ (interact with board)
board.handle_click() → get square
board.set_selected_tile() → update UI
board.legal_moves_for_selection() → check move
    ↓ (if valid move)
self.pending_move = Some(move)
    ↓
Orchestrator polls pending_move
    ↓
player.get_move() returns Some(move)
    ↓
Orchestrator::process_move(move)
```

---

### 5. AIPlayer (Future Implementation)

**Responsibility**: Player implementation that computes moves using chess engine.

#### Data Structure

```rust
pub struct AIPlayer {
    board: Arc<RefCell<Board>>,
    name: String,
    difficulty: Difficulty,
    engine: Box<dyn ChessEngine>,
}

pub enum Difficulty {
    Easy,      // Depth 2-3, random move selection
    Medium,    // Depth 4-5, basic evaluation
    Hard,      // Depth 6-8, full evaluation
    Expert,    // Depth 10+, opening book, endgame tables
}
```

#### Implementation Sketch

```rust
impl Player for AIPlayer {
    fn get_move(&mut self, color: Color) -> Option<Move> {
        let position = self.board.borrow().position().clone();

        // Run engine search (blocks during computation)
        let mv = self.engine.search(position, self.difficulty.depth());

        Some(mv)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
```

---

## Data Flow & Interactions

### Game Startup Flow

```
main()
  → Create EventLoop
  → Create App
  → Run event loop
    → App::resumed()
      → Create Window
      → Create Orchestrator
        → Create Board (with WgpuRenderer)
        → Set game_mode = Menu
        → Render menu
```

### Mode Selection Flow

```
User clicks "Player vs Player"
  → MouseInput event
    → App::window_event()
      → Orchestrator::handle_event()
        → Orchestrator::set_game_mode(PvP)
          → Orchestrator::start_game()
            → Create player1: HumanPlayer(board.clone())
            → Create player2: HumanPlayer(board.clone())
            → Set current_turn = White
            → Set game_active = true
            → Orchestrator::request_move()
```

### Move Execution Flow (PvP)

```
Orchestrator::request_move()
  → player = players[current_turn]
  → player.get_move(current_turn)
    → [HumanPlayer waits for input]

User clicks piece → clicks destination
  → MouseInput events
    → Orchestrator::handle_event()
      → player.handle_event(event)
        → player.handle_click()
          → board.borrow_mut().handle_click()
          → board.set_selected_tile()
          → Check if move created
          → Set player.pending_move = Some(move)

      → Orchestrator polls player.get_move()
        → Returns Some(move)

  → Orchestrator::process_move(move)
    → board.borrow_mut().execute_move(move)
    → Orchestrator::check_game_end()
      → if checkmate/stalemate:
          → Display result
          → Set game_active = false
      → else:
          → Switch current_turn
          → Orchestrator::request_move() (next player)
```

### Rendering Flow

```
Orchestrator needs redraw
  → window.request_redraw()
    → WindowEvent::RedrawRequested
      → App::window_event()
        → Orchestrator::handle_event()
          → board.borrow_mut().draw()
            → renderer.draw_position(
                position,
                selected_tile,
                pov
              )
            → GPU renders to screen
```

---

## Concurrency & Shared State

### Sharing Strategy

**Board as Shared State**: `Arc<RefCell<Board>>`

#### Why Arc?
- Multiple owners: Orchestrator, Player1, Player2 all hold references
- Orchestrator needs to access board for rendering/state queries
- Players need to access board for UI interaction and state queries
- Reference counting ensures board lives as long as needed

#### Why RefCell?
- Interior mutability: Allow mutation through shared reference
- Single-threaded: All access from main thread (winit event loop)
- Runtime borrow checking: Safe mutable access
- Borrow panics if multiple mutable borrows (programming error)

#### Alternative Considered: Arc<Mutex<Board>>
- **Rejected**: Unnecessary overhead, no concurrent threads
- **When to use**: If adding background AI computation thread
- **Trade-off**: Mutex has performance cost, blocks on contention

### Borrow Rules

```rust
// Immutable borrow (multiple allowed)
let board = self.board.borrow();
let position = board.position();

// Mutable borrow (exclusive)
let mut board = self.board.borrow_mut();
board.execute_move(mv);

// PANIC: Cannot hold both
let board_ref = self.board.borrow();
let mut board_mut = self.board.borrow_mut();  // RUNTIME PANIC!
```

### Safety Guidelines

1. **Keep borrows short**: Don't hold RefCell borrow across event boundaries
2. **No nested borrows**: Release borrow before calling methods that might borrow
3. **Clone when needed**: Clone data out of RefCell if need to hold longer
4. **Document borrow requirements**: Comment methods that borrow board

### Example Safe Pattern

```rust
// BAD: Holding borrow too long
let board = self.board.borrow();
let legal_moves = board.legal_moves_for_selection();
for mv in legal_moves {
    self.board.borrow_mut().execute_move(mv);  // PANIC: Already borrowed!
}

// GOOD: Clone out and release borrow
let legal_moves = {
    let board = self.board.borrow();
    board.legal_moves_for_selection().to_vec()
};  // Borrow released here

for mv in legal_moves {
    self.board.borrow_mut().execute_move(mv);  // OK: New borrow
}
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1)

**Goal**: Create new components without breaking existing functionality

#### Tasks:
1. Create `orchestrator.rs` module with Orchestrator struct
2. Create `board.rs` module with Board struct (wrapping current logic)
3. Define Player trait in `agent/player.rs`
4. Move Position/Renderer ownership from TwoPlayerAgent to Board
5. Update imports and module declarations

#### Acceptance Criteria:
- Code compiles
- Existing two-player mode still works through Orchestrator
- No functional changes to gameplay

---

### Phase 2: HumanPlayer Implementation (Week 2)

**Goal**: Extract player logic into separate HumanPlayer struct

#### Tasks:
1. Create `agent/human_player.rs` with HumanPlayer implementation
2. Move click handling logic from TwoPlayerAgent to HumanPlayer
3. Implement Player trait for HumanPlayer
4. Update Orchestrator to use HumanPlayer instances
5. Test two-player mode with new architecture

#### Acceptance Criteria:
- Two-player mode works identically to before
- Click handling properly delegated to HumanPlayer
- Board state updated correctly through shared reference
- No rendering glitches or state desync

---

### Phase 3: Menu System (Week 3)

**Goal**: Add game mode selection UI

#### Tasks:
1. Design menu UI (buttons for PvP, PvAI, etc.)
2. Create menu rendering logic (can reuse WGPU or simple text)
3. Implement GameMode enum and state transitions
4. Add menu event handling in Orchestrator
5. Create "Return to Menu" functionality

#### Acceptance Criteria:
- Application starts in menu mode
- Can select PvP and start game
- PvAI button exists but is disabled/grayed out
- Can return to menu after game ends
- Menu rendering looks clean and professional

---

### Phase 4: Refactoring & Cleanup (Week 4)

**Goal**: Remove legacy code and optimize new architecture

#### Tasks:
1. Delete `agent/two_player.rs` (replaced by Orchestrator + HumanPlayer)
2. Delete `src/chess_repr/` legacy module
3. Extract long methods in HumanPlayer into smaller functions
4. Add comprehensive documentation to new components
5. Update CLAUDE.md with new architecture
6. Add integration tests for Orchestrator and Board

#### Acceptance Criteria:
- No unused legacy code remains
- All public APIs have doc comments
- Code passes `cargo clippy` with no warnings
- Test coverage for new components
- CLAUDE.md accurately reflects new architecture

---

### Phase 5: AI Player Foundation (Week 5+)

**Goal**: Create skeleton for AI player (not full implementation)

#### Tasks:
1. Create ChessEngine trait
2. Create SimpleAI struct (basic minimax with alpha-beta)
3. Implement AIPlayer struct
4. Implement Player trait for AIPlayer
5. Enable PvAI mode in menu
6. Add difficulty selection

#### Acceptance Criteria:
- Can select PvAI from menu
- AI makes legal moves (even if weak)
- No lag in UI during AI thinking
- Can play complete PvAI game
- Difficulty setting affects move quality

---

## Migration Strategy

### Backwards Compatibility

**Problem**: Need to refactor without breaking existing code during development

**Solution**: Incremental migration with coexistence

#### Step 1: Add Without Breaking
- Create new modules alongside old
- Old code path: `main → App → TwoPlayerAgent`
- New code path: `main → App → Orchestrator`
- Use feature flag or compiler cfg to switch

```rust
#[cfg(feature = "new_arch")]
pub use orchestrator::Orchestrator;

#[cfg(not(feature = "new_arch"))]
pub use agent::TwoPlayerAgent;
```

#### Step 2: Gradual Migration
- Start with Orchestrator managing menu
- Delegate to TwoPlayerAgent for actual gameplay
- Incrementally move logic to Board and HumanPlayer
- TwoPlayerAgent shrinks until empty

#### Step 3: Clean Removal
- When new architecture fully working, delete old code
- Remove feature flags
- Update all imports

### Testing During Migration

1. **Dual Testing**: Run both old and new code paths in CI
2. **Perft Validation**: Ensure move generation unchanged
3. **Visual Comparison**: Screenshot tests for rendering
4. **Manual Testing**: Play complete games in both modes

### Rollback Plan

If critical bugs found in new architecture:
1. Revert to old code path via feature flag
2. Fix bugs in new architecture offline
3. Re-enable when stable

---

## Design Decisions & Rationale

### Decision 1: Board Owns Renderer

**Rationale**:
- Board is responsible for displaying game state
- Tight coupling between position and what's rendered
- Board methods can internally trigger redraws
- Simplifies Orchestrator (doesn't need to manage rendering)

**Alternative Considered**: Orchestrator owns renderer
- **Rejected**: Would need to pass renderer to Board for every draw
- **Trade-off**: More complex API, more parameter passing

---

### Decision 2: Blocking get_move()

**Rationale**:
- Simpler control flow (no async/await complexity)
- Matches chess paradigm (players move in turn)
- Easy to implement for HumanPlayer (wait for click)
- Easy to implement for AIPlayer (wait for search)

**Alternative Considered**: Async get_move()
- **Rejected**: Unnecessary complexity for single-threaded game
- **Future**: Can migrate to async if adding networked players
- **Trade-off**: Async more flexible but harder to reason about

---

### Decision 3: Arc<RefCell<>> for Board

**Rationale**:
- Single-threaded application (winit event loop)
- RefCell lighter than Mutex
- Runtime borrow checking catches errors during development
- Clear ownership semantics

**Alternative Considered**: Arc<Mutex<Board>>
- **When to use**: If AI runs in background thread
- **Trade-off**: More overhead, potential for deadlocks
- **Migration path**: Easy to change Arc<RefCell<>> to Arc<Mutex<>>

---

### Decision 4: No Constructor in Player Trait

**Rationale**:
- Different players need different construction parameters
- Trait focuses on behavior (get_move), not construction
- Follows Rust idioms (traits rarely have constructors)
- More flexible (allows builder patterns, factory methods)

**Alternative Considered**: Required `new()` in trait
- **Rejected**: Impossible to enforce uniform parameters
- **Trade-off**: Lose compile-time guarantee of constructor, gain flexibility

---

### Decision 5: Orchestrator Handles Turn Management

**Rationale**:
- Game flow logic belongs at high level
- Players shouldn't know about opponents or turns
- Orchestrator can add observer pattern (spectators, logging)
- Easier to implement draw offers, resignations, time controls

**Alternative Considered**: Board handles turns
- **Rejected**: Board should be pure state + rendering
- **Trade-off**: More complex Board, less reusable

---

### Decision 6: Menu as GameMode (Not Separate Scene)

**Rationale**:
- Simple state machine in Orchestrator
- Reuses same window and event loop
- Easy transitions between menu and game
- Less complex than scene graph

**Alternative Considered**: Separate Scene abstraction
- **When to use**: If adding many screens (settings, replays, etc.)
- **Trade-off**: More abstract but more boilerplate
- **Migration path**: Can extract to Scene trait later

---

## Future Enhancements

### Planned Features (Post-Refactor)

1. **Undo/Redo**
   - Board maintains move history
   - Add `undo()` and `redo()` methods
   - UI buttons for undo/redo
   - Uses existing `unmake_move()` optimization

2. **Save/Load Games**
   - Export position to PGN format
   - Import PGN to start from position
   - Save in-progress games

3. **Time Controls**
   - Add `TimeControl` component
   - Track time per player
   - Update on each move
   - Handle time forfeit

4. **Opening Book**
   - Precomputed opening moves
   - AI uses book in early game
   - User can disable for practice

5. **Position Analysis**
   - Evaluation bar showing advantage
   - Best move hints
   - Blunder detection

6. **Online Multiplayer**
   - NetworkPlayer implementation
   - Websocket connection to server
   - Move synchronization
   - Disconnect handling

7. **Chess Variants**
   - Chess960 (Fischer Random)
   - Three-check chess
   - Atomic chess
   - Abstract Position rules into variants

---

## Appendix: Code Location Map

### New Files to Create

```
src/
├── orchestrator.rs          [Phase 1]
├── board.rs                 [Phase 1]
├── agent/
│   ├── player.rs            [Phase 1]
│   ├── human_player.rs      [Phase 2]
│   └── ai_player.rs         [Phase 5]
└── engine/
    ├── mod.rs               [Phase 5]
    ├── engine.rs            [Phase 5]
    └── simple_ai.rs         [Phase 5]
```

### Files to Modify

```
src/
├── main.rs                  [Update to use Orchestrator]
├── lib.rs                   [Add new module exports]
└── agent/
    └── mod.rs               [Add player submodules]
```

### Files to Delete

```
src/
├── agent/
│   └── two_player.rs        [Phase 4: Delete after migration]
└── chess_repr/              [Phase 4: Delete legacy module]
    └── [entire directory]
```

---

## Glossary

- **Orchestrator**: Root component managing application lifecycle and game flow
- **Board**: Shared state object containing position, renderer, and UI state
- **Player**: Trait abstraction for entities that provide moves (Human, AI, Network)
- **GameMode**: Enum representing current application state (Menu, PvP, PvAI, etc.)
- **POV (Point of View)**: Which color's perspective the board is rendered from
- **Arc<RefCell<>>**: Rust smart pointer pattern for shared mutable state
- **Perft**: Performance test for move generation (Permutation test)
- **FEN**: Forsyth-Edwards Notation, standard chess position format
- **PGN**: Portable Game Notation, standard game record format

---

## References

- [Original CLAUDE.md](./CLAUDE.md) - Project overview and current architecture
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Rust design patterns
- [Chess Programming Wiki](https://www.chessprogramming.org/) - Chess engine techniques
- [WGPU Documentation](https://docs.rs/wgpu/) - Graphics API reference

---

**Document Version**: 1.0
**Last Updated**: 2025-11-04
**Authors**: Claude Code + Eduardo
**Status**: Design Approved - Ready for Implementation
