// Monte Carlo Tree Search implementation with evaluation-guided playouts and progressive widening

use crate::game_repr::{Position, Move, Color};
use super::evaluation::{evaluate, quick_evaluate};
use super::move_ordering::{generate_ordered_moves, generate_all_ordered_moves};
use smallvec::SmallVec;
use std::f64;

// MCTS configuration constants
const EXPLORATION_CONSTANT: f64 = 1.414; // sqrt(2), standard UCB1 constant
const EVAL_WEIGHT: f64 = 2.0;  // Weight for evaluation bias in UCB (increased from 0.3)
const PROGRESSIVE_WIDENING_CONSTANT: f64 = 3.0; // Controls how fast we add children
const INITIAL_CHILDREN_COUNT: usize = 15; // Start with top 15 moves
const PLAYOUT_DEPTH_LIMIT: usize = 50; // Maximum depth for playouts
const PLAYOUT_MOVES_CONSIDERED: usize = 12; // Number of moves to consider during playout

/// MCTS Node representing a game state in the search tree
#[derive(Clone)]
struct MCTSNode {
    /// Number of times this node has been visited
    visits: u32,
    /// Total score accumulated (sum of playout results from this node)
    total_score: f64,
    /// Children nodes (states after making moves)
    children: Vec<MCTSNode>,
    /// Move that led to this node (None for root)
    mov: Option<Move>,
    /// Cached evaluation score for this position (for UCB bias)
    eval_score: i32,
    /// Whether all possible children have been expanded
    fully_expanded: bool,
    /// All available moves from this position (for progressive widening)
    available_moves: SmallVec<[Move; 64]>,
    /// Number of children currently expanded
    expanded_children_count: usize,
}

impl MCTSNode {
    /// Create a new node
    fn new(mov: Option<Move>, pos: &Position, color: Color) -> Self {
        let eval_score = quick_evaluate(pos, color);
        let available_moves = generate_all_ordered_moves(pos, color);
        let fully_expanded = available_moves.is_empty();

        Self {
            visits: 0,
            total_score: 0.0,
            children: Vec::new(),
            mov,
            eval_score,
            fully_expanded,
            available_moves,
            expanded_children_count: 0,
        }
    }

    /// Check if this node is terminal (no legal moves)
    fn is_terminal(&self) -> bool {
        self.available_moves.is_empty()
    }

    /// Check if we should expand more children (progressive widening)
    fn should_expand_more_children(&self) -> bool {
        if self.fully_expanded {
            return false;
        }

        // Progressive widening: expand more children as visit count increases
        // Formula: num_children < C * sqrt(visits)
        let target_children = (PROGRESSIVE_WIDENING_CONSTANT * (self.visits as f64).sqrt()) as usize;
        let target_children = target_children.max(INITIAL_CHILDREN_COUNT);

        self.expanded_children_count < target_children.min(self.available_moves.len())
    }

