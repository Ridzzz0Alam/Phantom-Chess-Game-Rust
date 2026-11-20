//! Side colours.

use std::fmt;

/// One of the two sides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    /// The side that moves first.
    White,
    /// The side that moves second.
    Black,
}

impl Color {
    /// Both colours, White first.
    pub const ALL: [Color; 2] = [Color::White, Color::Black];

    /// The other colour.
    #[must_use]
    pub const fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// The rank (0-based) this side's pieces start on: 0 for White, 7 for Black.
    #[must_use]
    pub const fn back_rank(self) -> u8 {
        match self {
            Color::White => 0,
            Color::Black => 7,
        }
    }

    /// Direction pawns of this colour move in: +1 for White, -1 for Black.
    #[must_use]
    pub const fn pawn_direction(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Color::White => "White",
            Color::Black => "Black",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_flips_colour() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }

    #[test]
    fn opposite_twice_is_identity() {
        for c in Color::ALL {
            assert_eq!(c.opposite().opposite(), c);
        }
    }

    #[test]
    fn back_rank_and_pawn_direction() {
        assert_eq!(Color::White.back_rank(), 0);
        assert_eq!(Color::Black.back_rank(), 7);
        assert_eq!(Color::White.pawn_direction(), 1);
        assert_eq!(Color::Black.pawn_direction(), -1);
    }

    #[test]
    fn display_names() {
        assert_eq!(Color::White.to_string(), "White");
        assert_eq!(Color::Black.to_string(), "Black");
    }
}
