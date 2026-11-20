//! Mailbox board: one optional piece per square.

use crate::color::Color;
use crate::piece::{Piece, PieceKind};
use crate::square::Square;
use std::fmt;

/// The 64 squares and what stands on them. Holds no game state (turn, castling, …);
/// see [`Position`](crate::Position) for that.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board {
    squares: [Option<Piece>; Square::COUNT],
}

impl Board {
    /// A board with no pieces.
    #[must_use]
    pub const fn empty() -> Board {
        Board {
            squares: [None; Square::COUNT],
        }
    }

    /// The standard starting arrangement.
    #[must_use]
    pub fn startpos() -> Board {
        const BACK_RANK: [PieceKind; 8] = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];
        let mut board = Board::empty();
        for color in Color::ALL {
            let back = color.back_rank();
            let pawns = if color == Color::White { 1 } else { 6 };
            for (file, kind) in (0u8..).zip(BACK_RANK) {
                board.set(Square::at(file, back), Piece::new(color, kind));
                board.set(Square::at(file, pawns), Piece::new(color, PieceKind::Pawn));
            }
        }
        board
    }

    /// The piece on `sq`, if any.
    #[must_use]
    pub fn get(&self, sq: Square) -> Option<Piece> {
        self.squares[sq.index()]
    }

    /// Puts `piece` on `sq`, returning whatever stood there before.
    pub fn set(&mut self, sq: Square, piece: Piece) -> Option<Piece> {
        self.squares[sq.index()].replace(piece)
    }

    /// Empties `sq`, returning the piece that stood there.
    pub fn remove(&mut self, sq: Square) -> Option<Piece> {
        self.squares[sq.index()].take()
    }

    /// Whether `sq` is empty.
    #[must_use]
    pub fn is_empty(&self, sq: Square) -> bool {
        self.get(sq).is_none()
    }

    /// Iterates over occupied squares, a1 → h8.
    pub fn iter(&self) -> impl Iterator<Item = (Square, Piece)> + '_ {
        Square::all().filter_map(|sq| self.get(sq).map(|p| (sq, p)))
    }

    /// Iterates over the squares and pieces of one colour.
    pub fn pieces_of(&self, color: Color) -> impl Iterator<Item = (Square, Piece)> + '_ {
        self.iter().filter(move |(_, p)| p.color == color)
    }

    /// The square of `color`'s king (the first one found, if the board is odd).
    #[must_use]
    pub fn find_king(&self, color: Color) -> Option<Square> {
        let king = Piece::new(color, PieceKind::King);
        self.iter().find(|&(_, p)| p == king).map(|(sq, _)| sq)
    }
}

impl Default for Board {
    fn default() -> Board {
        Board::empty()
    }
}

/// Text diagram, rank 8 at the top, `.` for empty squares.
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let c = self
                    .get(Square::at(file, rank))
                    .map_or('.', Piece::to_fen_char);
                write!(f, "{c}")?;
                if file < 7 {
                    f.write_str(" ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(s: &str) -> Square {
        Square::try_from(s).unwrap()
    }

    #[test]
    fn set_get_remove() {
        let mut b = Board::empty();
        let wn = Piece::new(Color::White, PieceKind::Knight);
        let bq = Piece::new(Color::Black, PieceKind::Queen);
        assert_eq!(b.set(sq("f3"), wn), None);
        assert_eq!(b.get(sq("f3")), Some(wn));
        assert_eq!(b.set(sq("f3"), bq), Some(wn));
        assert_eq!(b.remove(sq("f3")), Some(bq));
        assert!(b.is_empty(sq("f3")));
        assert_eq!(b.remove(sq("f3")), None);
    }

    #[test]
    fn startpos_layout() {
        let b = Board::startpos();
        assert_eq!(b.iter().count(), 32);
        assert_eq!(b.pieces_of(Color::White).count(), 16);
        assert_eq!(
            b.get(sq("d1")),
            Some(Piece::new(Color::White, PieceKind::Queen))
        );
        assert_eq!(
            b.get(sq("g8")),
            Some(Piece::new(Color::Black, PieceKind::Knight))
        );
        assert_eq!(
            b.get(sq("c7")),
            Some(Piece::new(Color::Black, PieceKind::Pawn))
        );
        assert!(b.is_empty(sq("e4")));
    }

    #[test]
    fn find_king_in_startpos() {
        let b = Board::startpos();
        assert_eq!(b.find_king(Color::White), Some(sq("e1")));
        assert_eq!(b.find_king(Color::Black), Some(sq("e8")));
        assert_eq!(Board::empty().find_king(Color::White), None);
    }

    #[test]
    fn iter_is_in_square_order() {
        let b = Board::startpos();
        let first: Vec<Square> = b.iter().map(|(s, _)| s).take(2).collect();
        assert_eq!(first, vec![sq("a1"), sq("b1")]);
    }

    #[test]
    fn display_diagram() {
        let text = Board::startpos().to_string();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "r n b q k b n r");
        assert_eq!(lines[4], ". . . . . . . .");
        assert_eq!(lines[7], "R N B Q K B N R");
    }
}
