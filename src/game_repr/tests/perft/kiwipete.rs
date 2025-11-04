use super::*;

// ==================== KIWIPETE PERFT TESTS ====================
// Position 2: Tests castling, en passant, promotions
// FEN: r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -

#[test]
fn test_perft_kiwipete_depth_1() {
    let pos = Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R");
    assert_eq!(pos.perft(1), 48);
}

#[test]
fn test_perft_kiwipete_depth_2() {
    let pos = Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R");
    assert_eq!(pos.perft(2), 2039);
}

#[test]
fn test_perft_kiwipete_depth_3() {
    let pos = Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R");
    assert_eq!(pos.perft(3), 97862);
}

#[test]
fn test_perft_kiwipete_depth_4() {
    let pos = Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R");
    assert_eq!(pos.perft(4), 4085603);
}

#[test]
fn test_perft_kiwipete_depth_5() {
    let pos = Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R");
    assert_eq!(pos.perft(5), 193690690);
}
