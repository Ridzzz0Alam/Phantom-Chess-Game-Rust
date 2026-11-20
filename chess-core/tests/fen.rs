//! FEN round-trip and error-variant tests.

use chess_core::{CastleSide, Color, FenError, Position, STARTPOS_FEN, Square};

fn parse(fen: &str) -> Position {
    Position::try_from(fen).unwrap_or_else(|e| panic!("{fen:?} should parse: {e}"))
}

#[test]
fn startpos_round_trips() {
    assert_eq!(parse(STARTPOS_FEN).to_fen(), STARTPOS_FEN);
}

#[test]
fn valid_fens_round_trip() {
    let fens = [
        // Kiwipete
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        // Perft position 3
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        // Perft position 4
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        // Perft position 5
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        // Perft position 6
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        // After 1. e4: en-passant square, Black to move
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        // After 1. e4 d5 2. e5 f5: White may capture en passant on f6
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        // Bare kings, large counters
        "8/8/8/4k3/8/8/8/4K3 b - - 99 300",
    ];
    for fen in fens {
        assert_eq!(parse(fen).to_fen(), fen);
    }
}

#[test]
fn parsed_fields_are_correct() {
    let p = parse("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    assert_eq!(p.side_to_move(), Color::White);
    assert_eq!(p.en_passant(), Square::try_from("f6").ok());
    assert_eq!(p.fullmove_number(), 3);

    let p = parse("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    assert!(!p.castling().has(Color::White, CastleSide::KingSide));
    assert!(p.castling().has(Color::Black, CastleSide::QueenSide));
}

fn err(fen: &str) -> FenError {
    match Position::try_from(fen) {
        Ok(_) => panic!("{fen:?} should be rejected"),
        Err(e) => e,
    }
}

#[test]
fn wrong_field_count() {
    assert_eq!(err(""), FenError::WrongFieldCount(0));
    assert_eq!(
        err("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -"),
        FenError::WrongFieldCount(4)
    );
    assert_eq!(
        err(&format!("{STARTPOS_FEN} extra")),
        FenError::WrongFieldCount(7)
    );
}

#[test]
fn bad_board() {
    let cases = [
        // 7 ranks
        "rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // rank with 9 squares
        "rnbqkbnr/ppppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // rank with 7 squares
        "rnbqkbnr/ppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // digit 9
        "rnbqkbnr/pppppppp/9/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // digit 0
        "rnbqkbnr/pppppppp/08/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // unknown piece letter
        "rnbqkbnr/ppppxppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // no white king
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQQBNR w kq - 0 1",
        // two black kings
        "rnbkkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1",
        // pawn on the 8th rank
        "Pnbqkbnr/pppppppp/8/8/8/8/1PPPPPPP/RNBQKBNR w KQk - 0 1",
    ];
    for fen in cases {
        assert!(matches!(err(fen), FenError::BadBoard(_)), "{fen}");
    }
}

#[test]
fn bad_piece_letter_message_comes_from_parse_error() {
    assert_eq!(
        err("rnbqkbnr/ppppxppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        FenError::BadBoard("invalid piece letter 'x'".into())
    );
}

#[test]
fn bad_side() {
    for side in ["x", "W", "white"] {
        let fen = format!("4k3/8/8/8/8/8/8/4K3 {side} - - 0 1");
        assert_eq!(err(&fen), FenError::BadSide(side.into()));
    }
}

#[test]
fn bad_castling() {
    let base = "r3k2r/8/8/8/8/8/8/R3K2R w";
    for c in ["KQkx", "KK", "kQ-", "--"] {
        let fen = format!("{base} {c} - 0 1");
        assert_eq!(err(&fen), FenError::BadCastling(c.into()), "{fen}");
    }
    // Right claimed but king not on e1.
    assert_eq!(
        err("r3k2r/8/8/8/8/8/8/R4K1R w K - 0 1"),
        FenError::BadCastling("K".into())
    );
    // Right claimed but rook missing from a8.
    assert_eq!(
        err("4k2r/8/8/8/8/8/8/R3K2R w q - 0 1"),
        FenError::BadCastling("q".into())
    );
}

#[test]
fn bad_en_passant() {
    let cases = [
        // not a square
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e9 0 1",
        // wrong rank for side to move
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e6 0 1",
        // right rank, but no pawn just pushed there
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq d3 0 1",
        // White to move, e6 claimed but no black pawn on e5
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e6 0 1",
    ];
    for fen in cases {
        assert!(matches!(err(fen), FenError::BadEnPassant(_)), "{fen}");
    }
}

#[test]
fn bad_numbers() {
    let base = "4k3/8/8/8/8/8/8/4K3 w - -";
    for (half, full, bad) in [
        ("-1", "1", "-1"),
        ("x", "1", "x"),
        ("0", "0", "0"),
        ("0", "70000", "70000"),
    ] {
        let fen = format!("{base} {half} {full}");
        assert_eq!(err(&fen), FenError::BadNumber(bad.into()), "{fen}");
    }
}

#[test]
fn from_str_matches_try_from() {
    assert_eq!(STARTPOS_FEN.parse::<Position>(), Ok(Position::startpos()));
}