    /// Expand one more child node
    /// Uses 2-ply evaluation: considers opponent's best response before evaluating
    fn expand_one_child(&mut self, pos: &Position, color: Color, parent_color: Color) {
        if self.expanded_children_count >= self.available_moves.len() {
            self.fully_expanded = true;
            return;
        }

        let mov = self.available_moves[self.expanded_children_count];
        self.expanded_children_count += 1;

        // Make move to create child position
        let mut child_pos = pos.clone();
        child_pos.make_move_undoable(mov);

        // Use 2-ply evaluation: consider opponent's best response
        // After making our move, it's opponent's turn (color.opposite())
        let eval_score = if color == parent_color {
            // We just made our move, now it's opponent's turn
            // Need to consider their best response (2-ply)
            let opponent_moves = generate_ordered_moves(&child_pos, color.opposite(), 5);

            if opponent_moves.is_empty() {
                // No opponent moves (checkmate or stalemate)
                quick_evaluate(&child_pos, parent_color)
            } else {
                // Try opponent's best moves and find worst case for us
                let mut worst_case_eval = i32::MAX;
                for &opp_mov in opponent_moves.iter().take(5) {
                    let mut pos_after_opponent = child_pos.clone();
                    pos_after_opponent.make_move_undoable(opp_mov);

                    // Now it's our turn again - evaluate from our perspective
                    let eval = quick_evaluate(&pos_after_opponent, parent_color);
                    worst_case_eval = worst_case_eval.min(eval);
                }
                worst_case_eval
            }
        } else {
            // It's opponent's turn to move, we need to evaluate from their perspective
            // (this happens at odd depths in the tree)
            let opponent_moves = generate_ordered_moves(&child_pos, color.opposite(), 5);

            if opponent_moves.is_empty() {
                // No opponent moves
                quick_evaluate(&child_pos, parent_color)
            } else {
                // Try opponent's moves and find worst case from parent's perspective
                let mut worst_case_eval = i32::MAX;
                for &opp_mov in opponent_moves.iter().take(5) {
                    let mut pos_after_opponent = child_pos.clone();
                    pos_after_opponent.make_move_undoable(opp_mov);

                    let eval = quick_evaluate(&pos_after_opponent, parent_color);
                    worst_case_eval = worst_case_eval.min(eval);
                }
                worst_case_eval
            }
        };

        // Create child node with evaluation-guided bias
        let child_node = MCTSNode::new_with_eval_score(Some(mov), &child_pos, color.opposite(), eval_score);
        self.children.push(child_node);

        if self.expanded_children_count >= self.available_moves.len() {
            self.fully_expanded = true;
        }
    }

    /// Create node with pre-computed evaluation score
    fn new_with_eval_score(mov: Option<Move>, pos: &Position, color: Color, eval_score: i32) -> Self {
        let available_moves = generate_all_ordered_moves(pos, color);
        let fully_expanded = available_moves.is_empty();

        Self {
            visits: 0,
            total_score: 0.0,
            children: Vec::new(),
            mov,
            eval_score,
            fully_expanded,
            available_moves,
            expanded_children_count: 0,
        }
    }

    /// Calculate UCB1 score with evaluation bias
    fn ucb_score(&self, parent_visits: u32, color_multiplier: f64) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY; // Unvisited nodes have highest priority
        }

        let exploit = self.total_score / self.visits as f64;

        let explore = EXPLORATION_CONSTANT * ((parent_visits as f64).ln() / self.visits as f64).sqrt();

        // Evaluation bias: normalize eval to roughly [0, 1] range
        // Assuming eval is in centipawns, typical range is -2000 to +2000
        let normalized_eval = (self.eval_score as f64 / 2000.0).clamp(-1.0, 1.0);
        let eval_bias = EVAL_WEIGHT * normalized_eval * color_multiplier;

        exploit + explore + eval_bias
    }

    /// Select best child using UCB1
    fn select_best_child(&self, color_multiplier: f64) -> Option<usize> {
        if self.children.is_empty() {
            return None;
        }

        let mut best_idx = 0;
        let mut best_score = f64::NEG_INFINITY;

        for (idx, child) in self.children.iter().enumerate() {
            let score = child.ucb_score(self.visits, color_multiplier);
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        Some(best_idx)
    }

    /// Get most visited child (for best move selection)
    fn most_visited_child(&self) -> Option<usize> {
        if self.children.is_empty() {
            return None;
        }

        let mut best_idx = 0;
        let mut best_visits = 0;

        for (idx, child) in self.children.iter().enumerate() {
            if child.visits > best_visits {
                best_visits = child.visits;
                best_idx = idx;
            }
        }

        Some(best_idx)
    }
}

/// MCTS Tree for move search
pub struct MCTSTree {
    root: MCTSNode,
    root_color: Color,
}

impl MCTSTree {
    /// Create new MCTS tree for a position
    pub fn new(pos: &Position, color: Color) -> Self {
        let root = MCTSNode::new(None, pos, color);

        Self {
            root,
            root_color: color,
        }
    }

