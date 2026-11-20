//! [`AnyGame`]: the runtime boundary between the typed engine and code (UI, FEN
//! loading) that only knows whose turn it is at runtime.

use super::{BlackToMove, FinishedGame, Game, Played, Side, WhiteToMove};
use crate::color::Color;
use crate::error::{FenError, MoveError};
use crate::mv::Move;
use crate::outcome::GameResult;
use crate::position::Position;
use crate::square::Square;
use std::collections::HashMap;
use std::marker::PhantomData;

/// A game whose side to move is only known at runtime. This is what a UI stores.
///
/// The engine underneath is still typestate-checked: each variant holds a
/// correctly typed [`Game`], and [`AnyGame::play`] dispatches to [`Game::play`].
#[derive(Clone, Debug)]
pub enum AnyGame {
    /// White to move.
    White(Game<WhiteToMove>),
    /// Black to move.
    Black(Game<BlackToMove>),
    /// The game has ended.
    Over(FinishedGame),
}

impl<S: Side> From<Game<S>> for AnyGame {
    fn from(game: Game<S>) -> AnyGame {
        S::into_any(game)
    }
}

impl<N: Side> From<Played<N>> for AnyGame {
    fn from(played: Played<N>) -> AnyGame {
        match played {
            Played::Ongoing(game) => game.into(),
            Played::Finished(f) => AnyGame::Over(f),
        }
    }
}

impl Default for AnyGame {
    fn default() -> AnyGame {
        AnyGame::new()
    }
}

impl AnyGame {
    /// A new game from the starting position.
    #[must_use]
    pub fn new() -> AnyGame {
        AnyGame::White(Game::new())
    }

    /// Starts a game from a FEN string. The game may already be over
    /// (e.g. a checkmate position).
    pub fn from_fen(fen: &str) -> Result<AnyGame, FenError> {
        let position = Position::try_from(fen)?;
        let key = position.repetition_key();
        if let Some(result) = position.game_result(1) {
            return Ok(AnyGame::Over(FinishedGame {
                position,
                history: Vec::new(),
                result,
            }));
        }
        let repetitions = HashMap::from([(key, 1)]);
        Ok(match position.side_to_move() {
            Color::White => AnyGame::White(Game {
                position,
                history: Vec::new(),
                repetitions,
                _turn: PhantomData,
            }),
            Color::Black => AnyGame::Black(Game {
                position,
                history: Vec::new(),
                repetitions,
                _turn: PhantomData,
            }),
        })
    }

    /// Plays `mv`. On failure the unchanged game is handed back with the reason;
    /// a finished game rejects every move with [`MoveError::GameOver`].
    #[expect(
        clippy::result_large_err,
        reason = "returning the game by value on error is the point of the API; Ok is just as large"
    )]
    pub fn play(self, mv: Move) -> Result<AnyGame, (AnyGame, MoveError)> {
        match self {
            AnyGame::White(g) => g
                .play(mv)
                .map(AnyGame::from)
                .map_err(|(g, e)| (g.into(), e)),
            AnyGame::Black(g) => g
                .play(mv)
                .map(AnyGame::from)
                .map_err(|(g, e)| (g.into(), e)),
            over @ AnyGame::Over(_) => Err((over, MoveError::GameOver)),
        }
    }

    /// The side to move resigns. No effect on a finished game.
    #[must_use]
    pub fn resign(self) -> AnyGame {
        match self {
            AnyGame::White(g) => AnyGame::Over(g.resign()),
            AnyGame::Black(g) => AnyGame::Over(g.resign()),
            over @ AnyGame::Over(_) => over,
        }
    }

    /// Ends the game as a draw by agreement. No effect on a finished game.
    #[must_use]
    pub fn agree_draw(self) -> AnyGame {
        match self {
            AnyGame::White(g) => AnyGame::Over(g.agree_draw()),
            AnyGame::Black(g) => AnyGame::Over(g.agree_draw()),
            over @ AnyGame::Over(_) => over,
        }
    }

    /// The current (or final) position.
    #[must_use]
    pub fn position(&self) -> &Position {
        match self {
            AnyGame::White(g) => g.position(),
            AnyGame::Black(g) => g.position(),
            AnyGame::Over(f) => f.position(),
        }
    }

    /// All moves played so far.
    #[must_use]
    pub fn history(&self) -> &[Move] {
        match self {
            AnyGame::White(g) => g.history(),
            AnyGame::Black(g) => g.history(),
            AnyGame::Over(f) => f.history(),
        }
    }

    /// Legal moves (empty once the game is over).
    #[must_use]
    pub fn legal_moves(&self) -> Vec<Move> {
        match self {
            AnyGame::Over(_) => Vec::new(),
            _ => self.position().legal_moves(),
        }
    }

    /// Legal moves of the piece on `from` (empty once the game is over).
    #[must_use]
    pub fn legal_moves_from(&self, from: Square) -> Vec<Move> {
        match self {
            AnyGame::Over(_) => Vec::new(),
            _ => self.position().legal_moves_from(from),
        }
    }

    /// The side to move, or `None` once the game is over.
    #[must_use]
    pub fn side_to_move(&self) -> Option<Color> {
        match self {
            AnyGame::White(_) => Some(Color::White),
            AnyGame::Black(_) => Some(Color::Black),
            AnyGame::Over(_) => None,
        }
    }

    /// The result, once the game is over.
    #[must_use]
    pub fn result(&self) -> Option<GameResult> {
        match self {
            AnyGame::Over(f) => Some(f.result()),
            _ => None,
        }
    }

    /// Whether the game has ended.
    #[must_use]
    pub fn is_over(&self) -> bool {
        matches!(self, AnyGame::Over(_))
    }

    /// FEN of the current position.
    #[must_use]
    pub fn to_fen(&self) -> String {
        self.position().to_fen()
    }
}
