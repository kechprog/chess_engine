use super::*;

// ==================== MIDDLE GAME WITH PROMOTION PERFT TESTS ====================
// Position 5: Tests immediate promotion scenarios
// FEN: rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8

#[test]
fn test_perft_middle_game_promotion_depth_1() {
    let pos = Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R");
    assert_eq!(pos.perft(1), 44);
}

#[test]
fn test_perft_middle_game_promotion_depth_2() {
    let pos = Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R");
    assert_eq!(pos.perft(2), 1486);
}

#[test]
fn test_perft_middle_game_promotion_depth_3() {
    let pos = Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R");
    assert_eq!(pos.perft(3), 62379);
}

#[test]
fn test_perft_middle_game_promotion_depth_4() {
    let pos = Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R");
    assert_eq!(pos.perft(4), 2103487);
}

#[test]
fn test_perft_middle_game_promotion_depth_5() {
    let pos = Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R");
    assert_eq!(pos.perft(5), 89941194);
}
