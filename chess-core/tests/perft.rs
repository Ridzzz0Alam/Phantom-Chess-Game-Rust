//! Perft: count leaf nodes of the legal move tree and compare with known values.
//! Depth-4 tests are `#[ignore]`d; run them with
//! `cargo test -p chess-core --release -- --ignored`.

use chess_core::Position;

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
const POSITION_6: &str = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

fn perft(fen: &str, depth: u32) -> u64 {
    match Position::try_from(fen) {
        Ok(p) => p.perft(depth),
        Err(e) => panic!("bad test FEN {fen:?}: {e}"),
    }
}

#[test]
fn startpos_d1_to_d3() {
    assert_eq!(perft(STARTPOS, 1), 20);
    assert_eq!(perft(STARTPOS, 2), 400);
    assert_eq!(perft(STARTPOS, 3), 8_902);
}

#[test]
#[ignore = "slow in debug; run with --release -- --ignored"]
fn startpos_d4() {
    assert_eq!(perft(STARTPOS, 4), 197_281);
}

#[test]
fn kiwipete_d1_to_d3() {
    assert_eq!(perft(KIWIPETE, 1), 48);
    assert_eq!(perft(KIWIPETE, 2), 2_039);
    assert_eq!(perft(KIWIPETE, 3), 97_862);
}

#[test]
#[ignore = "slow in debug; run with --release -- --ignored"]
fn kiwipete_d4() {
    assert_eq!(perft(KIWIPETE, 4), 4_085_603);
}

#[test]
fn position_3_d1_to_d3() {
    assert_eq!(perft(POSITION_3, 1), 14);
    assert_eq!(perft(POSITION_3, 2), 191);
    assert_eq!(perft(POSITION_3, 3), 2_812);
}

#[test]
#[ignore = "slow in debug; run with --release -- --ignored"]
fn position_3_d4() {
    assert_eq!(perft(POSITION_3, 4), 43_238);
}

#[test]
fn position_4_d1_to_d3() {
    assert_eq!(perft(POSITION_4, 1), 6);
    assert_eq!(perft(POSITION_4, 2), 264);
    assert_eq!(perft(POSITION_4, 3), 9_467);
}

#[test]
#[ignore = "slow in debug; run with --release -- --ignored"]
fn position_4_d4() {
    assert_eq!(perft(POSITION_4, 4), 422_333);
}

#[test]
fn position_5_d1_to_d3() {
    assert_eq!(perft(POSITION_5, 1), 44);
    assert_eq!(perft(POSITION_5, 2), 1_486);
    assert_eq!(perft(POSITION_5, 3), 62_379);
}

#[test]
#[ignore = "slow in debug; run with --release -- --ignored"]
fn position_5_d4() {
    assert_eq!(perft(POSITION_5, 4), 2_103_487);
}

#[test]
fn position_6_d1_to_d3() {
    assert_eq!(perft(POSITION_6, 1), 46);
    assert_eq!(perft(POSITION_6, 2), 2_079);
    assert_eq!(perft(POSITION_6, 3), 89_890);
}