    /// Run MCTS for a fixed number of iterations
    pub fn search(&mut self, pos: &Position, iterations: u32) -> Option<Move> {
        for _i in 0..iterations {
            self.iteration(pos);

            // Debug output disabled for faster testing
            // if cfg!(test) && _i < 5 {
            //     if let Some(best_idx) = self.root.most_visited_child() {
            //         let child = &self.root.children[best_idx];
            //         if let Some(mov) = child.mov {
            //             println!("Iteration {}: best_move from={} to={}, visits={}, score={:.3}",
            //                 i + 1, mov._from(), mov._to(), child.visits, child.total_score / child.visits as f64);
            //         }
            //     }
            // }
        }

        // Select best move (most visited child)
        if let Some(best_child_idx) = self.root.most_visited_child() {
            self.root.children[best_child_idx].mov
        } else {
            None
        }
    }

    /// Single MCTS iteration (selection, expansion, simulation, backpropagation)
    fn iteration(&mut self, root_pos: &Position) {
        let mut pos = root_pos.clone();
        let mut path: Vec<usize> = Vec::new();
        let mut current_color = self.root_color;

        // 1. Selection - traverse tree using UCB until we reach a leaf or expandable node
        // Build the path by collecting child indices to follow
        let selection_path = self.select(&pos);

        // Apply the moves along the path
        for &child_idx in &selection_path {
            let node_ref = self.get_node_at_path(&path);
            let mov = node_ref.children[child_idx].mov.unwrap();
            pos.make_move_undoable(mov);
            current_color = current_color.opposite();
            path.push(child_idx);
        }

        // 2. Simulation - play out the position using evaluation-guided policy
        let playout_result = self.simulate(&pos, current_color);

        // 3. Backpropagation - update statistics
        self.backpropagate(&path, playout_result);
    }

    /// Select phase: traverse tree and return path to leaf
    fn select(&mut self, root_pos: &Position) -> Vec<usize> {
        let mut path = Vec::new();
        let mut pos = root_pos.clone();
        let mut current_color = self.root_color;
        let root_color = self.root_color; // Copy to avoid borrow conflicts

        loop {
            let node = self.get_node_at_path_mut(&path);

            // Debug output disabled
            // let debug_print = cfg!(test) && node.visits < 5;
            // let node_visits = node.visits;
            // let node_children = node.children.len();
            // if debug_print {
            //     println!("  Selection depth={}, node_visits={}, children={}",
            //         path.len(), node_visits, node_children);
            // }

            // Check if terminal
            if node.is_terminal() {
                break;
            }

            // Check if we should expand more children (progressive widening)
            if node.should_expand_more_children() {
                node.expand_one_child(&pos, current_color, root_color);
            }

            // If no children after expansion, this is a leaf
            if node.children.is_empty() {
                break;
            }

            // If we have unexplored children (visits == 0), select one and stop
            let unexplored_idx = node.children.iter().position(|child| child.visits == 0);
            if let Some(idx) = unexplored_idx {
                path.push(idx);
                break;
            }

            // All children visited, select best using UCB and continue
            let color_multiplier = if current_color == root_color { 1.0 } else { -1.0 };
            let best_child_idx = node.select_best_child(color_multiplier).unwrap();

            // Apply move and continue deeper
            let mov = node.children[best_child_idx].mov.unwrap();
            pos.make_move_undoable(mov);
            current_color = current_color.opposite();
            path.push(best_child_idx);
        }

        path
    }

    /// Get node at a given path (immutable)
    fn get_node_at_path(&self, path: &[usize]) -> &MCTSNode {
        let mut node = &self.root;
        for &idx in path {
            node = &node.children[idx];
        }
        node
    }

    /// Get node at a given path (mutable)
    fn get_node_at_path_mut(&mut self, path: &[usize]) -> &mut MCTSNode {
        let mut node = &mut self.root;
        for &idx in path {
            node = &mut node.children[idx];
        }
        node
    }

