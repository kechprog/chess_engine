my hobby project to make a chess game with opengl in rust
and of course a chess engine backed

## Performance

The engine has been optimized for move generation performance:

**Current Performance:** 1.45s for perft depth 6 (82.8M nodes/sec)
**Baseline:** 20.83s (14.4× improvement)

**Optimizations Applied:**
- Bitboard representation with precomputed lookup tables
- Move list recycling to eliminate heap allocations
- Pin detection for bulk move validation
- Lazy move validation (skip validation for unpinned pieces when not in check)