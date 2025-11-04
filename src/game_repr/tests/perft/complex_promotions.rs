use super::*;

// ==================== COMPLEX POSITION WITH PROMOTIONS PERFT TESTS ====================
// Position 4: Tests promotion captures and underpromotion
// FEN: r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1

#[test]
fn test_perft_complex_promotions_depth_1() {
    let pos = Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1");
    assert_eq!(pos.perft(1), 6);
}

#[test]
fn test_perft_complex_promotions_depth_2() {
    let pos = Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1");
    assert_eq!(pos.perft(2), 264);
}

#[test]
fn test_perft_complex_promotions_depth_3() {
    let pos = Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1");
    assert_eq!(pos.perft(3), 9467);
}

#[test]
fn test_perft_complex_promotions_depth_4() {
    let pos = Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1");
    assert_eq!(pos.perft(4), 422333);
}

#[test]
fn test_perft_complex_promotions_depth_5() {
    let pos = Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1");
    assert_eq!(pos.perft(5), 15833292);
}
