//! The phantom-type game layer: whose turn it is is part of the type.
//!
//! [`Game<WhiteToMove>`] can only produce a [`Game<BlackToMove>`] (or a finished
//! game) and vice versa, and [`Game::play`] consumes `self`. So code in which the
//! same side moves twice, or in which a stale game value is reused, does not
//! compile. [`AnyGame`] is the boundary to the dynamic world (UI, FEN loading),
//! where the side to move is only known at runtime.

use crate::color::Color;
use crate::error::MoveError;
use crate::mv::Move;
use crate::outcome::{DrawReason, GameResult, WinReason};
use crate::position::Position;
use crate::square::Square;
use std::collections::HashMap;
use std::marker::PhantomData;

mod any_game;
pub use any_game::AnyGame;

mod sealed {
    /// Private supertrait: only this crate can implement [`Side`](super::Side).
    pub trait Sealed {}
    impl Sealed for super::WhiteToMove {}
    impl Sealed for super::BlackToMove {}
}

/// A type-level "whose turn it is". Implemented only by [`WhiteToMove`] and
/// [`BlackToMove`]; the trait is sealed, so no third side can exist.
///
/// ```compile_fail,E0277
/// use chess_core::{Color, Side};
/// enum Nobody {}
/// impl Side for Nobody {
///     const COLOR: Color = Color::White;
///     type Next = Nobody;
///     # fn into_any(_: chess_core::Game<Self>) -> chess_core::AnyGame { todo!() }
/// }
/// ```
pub trait Side: sealed::Sealed + Sized {
    /// The runtime colour of this side.
    const COLOR: Color;
    /// The side that moves next.
    type Next: Side;

    /// Wraps a typed game in the matching [`AnyGame`] variant.
    #[doc(hidden)]
    fn into_any(game: Game<Self>) -> AnyGame;
}

/// Marker type: White is to move. Uninhabited, it exists only at type level.
#[derive(Clone, Copy, Debug)]
pub enum WhiteToMove {}

/// Marker type: Black is to move. Uninhabited, it exists only at type level.
#[derive(Clone, Copy, Debug)]
pub enum BlackToMove {}

impl Side for WhiteToMove {
    const COLOR: Color = Color::White;
    type Next = BlackToMove;
    fn into_any(game: Game<Self>) -> AnyGame {
        AnyGame::White(game)
    }
}

impl Side for BlackToMove {
    const COLOR: Color = Color::Black;
    type Next = WhiteToMove;
    fn into_any(game: Game<Self>) -> AnyGame {
        AnyGame::Black(game)
    }
}

/// A game in progress where side `S` is to move.
///
/// Fields are private, so a `Game<S>` can only be obtained from
/// [`Game::new`], [`AnyGame`], or by playing a move, and its position's side to
/// move always equals `S::COLOR`.
///
/// ```compile_fail,E0616
/// let game = chess_core::Game::new();
/// let _ = game.position; // error: field `position` is private
/// ```
#[derive(Clone, Debug)]
pub struct Game<S: Side> {
    position: Position,
    history: Vec<Move>,
    repetitions: HashMap<u64, u8>,
    _turn: PhantomData<S>,
}

/// The outcome of playing a move: the game continues with the other side to
/// move, or it is over.
#[derive(Clone, Debug)]
pub enum Played<N: Side> {
    /// The game goes on; `N` is to move.
    Ongoing(Game<N>),
    /// The move ended the game.
    Finished(FinishedGame),
}

impl<N: Side> Played<N> {
    /// The continuing game, if the game is not over.
    #[must_use]
    pub fn ongoing(self) -> Option<Game<N>> {
        match self {
            Played::Ongoing(g) => Some(g),
            Played::Finished(_) => None,
        }
    }

    /// The finished game, if the move ended it.
    #[must_use]
    pub fn finished(self) -> Option<FinishedGame> {
        match self {
            Played::Ongoing(_) => None,
            Played::Finished(f) => Some(f),
        }
    }
}

/// A game that has ended. No more moves can be played.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinishedGame {
    position: Position,
    history: Vec<Move>,
    result: GameResult,
}

impl FinishedGame {
    /// The final position.
    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// All moves played, in order.
    #[must_use]
    pub fn history(&self) -> &[Move] {
        &self.history
    }

    /// How the game ended.
    #[must_use]
    pub fn result(&self) -> GameResult {
        self.result
    }
}

