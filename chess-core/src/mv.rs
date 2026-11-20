//! Moves in UCI-style coordinate notation.

use crate::error::ParseError;
use crate::piece::PieceKind;
use crate::square::Square;
use std::fmt;
use std::str::FromStr;

/// A move from one square to another, with an optional promotion piece.
///
/// Castling is written as the king's move (`e1g1`). En passant is the pawn's move.
///
/// ```
/// use chess_core::{Move, PieceKind};
/// let m = Move::try_from("e7e8q").unwrap();
/// assert_eq!(m.promotion, Some(PieceKind::Queen));
/// assert_eq!(m.to_string(), "e7e8q");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    /// Square the piece leaves.
    pub from: Square,
    /// Square the piece lands on.
    pub to: Square,
    /// Piece a pawn promotes to, if any.
    pub promotion: Option<PieceKind>,
}

impl Move {
    /// A move with no promotion.
    #[must_use]
    pub const fn new(from: Square, to: Square) -> Move {
        Move {
            from,
            to,
            promotion: None,
        }
    }

    /// A pawn move that promotes to `kind`.
    #[must_use]
    pub const fn with_promotion(from: Square, to: Square, kind: PieceKind) -> Move {
        Move {
            from,
            to,
            promotion: Some(kind),
        }
    }
}

/// Parses `"e2e4"` or `"e7e8q"` (promotion letter `q`, `r`, `b` or `n`, either case).
impl TryFrom<&str> for Move {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let bad = || ParseError::BadMove(s.to_string());
        if !s.is_ascii() || !(s.len() == 4 || s.len() == 5) {
            return Err(bad());
        }
        let from = Square::try_from(&s[0..2]).map_err(|_| bad())?;
        let to = Square::try_from(&s[2..4]).map_err(|_| bad())?;
        if from == to {
            return Err(bad());
        }
        let promotion = match s[4..].chars().next() {
            None => None,
            Some(c) => match PieceKind::try_from(c) {
                Ok(kind) if kind.is_promotion_target() => Some(kind),
                _ => return Err(bad()),
            },
        };
        Ok(Move {
            from,
            to,
            promotion,
        })
    }
}

impl FromStr for Move {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Move::try_from(s)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(kind) = self.promotion {
            write!(f, "{}", kind.to_char())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(s: &str) -> Square {
        Square::try_from(s).unwrap()
    }

    #[test]
    fn parses_plain_move() {
        assert_eq!(Move::try_from("e2e4"), Ok(Move::new(sq("e2"), sq("e4"))));
    }

    #[test]
    fn parses_all_promotions() {
        for (c, kind) in [
            ('q', PieceKind::Queen),
            ('r', PieceKind::Rook),
            ('b', PieceKind::Bishop),
            ('n', PieceKind::Knight),
        ] {
            let text = format!("e7e8{c}");
            assert_eq!(
                Move::try_from(text.as_str()),
                Ok(Move::with_promotion(sq("e7"), sq("e8"), kind))
            );
        }
    }

    #[test]
    fn accepts_uppercase_promotion_letter() {
        assert_eq!(
            Move::try_from("a2a1Q").map(|m| m.promotion),
            Ok(Some(PieceKind::Queen))
        );
    }

    #[test]
    fn rejects_malformed_moves() {
        for s in [
            "", "e2", "e2e", "e2e4qq", "e2e9", "z2e4", "e2e2", "e7e8k", "e7e8p", "e7e8x", "e2-e4",
            "é2e4",
        ] {
            assert_eq!(
                Move::try_from(s),
                Err(ParseError::BadMove(s.to_string())),
                "input {s:?}"
            );
        }
    }

    #[test]
    fn display_round_trips() {
        for s in ["e2e4", "g1f3", "e1g1", "b7b8n", "h2h1q"] {
            let m: Move = s.parse().unwrap();
            assert_eq!(m.to_string(), s);
        }
    }
}
