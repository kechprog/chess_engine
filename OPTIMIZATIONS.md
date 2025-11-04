# Chess Engine Performance Optimizations

**Date:** 2025-11-04
**Baseline:** 12.4s for perft depth 6 (~9.9M nodes/sec)
**Profiling Tool:** cargo flamegraph

## Flamegraph Analysis Summary

Memory allocation is the dominant bottleneck, consuming **10-15% of total CPU time**. The perft function recursively generates move lists at every node, and each call to `all_legal_moves()` allocates a new `Vec<Move>`. With depth 6 generating ~120 million nodes, this results in millions of heap allocations.

The bitboard move generation is **extremely efficient** - individual piece move functions barely register (<0.1% each). The overhead is almost entirely in:
1. Memory management (allocations/deallocations)
2. Move validation (is_square_attacked checks)
3. Move application/reversal overhead

## Hot Spots (Top 10 by CPU Time)

1. **Vec allocation/deallocation** - ~3-4% - Creating and destroying `Vec<Move>` for move lists - **HIGH priority**
   - `Vec::with_capacity` (~0.70% + others = ~2%)
   - `drop_in_place` for Vec/RawVec (~1.04% + 0.51% + 0.46% = ~2%)
   - `process_heap_alloc` (~0.75% + 0.69% + others = ~2%)

2. **Position::is_square_attacked** - ~1.5% - Checking if square is under attack - **MEDIUM priority**

3. **alloc::alloc::Global::grow_impl** - ~1% - Vector reallocation when capacity exceeded - **HIGH priority**

4. **Move::_from / Move::_to** - ~1.3% combined - Extracting move data from encoding - **LOW priority**

5. **Bitboards::occupied_by_color** - ~0.67% - Getting all pieces of color - **LOW priority**

6. **Bitboards::move_piece** - ~0.54% - Updating bitboard state - **LOW priority**

7. **legal_moves closure (filter)** - ~0.47% - Filtering pseudo-legal moves - **MEDIUM priority**

8. **Bitboards::pieces_of_type** - ~0.45% - Looking up pieces by type - **LOW priority**

9. **Position::make_move_undoable** - ~0.42% - Creating undo information - **MEDIUM priority**

10. **Position::mk_move** - ~0.23% - Executing the move - **LOW priority**

## Optimization Opportunities (Ranked by Impact)

### 1. [HIGH IMPACT] Move List Recycling with Fixed-Size Buffers
**Estimated Speedup:** 10-15%
**Status:** Not implemented
**Difficulty:** Medium

**Problem:** Current implementation allocates a new `Vec<Move>` for every position (position.rs:537-556). With 120M nodes at depth 6, this causes millions of heap allocations.

**Solution Options:**

#### Option A: Pre-allocated buffer passed down call stack
```rust
pub fn all_legal_moves_into(&self, moves: &mut Vec<Move>) {
    moves.clear();
    // Generate moves directly into provided buffer
}
```

#### Option B: Stack-allocated arrays (max ~218 moves possible)
```rust
pub struct MoveList {
    moves: [Move; 256],
    len: usize,
}
```

#### Option C: Hybrid - SmallVec for inline storage
```rust
use smallvec::SmallVec;
type MoveList = SmallVec<[Move; 64]>;
```

**Implementation Notes:**
- Need to update all_legal_moves() in position.rs:537-556
- Update legal_moves() for each piece type
- Modify perft() to reuse move buffer
- Ensure proper clearing between uses

