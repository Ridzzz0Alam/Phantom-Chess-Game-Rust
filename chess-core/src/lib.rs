//! `chess-core`: a hand-written chess rules engine.
//!
//! Turn order is enforced with phantom types (`Game<WhiteToMove>` / `Game<BlackToMove>`),
//! added in Phase 4. This crate contains no `unsafe` code.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod board;
pub mod color;
pub mod error;
pub mod fen;
pub mod movegen;
pub mod mv;
pub mod piece;
pub mod position;
pub mod square;

pub use board::Board;
pub use color::Color;
pub use error::{FenError, MoveError, ParseError};
pub use fen::STARTPOS_FEN;
pub use mv::Move;
pub use piece::{Piece, PieceKind};
pub use position::{CastleSide, CastlingRights, Position};
pub use square::Square;
