//! Piece kinds and coloured pieces.

use crate::color::Color;
use crate::error::ParseError;
use std::fmt;

/// The kind of a piece, independent of colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceKind {
    /// Pawn.
    Pawn,
    /// Knight.
    Knight,
    /// Bishop.
    Bishop,
    /// Rook.
    Rook,
    /// Queen.
    Queen,
    /// King.
    King,
}

impl PieceKind {
    /// All six kinds.
    pub const ALL: [PieceKind; 6] = [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ];

    /// The kinds a pawn may promote to, strongest first.
    pub const PROMOTIONS: [PieceKind; 4] = [
        PieceKind::Queen,
        PieceKind::Rook,
        PieceKind::Bishop,
        PieceKind::Knight,
    ];

    /// Lower-case FEN / UCI letter: `p n b r q k`.
    #[must_use]
    pub const fn to_char(self) -> char {
        match self {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        }
    }

    /// Whether a pawn may promote to this kind.
    #[must_use]
    pub const fn is_promotion_target(self) -> bool {
        matches!(
            self,
            PieceKind::Knight | PieceKind::Bishop | PieceKind::Rook | PieceKind::Queen
        )
    }
}

/// Parses a FEN letter, ignoring case (`'n'` and `'N'` are both `Knight`).
impl TryFrom<char> for PieceKind {
    type Error = ParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'p' => Ok(PieceKind::Pawn),
            'n' => Ok(PieceKind::Knight),
            'b' => Ok(PieceKind::Bishop),
            'r' => Ok(PieceKind::Rook),
            'q' => Ok(PieceKind::Queen),
            'k' => Ok(PieceKind::King),
            _ => Err(ParseError::BadPiece(c)),
        }
    }
}

impl fmt::Display for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PieceKind::Pawn => "pawn",
            PieceKind::Knight => "knight",
            PieceKind::Bishop => "bishop",
            PieceKind::Rook => "rook",
            PieceKind::Queen => "queen",
            PieceKind::King => "king",
        })
    }
}

/// A piece of a given colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    /// Owner of the piece.
    pub color: Color,
    /// What kind of piece it is.
    pub kind: PieceKind,
}

impl Piece {
    /// Creates a piece.
    #[must_use]
    pub const fn new(color: Color, kind: PieceKind) -> Piece {
        Piece { color, kind }
    }

    /// FEN letter: upper case for White (`N`), lower case for Black (`n`).
    #[must_use]
    pub const fn to_fen_char(self) -> char {
        let c = self.kind.to_char();
        match self.color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    /// Unicode chess glyph (♔♕♖♗♘♙ for White, ♚♛♜♝♞♟ for Black).
    #[must_use]
    pub const fn to_unicode(self) -> char {
        match (self.color, self.kind) {
            (Color::White, PieceKind::King) => '♔',
            (Color::White, PieceKind::Queen) => '♕',
            (Color::White, PieceKind::Rook) => '♖',
            (Color::White, PieceKind::Bishop) => '♗',
            (Color::White, PieceKind::Knight) => '♘',
            (Color::White, PieceKind::Pawn) => '♙',
            (Color::Black, PieceKind::King) => '♚',
            (Color::Black, PieceKind::Queen) => '♛',
            (Color::Black, PieceKind::Rook) => '♜',
            (Color::Black, PieceKind::Bishop) => '♝',
            (Color::Black, PieceKind::Knight) => '♞',
            (Color::Black, PieceKind::Pawn) => '♟',
        }
    }
}

/// Parses a FEN letter; the case gives the colour (`'N'` = White knight, `'n'` = Black knight).
impl TryFrom<char> for Piece {
    type Error = ParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        let kind = PieceKind::try_from(c)?;
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        Ok(Piece { color, kind })
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_fen_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_kind_parses_both_cases() {
        assert_eq!(PieceKind::try_from('n'), Ok(PieceKind::Knight));
        assert_eq!(PieceKind::try_from('N'), Ok(PieceKind::Knight));
        assert_eq!(PieceKind::try_from('K'), Ok(PieceKind::King));
    }

    #[test]
    fn piece_kind_rejects_unknown_letter() {
        assert_eq!(PieceKind::try_from('x'), Err(ParseError::BadPiece('x')));
        assert_eq!(PieceKind::try_from('1'), Err(ParseError::BadPiece('1')));
    }

    #[test]
    fn piece_case_gives_colour() {
        assert_eq!(
            Piece::try_from('Q'),
            Ok(Piece::new(Color::White, PieceKind::Queen))
        );
        assert_eq!(
            Piece::try_from('q'),
            Ok(Piece::new(Color::Black, PieceKind::Queen))
        );
    }

    #[test]
    fn piece_rejects_unknown_letter() {
        assert_eq!(Piece::try_from('z'), Err(ParseError::BadPiece('z')));
    }

    #[test]
    fn fen_char_round_trips_for_every_piece() {
        for color in Color::ALL {
            for kind in PieceKind::ALL {
                let p = Piece::new(color, kind);
                assert_eq!(Piece::try_from(p.to_fen_char()), Ok(p));
            }
        }
    }

    #[test]
    fn unicode_glyphs() {
        assert_eq!(Piece::new(Color::White, PieceKind::King).to_unicode(), '♔');
        assert_eq!(Piece::new(Color::Black, PieceKind::Pawn).to_unicode(), '♟');
    }

    #[test]
    fn only_minor_and_major_pieces_are_promotion_targets() {
        assert!(!PieceKind::Pawn.is_promotion_target());
        assert!(!PieceKind::King.is_promotion_target());
        for k in PieceKind::PROMOTIONS {
            assert!(k.is_promotion_target());
        }
    }
}
