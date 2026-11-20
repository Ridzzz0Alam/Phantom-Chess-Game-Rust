//! Board squares.

use crate::error::ParseError;
use std::fmt;
use std::str::FromStr;

/// A square on the board, stored as an index 0..64.
///
/// `0 = a1`, `7 = h1`, `56 = a8`, `63 = h8`. The inner value is private, so every
/// `Square` is valid.
///
/// ```
/// use chess_core::Square;
/// let e4 = Square::try_from("e4").unwrap();
/// assert_eq!((e4.file(), e4.rank()), (4, 3));
/// assert!(Square::try_from("z9").is_err());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Square(u8);

impl Square {
    /// Number of squares on the board.
    pub const COUNT: usize = 64;

    /// Builds a square from a 0-based file (a = 0) and rank (1st rank = 0).
    /// Returns `None` if either is outside `0..8`.
    #[must_use]
    pub const fn new(file: u8, rank: u8) -> Option<Square> {
        if file < 8 && rank < 8 {
            Some(Square(rank * 8 + file))
        } else {
            None
        }
    }

    /// Infallible constructor for in-crate constants. Both coordinates are
    /// masked to `0..8`, so callers must pass known-valid values.
    pub(crate) const fn at(file: u8, rank: u8) -> Square {
        Square((rank & 7) * 8 + (file & 7))
    }

    /// 0-based file: a = 0 … h = 7.
    #[must_use]
    pub const fn file(self) -> u8 {
        self.0 % 8
    }

    /// 0-based rank: 1st rank = 0 … 8th rank = 7.
    #[must_use]
    pub const fn rank(self) -> u8 {
        self.0 / 8
    }

    /// Index 0..64, for indexing board arrays.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// The square `df` files and `dr` ranks away, or `None` if that is off the board.
    #[must_use]
    pub const fn offset(self, df: i8, dr: i8) -> Option<Square> {
        // Widen to i16 so extreme offsets (e.g. 127) cannot overflow.
        let f = self.file() as i16 + df as i16;
        let r = self.rank() as i16 + dr as i16;
        if f >= 0 && f < 8 && r >= 0 && r < 8 {
            // In range 0..64 here, so the cast is lossless.
            Some(Square((r * 8 + f) as u8))
        } else {
            None
        }
    }

    /// Iterates over all 64 squares, a1, b1, …, h8.
    pub fn all() -> impl Iterator<Item = Square> {
        (0..64u8).map(Square)
    }
}

/// Parses algebraic notation such as `"e4"`.
impl TryFrom<&str> for Square {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Square::from_algebraic(s).ok_or_else(|| ParseError::BadSquare(s.to_string()))
    }
}

impl Square {
    /// `const` parser for algebraic names (`"e4"`): lower-case file a–h, rank 1–8.
    /// Usable in constants; this is what the [`sq!`](crate::sq) macro is built on.
    #[must_use]
    pub const fn from_algebraic(s: &str) -> Option<Square> {
        let &[f, r] = s.as_bytes() else {
            return None;
        };
        if f >= b'a' && f <= b'h' && r >= b'1' && r <= b'8' {
            Square::new(f - b'a', r - b'1')
        } else {
            None
        }
    }
}

/// Converts a raw index; fails for values ≥ 64.
impl TryFrom<u8> for Square {
    type Error = ParseError;

    fn try_from(i: u8) -> Result<Self, Self::Error> {
        if i < 64 {
            Ok(Square(i))
        } else {
            Err(ParseError::BadSquare(i.to_string()))
        }
    }
}

impl FromStr for Square {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Square::try_from(s)
    }
}

impl From<Square> for u8 {
    fn from(sq: Square) -> u8 {
        sq.0
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            char::from(b'a' + self.file()),
            char::from(b'1' + self.rank())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_indices_match_layout() {
        assert_eq!(Square::try_from("a1").map(u8::from), Ok(0));
        assert_eq!(Square::try_from("h1").map(u8::from), Ok(7));
        assert_eq!(Square::try_from("a8").map(u8::from), Ok(56));
        assert_eq!(Square::try_from("h8").map(u8::from), Ok(63));
    }

    #[test]
    fn file_and_rank() {
        let e4 = Square::try_from("e4").unwrap();
        assert_eq!(e4.file(), 4);
        assert_eq!(e4.rank(), 3);
        assert_eq!(Square::new(4, 3), Some(e4));
    }

    #[test]
    fn new_rejects_out_of_range() {
        assert_eq!(Square::new(8, 0), None);
        assert_eq!(Square::new(0, 8), None);
    }

    #[test]
    fn display_round_trips_for_all_squares() {
        for sq in Square::all() {
            assert_eq!(Square::try_from(sq.to_string().as_str()), Ok(sq));
        }
        assert_eq!(Square::all().count(), 64);
    }

    #[test]
    fn parse_rejects_bad_strings() {
        for s in ["", "e", "e44", "z9", "i1", "a0", "a9", "E4", "4e", "é4"] {
            assert_eq!(
                Square::try_from(s),
                Err(ParseError::BadSquare(s.to_string())),
                "input {s:?}"
            );
        }
    }

    #[test]
    fn from_str_works() {
        assert_eq!("d5".parse::<Square>(), Square::try_from("d5"));
    }

    #[test]
    fn try_from_u8() {
        assert_eq!(
            Square::try_from(0u8).map(|s| s.to_string()),
            Ok("a1".into())
        );
        assert_eq!(
            Square::try_from(63u8).map(|s| s.to_string()),
            Ok("h8".into())
        );
        assert_eq!(
            Square::try_from(64u8),
            Err(ParseError::BadSquare("64".into()))
        );
        assert!(Square::try_from(255u8).is_err());
    }

    #[test]
    fn offset_inside_and_off_board() {
        let e4 = Square::try_from("e4").unwrap();
        assert_eq!(e4.offset(1, 2), Square::try_from("f6").ok());
        assert_eq!(e4.offset(-4, -3), Square::try_from("a1").ok());
        let a1 = Square::try_from("a1").unwrap();
        assert_eq!(a1.offset(-1, 0), None);
        assert_eq!(a1.offset(0, -1), None);
        let h8 = Square::try_from("h8").unwrap();
        assert_eq!(h8.offset(1, 0), None);
        assert_eq!(h8.offset(0, 1), None);
        // h-file must not wrap onto the a-file of the next rank.
        let h4 = Square::try_from("h4").unwrap();
        assert_eq!(h4.offset(1, 0), None);
    }

    #[test]
    fn extreme_offsets_do_not_overflow() {
        let h8 = Square::try_from("h8").unwrap();
        assert_eq!(h8.offset(i8::MAX, i8::MAX), None);
        let a1 = Square::try_from("a1").unwrap();
        assert_eq!(a1.offset(i8::MIN, i8::MIN), None);
    }
}