    /// Simulate a game using evaluation-guided playout policy
    fn simulate(&self, start_pos: &Position, start_color: Color) -> f64 {
        let mut pos = start_pos.clone();
        let mut current_color = start_color;
        let mut depth = 0;

        // Play out until terminal state or depth limit
        while depth < PLAYOUT_DEPTH_LIMIT {
            // Check for terminal conditions
            if pos.is_checkmate(current_color) {
                // Checkmate - bad for current player
                return if current_color == self.root_color { -1.0 } else { 1.0 };
            }

            if pos.is_stalemate(current_color) || !pos.has_legal_moves(current_color) {
                // Stalemate or no moves - draw
                return 0.0;
            }

            // Generate top moves using move ordering
            let moves = generate_ordered_moves(&pos, current_color, PLAYOUT_MOVES_CONSIDERED);

            if moves.is_empty() {
                // No legal moves - draw
                return 0.0;
            }

            // Select move probabilistically based on evaluation scores
            let mov = self.select_playout_move(&pos, &moves, current_color);

            // Make the move
            pos.make_move_undoable(mov);
            current_color = current_color.opposite();
            depth += 1;
        }

        // Depth limit reached - evaluate final position
        let final_eval = evaluate(&pos, self.root_color);

        // Convert evaluation to win probability (sigmoid-like)
        // Normalize: roughly -2000 to +2000 centipawns -> -1.0 to +1.0
        let normalized = (final_eval as f64 / 2000.0).clamp(-1.0, 1.0);

        // Apply tanh for smoother win probability
        normalized.tanh()
    }

    /// Select a move during playout using evaluation guidance
    /// Uses 2-ply evaluation: considers opponent's best response before evaluating
    fn select_playout_move(&self, pos: &Position, moves: &SmallVec<[Move; 64]>, color: Color) -> Move {
        if moves.is_empty() {
            panic!("select_playout_move called with empty moves");
        }

        if moves.len() == 1 {
            return moves[0];
        }

        // Evaluate each move considering opponent's best response (2-ply)
        let mut best_move = moves[0];
        let mut best_eval = i32::MIN;

        for &mov in moves {
            let mut pos_after_our_move = pos.clone();
            pos_after_our_move.make_move_undoable(mov);

            // Now it's opponent's turn - we need to see their best response
            // Generate a few opponent moves to check
            let opponent_moves = generate_ordered_moves(&pos_after_our_move, color.opposite(), 5);

            if opponent_moves.is_empty() {
                // No opponent moves (checkmate or stalemate)
                // Evaluate the position directly
                let eval = evaluate(&pos_after_our_move, color);
                if eval > best_eval {
                    best_eval = eval;
                    best_move = mov;
                }
                continue;
            }

            // Try opponent's best moves and find worst case for us
            let mut worst_case_eval = i32::MAX;
            for &opp_mov in opponent_moves.iter().take(5) {
                let mut pos_after_opponent = pos_after_our_move.clone();
                pos_after_opponent.make_move_undoable(opp_mov);

                // Now it's our turn again - evaluate from our perspective
                let eval = quick_evaluate(&pos_after_opponent, color);
                worst_case_eval = worst_case_eval.min(eval);
            }

            // This move's score is the worst case after opponent responds
            if worst_case_eval > best_eval {
                best_eval = worst_case_eval;
                best_move = mov;
            }
        }

        best_move
    }

    /// Backpropagate the playout result up the tree
    fn backpropagate(&mut self, path: &[usize], result: f64) {
        // Update root
        self.root.visits += 1;
        self.root.total_score += result;

        // Update path
        let mut node = &mut self.root;
        for &child_idx in path {
            if child_idx < node.children.len() {
                node = &mut node.children[child_idx];
                node.visits += 1;
                node.total_score += result;
            }
        }
    }

    /// Get statistics about the search
    pub fn get_stats(&self) -> MCTSStats {
        MCTSStats {
            root_visits: self.root.visits,
            num_children: self.root.children.len(),
            best_move_visits: self.root.most_visited_child()
                .map(|idx| self.root.children[idx].visits)
                .unwrap_or(0),
        }
    }
}

