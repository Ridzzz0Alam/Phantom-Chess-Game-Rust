//! Scenario tests for the special rules: castling, en passant, promotion,
//! checkmate, stalemate, and the `MoveError` reasons.

use chess_core::{CastleSide, Color, Move, MoveError, Piece, PieceKind, Position, Square};

fn pos(fen: &str) -> Position {
    Position::try_from(fen).unwrap_or_else(|e| panic!("bad test FEN {fen:?}: {e}"))
}
fn sq(s: &str) -> Square {
    Square::try_from(s).unwrap()
}
fn mv(s: &str) -> Move {
    Move::try_from(s).unwrap()
}
fn is_legal(p: &Position, m: &str) -> bool {
    p.legal_moves().contains(&mv(m))
}
fn play(p: &mut Position, moves: &[&str]) {
    for m in moves {
        p.make_move(mv(m))
            .unwrap_or_else(|e| panic!("{m} should be legal: {e}"));
    }
}

// ---------- castling ----------

#[test]
fn castling_both_sides_when_path_is_clear() {
    let p = pos("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    assert!(is_legal(&p, "e1g1"));
    assert!(is_legal(&p, "e1c1"));
}

#[test]
fn castling_moves_the_rook_and_clears_rights() {
    let mut p = pos("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    play(&mut p, &["e1g1"]);
    assert_eq!(
        p.piece_at(sq("g1")),
        Some(Piece::new(Color::White, PieceKind::King))
    );
    assert_eq!(
        p.piece_at(sq("f1")),
        Some(Piece::new(Color::White, PieceKind::Rook))
    );
    assert_eq!(p.piece_at(sq("h1")), None);
    assert_eq!(p.castling().to_string(), "kq");
    play(&mut p, &["e8c8"]);
    assert_eq!(
        p.piece_at(sq("d8")),
        Some(Piece::new(Color::Black, PieceKind::Rook))
    );
    assert_eq!(p.castling().to_string(), "-");
}

#[test]
fn castling_through_attacked_square_is_illegal() {
    // Black rook on f8 attacks f1: king would pass through it.
    let p = pos("4kr2/8/8/8/8/8/8/R3K2R w KQ - 0 1");
    assert!(!is_legal(&p, "e1g1"));
    assert!(is_legal(&p, "e1c1"));
    assert_eq!(
        p.check_move(mv("e1g1")),
        Err(MoveError::IllegalMove(mv("e1g1")))
    );
}

#[test]
fn castling_into_check_is_illegal() {
    // Black rook on g8 attacks g1.
    let p = pos("4k1r1/8/8/8/8/8/8/R3K2R w KQ - 0 1");
    assert!(!is_legal(&p, "e1g1"));
}

#[test]
fn castling_out_of_check_is_illegal() {
    let p = pos("4r1k1/8/8/8/8/8/8/R3K2R w KQ - 0 1");
    assert!(p.in_check());
    assert!(!is_legal(&p, "e1g1"));
    assert!(!is_legal(&p, "e1c1"));
}

#[test]
fn queenside_castling_allowed_when_only_b_file_is_attacked() {
    // Rook on b8 attacks b1, which the king never crosses.
    let p = pos("1r2k3/8/8/8/8/8/8/R3K3 w Q - 0 1");
    assert!(is_legal(&p, "e1c1"));
}

#[test]
fn castling_blocked_by_piece_in_between() {
    let p = pos("4k3/8/8/8/8/8/8/RN2K1NR w KQ - 0 1");
    assert!(!is_legal(&p, "e1g1"));
    assert!(!is_legal(&p, "e1c1"));
}

#[test]
fn rook_move_loses_only_that_sides_right() {
    let mut p = pos("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    play(&mut p, &["h1h2"]);
    assert!(!p.castling().has(Color::White, CastleSide::KingSide));
    assert!(p.castling().has(Color::White, CastleSide::QueenSide));
}

#[test]
fn capturing_a_rook_removes_opponents_right() {
    let mut p = pos("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    play(&mut p, &["a1a8"]);
    assert_eq!(p.castling().to_string(), "Kk");
}

// ---------- en passant ----------

#[test]
fn en_passant_capture_removes_the_pawn() {
    let mut p = Position::startpos();
    play(&mut p, &["e2e4", "a7a6", "e4e5", "d7d5"]);
    assert_eq!(p.en_passant(), Some(sq("d6")));
    play(&mut p, &["e5d6"]);
    assert_eq!(p.piece_at(sq("d5")), None);
    assert_eq!(
        p.piece_at(sq("d6")),
        Some(Piece::new(Color::White, PieceKind::Pawn))
    );
    assert_eq!(p.halfmove_clock(), 0);
}

#[test]
fn en_passant_only_on_the_very_next_move() {
    let mut p = Position::startpos();
    play(&mut p, &["e2e4", "a7a6", "e4e5", "d7d5", "h2h3", "h7h6"]);
    assert_eq!(p.en_passant(), None);
    assert!(!is_legal(&p, "e5d6"));
}

#[test]
fn en_passant_that_exposes_king_on_rank_is_illegal() {
    // Taking c5 would remove both pawns from rank 5, exposing the king to the rook.
    let p = pos("8/8/8/KPp4r/8/8/8/7k w - c6 0 2");
    assert_eq!(
        p.check_move(mv("b5c6")),
        Err(MoveError::LeavesKingInCheck(mv("b5c6")))
    );
}

// ---------- promotion ----------

#[test]
fn all_four_promotions_are_generated_and_work() {
    let p = pos("8/P6k/8/8/8/8/8/K7 w - - 0 1");
    for (m, kind) in [
        ("a7a8q", PieceKind::Queen),
        ("a7a8r", PieceKind::Rook),
        ("a7a8b", PieceKind::Bishop),
        ("a7a8n", PieceKind::Knight),
    ] {
        let mut next = p.clone();
        play(&mut next, &[m]);
        assert_eq!(
            next.piece_at(sq("a8")),
            Some(Piece::new(Color::White, kind))
        );
    }
}

#[test]
fn promotion_by_capture() {
    let mut p = pos("1r5k/P7/8/8/8/8/8/K7 w - - 0 1");
    play(&mut p, &["a7b8n"]);
    assert_eq!(
        p.piece_at(sq("b8")),
        Some(Piece::new(Color::White, PieceKind::Knight))
    );
}

#[test]
fn promotion_without_piece_is_rejected() {
    let p = pos("8/P6k/8/8/8/8/8/K7 w - - 0 1");
    assert_eq!(p.check_move(mv("a7a8")), Err(MoveError::MissingPromotion));
}

#[test]
fn promotion_letter_on_normal_move_is_illegal() {
    let p = Position::startpos();
    assert_eq!(
        p.check_move(mv("e2e4q")),
        Err(MoveError::IllegalMove(mv("e2e4q")))
    );
}

// ---------- mate / stalemate ----------

#[test]
fn fools_mate_is_checkmate() {
    let mut p = Position::startpos();
    play(&mut p, &["f2f3", "e7e5", "g2g4", "d8h4"]);
    assert!(p.in_check());
    assert!(p.legal_moves().is_empty());
}

#[test]
fn known_stalemate() {
    let p = pos("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");
    assert!(!p.in_check());
    assert!(p.legal_moves().is_empty());
}

// ---------- move errors ----------

#[test]
fn move_errors_explain_the_reason() {
    let p = Position::startpos();
    assert_eq!(
        p.check_move(mv("e4e5")),
        Err(MoveError::NoPieceOnSquare(sq("e4")))
    );
    assert_eq!(
        p.check_move(mv("e7e5")),
        Err(MoveError::NotYourPiece(sq("e7")))
    );
    assert_eq!(
        p.check_move(mv("e2e5")),
        Err(MoveError::IllegalMove(mv("e2e5")))
    );
}

#[test]
fn moving_a_pinned_piece_leaves_king_in_check() {
    // Bishop on b4 pins the d2 knight to the e1 king.
    let p = pos("4k3/8/8/8/1b6/8/3N4/4K3 w - - 0 1");
    assert_eq!(
        p.check_move(mv("d2f3")),
        Err(MoveError::LeavesKingInCheck(mv("d2f3")))
    );
}

#[test]
fn rejected_move_leaves_position_unchanged() {
    let mut p = Position::startpos();
    let before = p.clone();
    assert!(p.make_move(mv("e2e5")).is_err());
    assert_eq!(p, before);
}

#[test]
fn clocks_update() {
    let mut p = Position::startpos();
    play(&mut p, &["g1f3", "g8f6"]);
    assert_eq!((p.halfmove_clock(), p.fullmove_number()), (2, 2));
    play(&mut p, &["e2e4"]);
    assert_eq!(p.halfmove_clock(), 0);
    assert_eq!(p.side_to_move(), Color::Black);
}
