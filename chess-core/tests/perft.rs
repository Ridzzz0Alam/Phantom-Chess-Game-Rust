//! Perft: count leaf nodes of the legal move tree and compare with known values.
//! Each line below becomes its own `#[test]` via the `perft_tests!` macro.
//! Depth-4 tests are `#[ignore]`d; run them with
//! `cargo test -p chess-core --release -- --ignored`.

use chess_core::perft_tests;

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
const POSITION_6: &str = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

perft_tests! {
    startpos_d1: STARTPOS, 1 => 20;
    startpos_d2: STARTPOS, 2 => 400;
    startpos_d3: STARTPOS, 3 => 8_902;
    #[ignore = "slow in debug; run with --release -- --ignored"]
    startpos_d4: STARTPOS, 4 => 197_281;

    kiwipete_d1: KIWIPETE, 1 => 48;
    kiwipete_d2: KIWIPETE, 2 => 2_039;
    kiwipete_d3: KIWIPETE, 3 => 97_862;
    #[ignore = "slow in debug; run with --release -- --ignored"]
    kiwipete_d4: KIWIPETE, 4 => 4_085_603;

    position_3_d1: POSITION_3, 1 => 14;
    position_3_d2: POSITION_3, 2 => 191;
    position_3_d3: POSITION_3, 3 => 2_812;
    #[ignore = "slow in debug; run with --release -- --ignored"]
    position_3_d4: POSITION_3, 4 => 43_238;

    position_4_d1: POSITION_4, 1 => 6;
    position_4_d2: POSITION_4, 2 => 264;
    position_4_d3: POSITION_4, 3 => 9_467;
    #[ignore = "slow in debug; run with --release -- --ignored"]
    position_4_d4: POSITION_4, 4 => 422_333;

    position_5_d1: POSITION_5, 1 => 44;
    position_5_d2: POSITION_5, 2 => 1_486;
    position_5_d3: POSITION_5, 3 => 62_379;
    #[ignore = "slow in debug; run with --release -- --ignored"]
    position_5_d4: POSITION_5, 4 => 2_103_487;

    position_6_d1: POSITION_6, 1 => 46;
    position_6_d2: POSITION_6, 2 => 2_079;
    position_6_d3: POSITION_6, 3 => 89_890;
}