impl Game<WhiteToMove> {
    /// A new game from the standard starting position.
    #[must_use]
    pub fn new() -> Self {
        let position = Position::startpos();
        let repetitions = HashMap::from([(position.repetition_key(), 1)]);
        Game {
            position,
            history: Vec::new(),
            repetitions,
            _turn: PhantomData,
        }
    }
}

impl Default for Game<WhiteToMove> {
    fn default() -> Self {
        Game::new()
    }
}

impl<S: Side> Game<S> {
    /// Plays `mv`, consuming this game.
    ///
    /// On success the turn passes to `S::Next` (or the game ends). On failure the
    /// unchanged game is handed back together with the reason, so an illegal move
    /// never loses state.
    ///
    /// ```
    /// use chess_core::{BlackToMove, Game, Played, WhiteToMove};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let white: Game<WhiteToMove> = Game::new();
    /// let black: Game<BlackToMove> = match white.play("e2e4".try_into()?) {
    ///     Ok(Played::Ongoing(g)) => g,
    ///     _ => unreachable!(),
    /// };
    /// let white_again: Game<WhiteToMove> = black.play("e7e5".try_into()?).unwrap().ongoing().unwrap();
    /// assert_eq!(white_again.history().len(), 2);
    /// # Ok(()) }
    /// ```
    ///
    /// The same side cannot move twice: `play` takes `self` by value, so the old
    /// `Game<WhiteToMove>` is gone after the first call.
    ///
    /// ```compile_fail,E0382
    /// use chess_core::{Game, Move};
    /// let game = Game::new();
    /// let e4 = Move::try_from("e2e4").unwrap();
    /// let d4 = Move::try_from("d2d4").unwrap();
    /// let _ = game.play(e4);
    /// let _ = game.play(d4); // error[E0382]: use of moved value: `game`
    /// ```
    ///
    /// And a White-to-move game is a different type from a Black-to-move one:
    ///
    /// ```compile_fail,E0308
    /// use chess_core::{BlackToMove, Game};
    /// fn black_plays(_: Game<BlackToMove>) {}
    /// black_plays(Game::new()); // error[E0308]: expected `BlackToMove`, found `WhiteToMove`
    /// ```
    #[expect(
        clippy::result_large_err,
        reason = "returning the game by value on error is the point of the API; Ok is just as large"
    )]
    pub fn play(self, mv: Move) -> Result<Played<S::Next>, (Self, MoveError)> {
        debug_assert_eq!(self.position.side_to_move(), S::COLOR);
        let mut game = self;
        if let Err(e) = game.position.make_move(mv) {
            return Err((game, e));
        }
        game.history.push(mv);
        if game.position.halfmove_clock() == 0 {
            // A capture or pawn move is irreversible: earlier positions can't recur.
            game.repetitions.clear();
        }
        let count = game
            .repetitions
            .entry(game.position.repetition_key())
            .or_insert(0);
        *count = count.saturating_add(1);
        let occurrences = *count;

        Ok(match game.position.game_result(occurrences) {
            Some(result) => Played::Finished(FinishedGame {
                position: game.position,
                history: game.history,
                result,
            }),
            None => Played::Ongoing(Game {
                position: game.position,
                history: game.history,
                repetitions: game.repetitions,
                _turn: PhantomData,
            }),
        })
    }

    /// The side to move (always `S::COLOR`).
    #[must_use]
    pub fn side(&self) -> Color {
        S::COLOR
    }

    /// The current position (read-only).
    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// All moves played so far, in order.
    #[must_use]
    pub fn history(&self) -> &[Move] {
        &self.history
    }

    /// Legal moves for the side to move.
    #[must_use]
    pub fn legal_moves(&self) -> Vec<Move> {
        self.position.legal_moves()
    }

    /// Legal moves of the piece on `from`.
    #[must_use]
    pub fn legal_moves_from(&self, from: Square) -> Vec<Move> {
        self.position.legal_moves_from(from)
    }

    /// Whether the side to move is in check.
    #[must_use]
    pub fn in_check(&self) -> bool {
        self.position.in_check()
    }

    /// The side to move resigns.
    #[must_use]
    pub fn resign(self) -> FinishedGame {
        self.finish(GameResult::win_for(
            S::COLOR.opposite(),
            WinReason::Resignation,
        ))
    }

    /// Both players agree to a draw.
    #[must_use]
    pub fn agree_draw(self) -> FinishedGame {
        self.finish(GameResult::Draw(DrawReason::Agreement))
    }

    fn finish(self, result: GameResult) -> FinishedGame {
        FinishedGame {
            position: self.position,
            history: self.history,
            result,
        }
    }
}