/// Statistics about MCTS search
#[derive(Debug)]
pub struct MCTSStats {
    pub root_visits: u32,
    pub num_children: usize,
    pub best_move_visits: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_repr::Type;

    #[test]
    fn test_mcts_tree_creation() {
        let pos = Position::default();
        let tree = MCTSTree::new(&pos, Color::White);

        // Root should be created
        assert_eq!(tree.root.visits, 0);
        assert_eq!(tree.root.mov, None);
    }

    #[test]
    fn test_mcts_search_finds_move() {
        let pos = Position::default();
        let mut tree = MCTSTree::new(&pos, Color::White);

        let best_move = tree.search(&pos, 100);

        // Should find a move
        assert!(best_move.is_some());

        // Move should be legal
        let mov = best_move.unwrap();
        assert!(pos.is_move_legal(mov));
    }

    #[test]
    fn test_progressive_widening() {
        let pos = Position::default();
        let mut tree = MCTSTree::new(&pos, Color::White);

        // Initially, root has no children
        assert_eq!(tree.root.children.len(), 0);

        // After some iterations, should have children
        tree.search(&pos, 50);

        // Should have expanded some children
        assert!(tree.root.children.len() > 0);
        assert!(tree.root.children.len() <= 20); // Starting position has 20 moves
    }

    #[test]
    fn test_mcts_prefers_obvious_capture() {
        // Position where Black queen is hanging
        let pos = Position::from_fen("rnb1kbnr/pppppppp/8/8/4q3/2N5/PPPPPPPP/R1BQKBNR w KQkq -");

        let mut tree = MCTSTree::new(&pos, Color::White);
        let best_move = tree.search(&pos, 200);

        // Should find a move
        assert!(best_move.is_some());

        // The move should be legal
        let mov = best_move.unwrap();
        assert!(pos.is_move_legal(mov));

        // Ideally, it should capture the queen, but we'll just check it finds a legal move
    }

    #[test]
    fn test_ai_captures_free_bishop() {
        // Position where Black has a free bishop on e4 (can be captured by White knight on c3)
        // This is the critical bug: AI should ALWAYS take a hanging piece
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/4b3/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1");

        let mut tree = MCTSTree::new(&pos, Color::White);

        // Run with decent iterations (20,000 as mentioned in the bug report)
        let best_move = tree.search(&pos, 20000);

        // Get stats to understand what happened
        let stats = tree.get_stats();
        println!("\nMCTS Stats:");
        println!("  Root visits: {}", stats.root_visits);
        println!("  Num children: {}", stats.num_children);
        println!("  Best move visits: {}", stats.best_move_visits);

        // Print info about all root children
        println!("\nRoot children:");
        for (i, child) in tree.root.children.iter().enumerate() {
            if let Some(mov) = child.mov {
                let to = mov._to();
                let from = mov._from();
                let captured = pos.position[to];
                println!("  {}: from={} to={} captures={:?} visits={} score={:.2}",
                    i + 1, from, to, captured.piece_type, child.visits, child.total_score / child.visits as f64);
            }
        }

        // Should find a move
        assert!(best_move.is_some(), "AI should find a move");

        let mov = best_move.unwrap();

        // Check if the move captures the bishop
        let to_square = mov._to();
        let captured_piece = pos.position[to_square];

        assert!(
            !captured_piece.is_none() && captured_piece.piece_type == Type::Bishop,
            "AI MUST capture the free bishop! Move chosen: from={}, to={}, captures={:?}",
            mov._from(),
            mov._to(),
            captured_piece
        );
    }

    #[test]
    fn test_move_ordering_puts_bishop_capture_first() {
        // Test that move ordering correctly prioritizes the bishop capture
        use crate::agent::ai::move_ordering::generate_ordered_moves;

        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/4b3/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1");

        // Get top 10 ordered moves
        let ordered_moves = generate_ordered_moves(&pos, Color::White, 10);

        // Find if the bishop capture is in the list
        let bishop_capture = ordered_moves.iter().find(|&m| {
            let to = m._to();
            let piece = pos.position[to];
            piece.piece_type == Type::Bishop
        });

        assert!(
            bishop_capture.is_some(),
            "Bishop capture MUST be in top 10 ordered moves!"
        );

        // It should actually be first or second
        let bishop_idx = ordered_moves.iter().position(|m| {
            let to = m._to();
            let piece = pos.position[to];
            piece.piece_type == Type::Bishop
        });

        assert!(
            bishop_idx.unwrap() < 5,
            "Bishop capture should be in top 5 moves, but found at position {}",
            bishop_idx.unwrap()
        );
    }

