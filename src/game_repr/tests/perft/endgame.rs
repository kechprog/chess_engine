use super::*;

// ==================== ENDGAME POSITION PERFT TESTS ====================
// Position 3: Tests en passant and pawn promotion in endgame
// FEN: 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -

#[test]
fn test_perft_endgame_depth_1() {
    let pos = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8");
    assert_eq!(pos.perft(1), 14);
}

#[test]
fn test_perft_endgame_depth_2() {
    let pos = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8");
    assert_eq!(pos.perft(2), 191);
}

#[test]
fn test_perft_endgame_depth_3() {
    let pos = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8");
    assert_eq!(pos.perft(3), 2812);
}

#[test]
fn test_perft_endgame_depth_4() {
    let pos = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8");
    assert_eq!(pos.perft(4), 43238);
}

#[test]
fn test_perft_endgame_depth_5() {
    let pos = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8");
    assert_eq!(pos.perft(5), 674624);
}
