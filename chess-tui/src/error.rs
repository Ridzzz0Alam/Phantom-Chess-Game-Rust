//! `AppError`: every error the TUI can show, wrapping the engine's errors and I/O.

use chess_core::{FenError, MoveError, ParseError};
use std::fmt;
use std::io;

/// Anything that can go wrong in the app. Shown in the status bar, never panics.
#[derive(Debug)]
pub enum AppError {
    /// Reading or writing a save file failed.
    Io(io::Error),
    /// A loaded or typed FEN was invalid.
    Fen(FenError),
    /// A typed move or square could not be parsed.
    Parse(ParseError),
    /// The move was rejected by the rules.
    Move(MoveError),
    /// A save name with characters other than letters, digits, `-` and `_`.
    BadFileName(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "file error: {e}"),
            AppError::Fen(e) => write!(f, "{e}"),
            AppError::Parse(e) => write!(f, "{e}"),
            AppError::Move(e) => write!(f, "{e}"),
            AppError::BadFileName(name) => write!(
                f,
                "invalid save name '{name}' (use letters, digits, - and _)"
            ),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            AppError::Fen(e) => Some(e),
            AppError::Parse(e) => Some(e),
            AppError::Move(e) => Some(e),
            AppError::BadFileName(_) => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<FenError> for AppError {
    fn from(e: FenError) -> Self {
        AppError::Fen(e)
    }
}

impl From<ParseError> for AppError {
    fn from(e: ParseError) -> Self {
        AppError::Parse(e)
    }
}

impl From<MoveError> for AppError {
    fn from(e: MoveError) -> Self {
        AppError::Move(e)
    }
}
