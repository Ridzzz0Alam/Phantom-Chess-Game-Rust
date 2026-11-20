//! Error types. All are hand-written enums implementing `Display` and `std::error::Error`.

use crate::mv::Move;
use crate::square::Square;
use std::error::Error;
use std::fmt;

/// Failure to parse a square, piece or move from text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Not a square like `"e4"` (or an index ≥ 64).
    BadSquare(String),
    /// Not a piece letter (`pnbrqk` / `PNBRQK`).
    BadPiece(char),
    /// Not a UCI move like `"e2e4"` or `"e7e8q"`.
    BadMove(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::BadSquare(s) => write!(f, "invalid square '{s}' (expected a1..h8)"),
            ParseError::BadPiece(c) => write!(f, "invalid piece letter '{c}'"),
            ParseError::BadMove(s) => {
                write!(f, "invalid move '{s}' (expected e.g. e2e4 or e7e8q)")
            }
        }
    }
}

impl Error for ParseError {}

/// Failure to parse a FEN string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FenError {
    /// FEN must have 6 space-separated fields; holds the number found.
    WrongFieldCount(usize),
    /// The piece-placement field is malformed.
    BadBoard(String),
    /// The side-to-move field is not `w` or `b`.
    BadSide(String),
    /// The castling field is malformed.
    BadCastling(String),
    /// The en-passant field is malformed.
    BadEnPassant(String),
    /// The halfmove or fullmove field is not a valid number.
    BadNumber(String),
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenError::WrongFieldCount(n) => {
                write!(f, "FEN must have 6 fields, found {n}")
            }
            FenError::BadBoard(s) => write!(f, "invalid FEN board: {s}"),
            FenError::BadSide(s) => write!(f, "invalid FEN side to move '{s}' (expected w or b)"),
            FenError::BadCastling(s) => write!(f, "invalid FEN castling rights '{s}'"),
            FenError::BadEnPassant(s) => write!(f, "invalid FEN en-passant square '{s}'"),
            FenError::BadNumber(s) => write!(f, "invalid FEN move counter '{s}'"),
        }
    }
}

impl Error for FenError {}

/// Lets FEN parsing use `?` on the square/piece parsers.
///
/// A bad piece letter can only appear in the board field and a bad square only in
/// the en-passant field, so each maps to that field's variant. Moves never appear
/// in FEN; `BadMove` maps to `BadBoard` only so the conversion is total.
impl From<ParseError> for FenError {
    fn from(e: ParseError) -> FenError {
        match e {
            ParseError::BadPiece(c) => FenError::BadBoard(format!("invalid piece letter '{c}'")),
            ParseError::BadSquare(s) => FenError::BadEnPassant(s),
            ParseError::BadMove(s) => FenError::BadBoard(s),
        }
    }
}

/// Why a move was rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoveError {
    /// The `from` square is empty.
    NoPieceOnSquare(Square),
    /// The piece on `from` belongs to the opponent.
    NotYourPiece(Square),
    /// The piece cannot move that way.
    IllegalMove(Move),
    /// The move would leave (or put) the mover's own king in check.
    LeavesKingInCheck(Move),
    /// A pawn reached the last rank but no promotion piece was given.
    MissingPromotion,
    /// The game has already ended.
    GameOver,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveError::NoPieceOnSquare(sq) => write!(f, "there is no piece on {sq}"),
            MoveError::NotYourPiece(sq) => write!(f, "the piece on {sq} is not yours"),
            MoveError::IllegalMove(mv) => write!(f, "{mv} is not a legal move"),
            MoveError::LeavesKingInCheck(mv) => {
                write!(f, "{mv} would leave your king in check")
            }
            MoveError::MissingPromotion => f.write_str("choose a promotion piece (q, r, b or n)"),
            MoveError::GameOver => f.write_str("the game is over"),
        }
    }
}

impl Error for MoveError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_messages() {
        assert_eq!(
            ParseError::BadSquare("z9".into()).to_string(),
            "invalid square 'z9' (expected a1..h8)"
        );
        assert_eq!(
            ParseError::BadPiece('x').to_string(),
            "invalid piece letter 'x'"
        );
    }

    #[test]
    fn fen_error_messages() {
        assert_eq!(
            FenError::WrongFieldCount(3).to_string(),
            "FEN must have 6 fields, found 3"
        );
    }

    #[test]
    fn move_error_messages_use_square_and_move_display() {
        let e2 = Square::try_from("e2").unwrap();
        assert_eq!(
            MoveError::NoPieceOnSquare(e2).to_string(),
            "there is no piece on e2"
        );
        let mv = Move::try_from("e2e5").unwrap();
        assert_eq!(
            MoveError::IllegalMove(mv).to_string(),
            "e2e5 is not a legal move"
        );
    }

    #[test]
    fn errors_are_std_errors() {
        fn takes_error(_: &dyn Error) {}
        takes_error(&ParseError::BadPiece('x'));
        takes_error(&FenError::BadSide("x".into()));
        takes_error(&MoveError::GameOver);
    }
}