**Files to Modify:**
- src/game_repr/position.rs (main changes)
- src/game_repr/piece_moves/*.rs (all piece move generators)
- All callers of all_legal_moves()

---

### 2. [HIGH IMPACT] Bulk Move Validation
**Estimated Speedup:** 5-10%
**Status:** Not implemented
**Difficulty:** High

**Problem:** Currently `legal_moves()` at position.rs:552 calls `is_square_attacked()` for every single move to check if king is left in check.

**Solution:**
1. Generate all pseudo-legal moves first (no validation)
2. Use pin detection to identify which pieces can/cannot move
3. Only validate moves from pinned pieces or king moves
4. Cache attack information across multiple move checks

**Technical Approach:**
- Compute pinned pieces bitboard at start
- For unpinned pieces, moves are automatically legal (if king not in check initially)
- For pinned pieces, only allow moves along pin ray
- King moves always need validation

**Implementation Notes:**
- Implement pin detection algorithm
- Add pinned pieces tracking to Position struct
- Modify move generation to use pin information
- Optimize for common case (most pieces unpinned)

**Files to Modify:**
- src/game_repr/position.rs (legal_moves, add pin detection)
- May need new bitboard helpers for pin rays

---

### 3. [MEDIUM IMPACT] Optimize is_square_attacked
**Estimated Speedup:** 3-5%
**Status:** Not implemented
**Difficulty:** Medium

**Problem:** Current implementation (position.rs:283-359) checks all piece types sequentially. Called very frequently during move validation.

**Solution:**
1. Add early termination after first attacker found
2. Check most likely attackers first (pawns, knights more common than queens)
3. Consider implementing magic bitboards for sliding pieces
4. Cache attack maps for frequently checked squares (especially king positions)

**Reordering by Likelihood:**
```rust
// Check in order of commonality:
// 1. Pawns (most common)
// 2. Knights
// 3. King
// 4. Bishops/Queens on diagonals
// 5. Rooks/Queens on orthogonals
```

**Implementation Notes:**
- Add early return `return true` as soon as attacker found
- Reorder checks by statistical likelihood
- Consider separate fast path for king safety checks
- Profile to verify improvement

**Files to Modify:**
- src/game_repr/position.rs (is_square_attacked function)

---

### 4. [MEDIUM IMPACT] Use SmallVec for Move Lists
**Estimated Speedup:** 2-4%
**Status:** Not implemented
**Difficulty:** Low

**Problem:** Most positions have 30-40 legal moves, but Vec always heap-allocates.

**Solution:**
Use `SmallVec<[Move; 64]>` to store moves inline for typical positions:

```rust
use smallvec::{SmallVec, smallvec};
type MoveList = SmallVec<[Move; 64]>;

pub fn all_legal_moves(&self) -> MoveList {
    let mut moves = MoveList::new();
    // ...
}
```

**Trade-offs:**
- Typical position: 0 heap allocations (30-40 moves fit inline)
- Worst case (218 moves): 1 heap allocation (same as current)
- Stack usage: 64 * 2 bytes = 128 bytes per call

**Implementation Notes:**
- Add smallvec dependency to Cargo.toml
- Change return type of all_legal_moves() and related functions
- May conflict with Option #1 (choose one approach)

**Files to Modify:**
- Cargo.toml (add dependency)
- src/game_repr/position.rs
- All callers of move generation functions

---

### 5. [MEDIUM IMPACT] Reduce Position::clone Overhead in perft
**Estimated Speedup:** 2-3%
**Status:** Not implemented
**Difficulty:** Low

**Problem:** Line 707 in position.rs clones the entire position at each depth level. Clone includes:
- Full bitboard state (12 × u64)
- 64-element mailbox array
- prev_moves Vec (grows during game)

**Solution:**
Verify that make_move_undoable/unmake_move pattern is used exclusively and eliminate unnecessary clones.

**Alternative:** If clones are necessary, implement custom Clone that only copies essential data:
```rust
impl Clone for Position {
    fn clone(&self) -> Self {
        Position {
            position: self.position,
            bitboards: self.bitboards,
            castling_cond: self.castling_cond,
            prev_moves: Vec::new(), // Don't clone move history for perft
        }
    }
}
```

**Implementation Notes:**
- Audit all uses of Position::clone()
- Ensure make_move_undoable/unmake_move fully restores state
- Consider specialized "clone for search" method

**Files to Modify:**
- src/game_repr/position.rs

---

### 6. [LOW IMPACT] Inline Hot Functions
**Estimated Speedup:** 1-2%
**Status:** Not implemented
**Difficulty:** Very Low

**Problem:** Frequently called small functions have call overhead.

**Solution:**
Add `#[inline(always)]` to hot functions:

```rust
#[inline(always)]
pub fn _from(&self) -> u8 { ... }

#[inline(always)]
pub fn _to(&self) -> u8 { ... }

#[inline(always)]
pub fn move_type(&self) -> MoveType { ... }

#[inline(always)]
pub fn occupied_by_color(&self, color: Color) -> u64 { ... }

#[inline(always)]
pub fn pieces_of_type(&self, color: Color, piece_type: Type) -> u64 { ... }
```

**Implementation Notes:**
- Start with conservative `#[inline]` first
- Profile to verify improvement
- Only use `#[inline(always)]` if proven beneficial
- May increase binary size slightly

**Files to Modify:**
- src/game_repr/moves.rs (Move methods)
- src/game_repr/bitboards/mod.rs (Bitboards methods)

---

### 7. [LOW IMPACT] Optimize unmake_move
**Estimated Speedup:** 1-2%
**Status:** Not implemented
**Difficulty:** Medium

**Problem:** unmake_move (position.rs:587-613) does redundant work reconstructing state.

**Solution:**
Store more state in UndoInfo to avoid recalculation:
- Captured piece location and type
- Previous castling conditions
- Bitboard deltas instead of full reconstruction

**Alternative:** Batch bitboard updates instead of individual operations.

**Implementation Notes:**
- Expand UndoInfo struct
- Measure memory vs speed tradeoff
- Ensure correctness with tests

**Files to Modify:**
- src/game_repr/position.rs (UndoInfo, unmake_move)

---

## Implementation Priority

**Phase 1 (Target: 20-25% total improvement)**
1. Move list recycling (10-15%)
2. Bulk move validation (5-10%)

**Phase 2 (Target: additional 5-10%)**
3. Optimize is_square_attacked (3-5%)
4. Inline hot functions (1-2%)

**Phase 3 (Diminishing returns)**
5. Use SmallVec (2-4%) - May be superseded by #1
6. Reduce clone overhead (2-3%)
7. Optimize unmake_move (1-2%)

## Benchmarking Protocol

After each optimization:
1. Run `cargo test` to verify correctness (all 101 tests must pass)
2. Run perft benchmark 3 times: `cargo run --profile profiling --example perft_benchmark`
3. Calculate average time and nodes/sec
4. Compare with baseline: 12.4s, 9.9M nodes/sec
5. Record results in this file
6. Git commit with performance results

## Expected Final Performance

With all high-impact optimizations (1-3): **~9-10 seconds** (14-16M nodes/sec)
With all optimizations implemented: **~8-9 seconds** (16-18M nodes/sec)

This would represent a **~60% improvement** over the current 12.4s baseline, and **~160% improvement** over the original 20.83s baseline.

## Notes

- Profile after each change to verify improvement
- Don't optimize prematurely - measure first
- Maintain test coverage (all 101 tests must pass)
- Consider correctness over speed in ambiguous cases
- Document any tradeoffs made