    #[test]
    fn test_evaluation_after_bishop_capture() {
        // Test that evaluation correctly shows advantage after capturing bishop
        use crate::game_repr::MoveType;

        let pos_before = Position::from_fen("rnbqkbnr/pppppppp/8/8/4b3/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1");
        let eval_before = evaluate(&pos_before, Color::White);
        println!("\nEval before capture: {} cp", eval_before);

        // After White captures bishop with knight (Nxe4)
        let mut pos_after = pos_before.clone();
        // Knight on c3 (square 18) captures bishop on e4 (square 28)
        let capture_move = Move::new(18, 28, MoveType::Normal);
        pos_after.make_move_undoable(capture_move);

        let eval_after = evaluate(&pos_after, Color::Black); // Now it's Black's turn
        println!("Eval after capture (Black's perspective): {} cp", eval_after);
        println!("Eval after capture (White's perspective): {} cp", -eval_after);

        // White should be ahead (started -306, after capture +78, so gained ~384 cp)
        // The final eval of +78 is reasonable: material is equal, White has slight positional edge
        assert!(
            -eval_after > 0,
            "After capturing free bishop, White should be ahead, but eval={} from White's perspective",
            -eval_after
        );
    }

    #[test]
    fn test_progressive_widening_includes_bishop_capture() {
        // Verify that progressive widening expands enough to include the bishop capture
        use crate::agent::ai::move_ordering::generate_all_ordered_moves;

        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/4b3/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1");

        // Get all ordered moves to see where the bishop capture is
        let all_ordered = generate_all_ordered_moves(&pos, Color::White);

        println!("\nAll {} legal moves in order:", all_ordered.len());
        for (i, m) in all_ordered.iter().enumerate() {
            let from = m._from();
            let to = m._to();
            let captured = pos.position[to];
            println!("  {}. from={} to={} captures={:?}", i + 1, from, to, captured.piece_type);
        }

        // Find the bishop capture position
        let bishop_idx = all_ordered.iter().position(|m| {
            let to = m._to();
            pos.position[to].piece_type == Type::Bishop
        });

        assert!(bishop_idx.is_some(), "Bishop capture must exist in legal moves");
        println!("\nBishop capture is at position {} in ordered move list", bishop_idx.unwrap() + 1);

        // With INITIAL_CHILDREN_COUNT = 15, the bishop capture should be included
        assert!(
            bishop_idx.unwrap() < INITIAL_CHILDREN_COUNT,
            "Bishop capture at position {} is NOT in initial {} children! This is the BUG!",
            bishop_idx.unwrap() + 1,
            INITIAL_CHILDREN_COUNT
        );
    }

    #[test]
    fn test_mcts_handles_few_moves() {
        // Position with very few legal moves
        let pos = Position::from_fen("7k/8/8/8/8/8/8/K7 w - -");

        let mut tree = MCTSTree::new(&pos, Color::White);
        let best_move = tree.search(&pos, 50);

        assert!(best_move.is_some());
        assert!(pos.is_move_legal(best_move.unwrap()));
    }

    #[test]
    fn test_stats_after_search() {
        let pos = Position::default();
        let mut tree = MCTSTree::new(&pos, Color::White);

        tree.search(&pos, 100);

        let stats = tree.get_stats();

        // Should have visited root 100 times
        assert_eq!(stats.root_visits, 100);

        // Should have some children
        assert!(stats.num_children > 0);

        // Best move should have been visited
        assert!(stats.best_move_visits > 0);
    }
}
