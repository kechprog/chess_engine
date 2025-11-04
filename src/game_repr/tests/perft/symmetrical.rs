use super::*;

// ==================== SYMMETRICAL MIDDLE GAME PERFT TESTS ====================
// Position 6: Tests complex tactical positions with pins
// FEN: r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10

#[test]
fn test_perft_symmetrical_depth_1() {
    let pos = Position::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1");
    assert_eq!(pos.perft(1), 46);
}

#[test]
fn test_perft_symmetrical_depth_2() {
    let pos = Position::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1");
    assert_eq!(pos.perft(2), 2079);
}

#[test]
fn test_perft_symmetrical_depth_3() {
    let pos = Position::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1");
    assert_eq!(pos.perft(3), 89890);
}

#[test]
fn test_perft_symmetrical_depth_4() {
    let pos = Position::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1");
    assert_eq!(pos.perft(4), 3894594);
}

#[test]
fn test_perft_symmetrical_depth_5() {
    let pos = Position::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1");
    assert_eq!(pos.perft(5), 164075551);
}
