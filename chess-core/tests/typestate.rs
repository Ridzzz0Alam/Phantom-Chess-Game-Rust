//! The phantom-type game layer: typed play, game-over detection, and `AnyGame`.
//! (Compile-time guarantees are proven by the `compile_fail` doc tests in `game.rs`.)

use chess_core::{
    AnyGame, BlackToMove, Color, DrawReason, Game, GameResult, Move, MoveError, Played, Position,
    WhiteToMove, WinReason,
};

fn mv(s: &str) -> Move {
    Move::try_from(s).unwrap()
}

/// Plays a move that must not end the game, returning the next typed game.
fn cont<S: chess_core::Side>(game: Game<S>, m: &str) -> Game<S::Next> {
    match game.play(mv(m)) {
        Ok(Played::Ongoing(next)) => next,
        Ok(Played::Finished(f)) => panic!("{m} unexpectedly ended the game: {}", f.result()),
        Err((_, e)) => panic!("{m} rejected: {e}"),
    }
}

#[test]
fn fools_mate_through_typed_games() {
    let g0: Game<WhiteToMove> = Game::new();
    let g1: Game<BlackToMove> = cont(g0, "f2f3");
    let g2: Game<WhiteToMove> = cont(g1, "e7e5");
    let g3: Game<BlackToMove> = cont(g2, "g2g4");
    let finished = match g3.play(mv("d8h4")) {
        Ok(Played::Finished(f)) => f,
        _ => panic!("Qh4# should finish the game"),
    };
    assert_eq!(
        finished.result(),
        GameResult::BlackWins(WinReason::Checkmate)
    );
    assert_eq!(finished.history().len(), 4);
}

#[test]
fn illegal_move_hands_the_game_back_unchanged() {
    let game = Game::new();
    let before = game.position().clone();
    let (game, err) = match game.play(mv("e2e5")) {
        Err(pair) => pair,
        Ok(_) => panic!("e2e5 must be illegal"),
    };
    assert_eq!(err, MoveError::IllegalMove(mv("e2e5")));
    assert_eq!(game.position(), &before);
    assert!(game.history().is_empty());
    // The returned game is still usable and still White's.
    let next: Game<BlackToMove> = cont(game, "e2e4");
    assert_eq!(next.side(), Color::Black);
}

#[test]
fn typed_side_matches_position() {
    let g = Game::new();
    assert_eq!(g.side(), g.position().side_to_move());
    let g = cont(g, "d2d4");
    assert_eq!(g.side(), g.position().side_to_move());
}

fn play_any(mut game: AnyGame, moves: &[&str]) -> AnyGame {
    for m in moves {
        game = match game.play(mv(m)) {
            Ok(next) => next,
            Err((_, e)) => panic!("{m} rejected: {e}"),
        };
    }
    game
}

#[test]
fn any_game_alternates_sides() {
    let g = AnyGame::new();
    assert_eq!(g.side_to_move(), Some(Color::White));
    let g = play_any(g, &["e2e4"]);
    assert!(matches!(g, AnyGame::Black(_)));
    let g = play_any(g, &["c7c5"]);
    assert!(matches!(g, AnyGame::White(_)));
    assert_eq!(g.history(), &[mv("e2e4"), mv("c7c5")]);
}

#[test]
fn any_game_rejects_moves_after_game_over() {
    let g = play_any(AnyGame::new(), &["f2f3", "e7e5", "g2g4", "d8h4"]);
    assert_eq!(
        g.result(),
        Some(GameResult::BlackWins(WinReason::Checkmate))
    );
    assert!(g.legal_moves().is_empty());
    let (g, err) = g.play(mv("e1f2")).err().unwrap();
    assert_eq!(err, MoveError::GameOver);
    assert!(g.is_over());
}

#[test]
fn threefold_repetition_ends_the_game() {
    let shuffle = ["g1f3", "g8f6", "f3g1", "f6g8"];
    // After one shuffle the start position has occurred twice: still going.
    let g = play_any(AnyGame::new(), &shuffle);
    assert!(!g.is_over());
    // After the second, it occurs a third time.
    let g = play_any(g, &shuffle);
    assert_eq!(
        g.result(),
        Some(GameResult::Draw(DrawReason::ThreefoldRepetition))
    );
}

#[test]
fn fifty_move_rule_ends_the_game() {
    let g = AnyGame::from_fen("4k3/8/8/8/8/8/4P3/R3K3 w - - 99 80").unwrap();
    let g = play_any(g, &["a1a2"]);
    assert_eq!(
        g.result(),
        Some(GameResult::Draw(DrawReason::FiftyMoveRule))
    );
}

#[test]
fn capturing_last_piece_is_insufficient_material() {
    let g = AnyGame::from_fen("4k3/8/8/8/8/8/3q4/4K3 w - - 0 1").unwrap();
    let g = play_any(g, &["e1d2"]);
    assert_eq!(
        g.result(),
        Some(GameResult::Draw(DrawReason::InsufficientMaterial))
    );
}

#[test]
fn moving_into_stalemate_ends_the_game() {
    let g = AnyGame::from_fen("7k/4Q3/6K1/8/8/8/8/8 w - - 0 1").unwrap();
    let g = play_any(g, &["e7f7"]);
    assert_eq!(g.result(), Some(GameResult::Draw(DrawReason::Stalemate)));
}

#[test]
fn from_fen_picks_the_right_side_or_finishes() {
    let black = AnyGame::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    assert!(matches!(black, Ok(AnyGame::Black(_))));
    let mated = AnyGame::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");
    assert_eq!(
        mated.ok().and_then(|g| g.result()),
        Some(GameResult::BlackWins(WinReason::Checkmate))
    );
    assert!(AnyGame::from_fen("not a fen").is_err());
}

#[test]
fn resign_and_draw_by_agreement() {
    let g = play_any(AnyGame::new(), &["e2e4"]).resign();
    assert_eq!(
        g.result(),
        Some(GameResult::WhiteWins(WinReason::Resignation))
    );
    let g = AnyGame::new().agree_draw();
    assert_eq!(g.result(), Some(GameResult::Draw(DrawReason::Agreement)));
}

#[test]
fn any_game_fen_round_trips() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let g = AnyGame::from_fen(fen).unwrap();
    assert_eq!(g.to_fen(), fen);
    assert_eq!(g.position(), &Position::try_from(fen).unwrap());
}
