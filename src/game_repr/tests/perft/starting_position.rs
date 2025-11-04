use super::*;

#[test]
fn test_perft_starting_position_depth_1() {
    let pos = Position::default();
    assert_eq!(pos.perft(1), 20);
}

#[test]
fn test_perft_starting_position_depth_2() {
    let pos = Position::default();
    assert_eq!(pos.perft(2), 400);
}

#[test]
fn test_perft_starting_position_depth_3() {
    let pos = Position::default();
    assert_eq!(pos.perft(3), 8902);
}

#[test]
fn test_perft_starting_position_depth_4() {
    let pos = Position::default();
    assert_eq!(pos.perft(4), 197281);
}

#[test]
fn test_perft_starting_position_depth_5() {
    let pos = Position::default();
    assert_eq!(pos.perft(5), 4865609);
}
