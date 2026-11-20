//! Untyped game state: board, side to move, castling, en passant, clocks.

use crate::board::Board;
use crate::color::Color;
use crate::error::MoveError;
use crate::mv::Move;
use crate::piece::{Piece, PieceKind};
use crate::square::Square;
use std::fmt;

/// Which way a king castles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CastleSide {
    /// Towards the h-file (`O-O`).
    KingSide,
    /// Towards the a-file (`O-O-O`).
    QueenSide,
}

impl CastleSide {
    /// Both sides, king side first.
    pub const BOTH: [CastleSide; 2] = [CastleSide::KingSide, CastleSide::QueenSide];

    /// File the castling rook starts on (h = 7 or a = 0).
    #[must_use]
    pub const fn rook_file(self) -> u8 {
        match self {
            CastleSide::KingSide => 7,
            CastleSide::QueenSide => 0,
        }
    }

    /// File the king lands on (g = 6 or c = 2).
    #[must_use]
    pub const fn king_target_file(self) -> u8 {
        match self {
            CastleSide::KingSide => 6,
            CastleSide::QueenSide => 2,
        }
    }

    /// File the rook lands on (f = 5 or d = 3).
    #[must_use]
    pub const fn rook_target_file(self) -> u8 {
        match self {
            CastleSide::KingSide => 5,
            CastleSide::QueenSide => 3,
        }
    }
}

/// File the king starts on for castling (e).
pub const KING_START_FILE: u8 = 4;

/// The four castling rights packed into the low 4 bits of a `u8`.
///
/// Bit 0 = White king side (`K`), bit 1 = White queen side (`Q`),
/// bit 2 = Black king side (`k`), bit 3 = Black queen side (`q`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct CastlingRights(u8);

impl CastlingRights {
    /// No castling allowed.
    pub const NONE: CastlingRights = CastlingRights(0);
    /// All four rights (`KQkq`).
    pub const ALL: CastlingRights = CastlingRights(0b1111);

    const fn bit(color: Color, side: CastleSide) -> u8 {
        match (color, side) {
            (Color::White, CastleSide::KingSide) => 0b0001,
            (Color::White, CastleSide::QueenSide) => 0b0010,
            (Color::Black, CastleSide::KingSide) => 0b0100,
            (Color::Black, CastleSide::QueenSide) => 0b1000,
        }
    }

    /// Whether `color` may still castle on `side`.
    #[must_use]
    pub const fn has(self, color: Color, side: CastleSide) -> bool {
        self.0 & Self::bit(color, side) != 0
    }

    /// Grants a right.
    pub fn insert(&mut self, color: Color, side: CastleSide) {
        self.0 |= Self::bit(color, side);
    }

    /// Revokes a right.
    pub fn remove(&mut self, color: Color, side: CastleSide) {
        self.0 &= !Self::bit(color, side);
    }

    /// Revokes both rights of `color` (e.g. after a king move).
    pub fn remove_color(&mut self, color: Color) {
        for side in CastleSide::BOTH {
            self.remove(color, side);
        }
    }

    /// Whether no rights remain.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The raw 4-bit value.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// FEN form: `KQkq`, a subset in that order, or `-`.
impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("-");
        }
        for (color, side, c) in [
            (Color::White, CastleSide::KingSide, 'K'),
            (Color::White, CastleSide::QueenSide, 'Q'),
            (Color::Black, CastleSide::KingSide, 'k'),
            (Color::Black, CastleSide::QueenSide, 'q'),
        ] {
            if self.has(color, side) {
                write!(f, "{c}")?;
            }
        }
        Ok(())
    }
}

/// Complete runtime chess state. This is the untyped engine; the turn-typed
/// wrapper `Game<S>` sits on top of it.
///
/// Fields are private: build a `Position` with [`Position::startpos`] or from FEN
/// via `Position::try_from("…")`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub(crate) board: Board,
    pub(crate) side_to_move: Color,
    pub(crate) castling: CastlingRights,
    pub(crate) en_passant: Option<Square>,
    pub(crate) halfmove_clock: u16,
    pub(crate) fullmove_number: u16,
}

impl Position {
    /// The standard starting position.
    #[must_use]
    pub fn startpos() -> Position {
        Position {
            board: Board::startpos(),
            side_to_move: Color::White,
            castling: CastlingRights::ALL,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// The board.
    #[must_use]
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// The piece on `sq`, if any.
    #[must_use]
    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.board.get(sq)
    }

    /// Whose turn it is.
    #[must_use]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Remaining castling rights.
    #[must_use]
    pub fn castling(&self) -> CastlingRights {
        self.castling
    }

    /// The square a pawn may capture onto en passant this move, if any.
    #[must_use]
    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    /// Half-moves since the last capture or pawn move (for the 50-move rule).
    #[must_use]
    pub fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    /// Full-move number, starting at 1 and incremented after Black moves.
    #[must_use]
    pub fn fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    /// Plays `mv` if it is legal; otherwise leaves the position unchanged and
    /// says why the move was rejected.
    pub fn make_move(&mut self, mv: Move) -> Result<(), MoveError> {
        self.check_move(mv)?;
        self.apply(mv);
        Ok(())
    }

    /// Plays `mv` without checking legality. `mv` must be pseudo-legal; anything
    /// else gives a meaningless (but memory-safe, non-panicking) position.
    ///
    /// Updates board, castling rights, en passant, clocks and side to move.
    pub(crate) fn apply(&mut self, mv: Move) {
        let Some(piece) = self.board.remove(mv.from) else {
            return;
        };
        let color = piece.color;
        let is_pawn = piece.kind == PieceKind::Pawn;
        let mut captured = self.board.get(mv.to).is_some();

        // En passant: a pawn moving diagonally onto the empty ep square takes the
        // pawn that stands behind that square.
        if is_pawn
            && Some(mv.to) == self.en_passant
            && mv.from.file() != mv.to.file()
            && self.board.is_empty(mv.to)
            && let Some(victim) = mv.to.offset(0, -color.pawn_direction())
        {
            captured = self.board.remove(victim).is_some();
        }

        // Castling: the king moves two files, so bring the rook across.
        if piece.kind == PieceKind::King && mv.from.file().abs_diff(mv.to.file()) == 2 {
            let side = if mv.to.file() > mv.from.file() {
                CastleSide::KingSide
            } else {
                CastleSide::QueenSide
            };
            let rank = mv.from.rank();
            if let Some(rook) = self.board.remove(Square::at(side.rook_file(), rank)) {
                self.board
                    .set(Square::at(side.rook_target_file(), rank), rook);
            }
        }

        let placed = match mv.promotion {
            Some(kind) if is_pawn => Piece::new(color, kind),
            _ => piece,
        };
        self.board.set(mv.to, placed);

        // Castling rights: lost when the king moves, or when a rook leaves or is
        // captured on its start square.
        if piece.kind == PieceKind::King {
            self.castling.remove_color(color);
        }
        for c in Color::ALL {
            for side in CastleSide::BOTH {
                let rook_home = Square::at(side.rook_file(), c.back_rank());
                if mv.from == rook_home || mv.to == rook_home {
                    self.castling.remove(c, side);
                }
            }
        }

        self.en_passant = if is_pawn && mv.from.rank().abs_diff(mv.to.rank()) == 2 {
            mv.from.offset(0, color.pawn_direction())
        } else {
            None
        };

        self.halfmove_clock = if is_pawn || captured {
            0
        } else {
            self.halfmove_clock.saturating_add(1)
        };
        if color == Color::Black {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }
        self.side_to_move = color.opposite();
    }
}

impl Default for Position {
    fn default() -> Position {
        Position::startpos()
    }
}

/// Prints the FEN string.
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_fen())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn castling_rights_insert_remove() {
        let mut r = CastlingRights::NONE;
        assert!(r.is_empty());
        r.insert(Color::Black, CastleSide::QueenSide);
        assert!(r.has(Color::Black, CastleSide::QueenSide));
        assert!(!r.has(Color::Black, CastleSide::KingSide));
        assert_eq!(r.bits(), 0b1000);
        r.remove(Color::Black, CastleSide::QueenSide);
        assert!(r.is_empty());
    }

    #[test]
    fn remove_color_keeps_other_side() {
        let mut r = CastlingRights::ALL;
        r.remove_color(Color::White);
        assert_eq!(r.to_string(), "kq");
    }

    #[test]
    fn castling_display() {
        assert_eq!(CastlingRights::ALL.to_string(), "KQkq");
        assert_eq!(CastlingRights::NONE.to_string(), "-");
        let mut r = CastlingRights::NONE;
        r.insert(Color::Black, CastleSide::KingSide);
        r.insert(Color::White, CastleSide::QueenSide);
        assert_eq!(r.to_string(), "Qk");
    }

    #[test]
    fn castle_side_files() {
        assert_eq!(CastleSide::KingSide.rook_file(), 7);
        assert_eq!(CastleSide::QueenSide.king_target_file(), 2);
        assert_eq!(CastleSide::QueenSide.rook_target_file(), 3);
    }

    #[test]
    fn startpos_state() {
        let p = Position::startpos();
        assert_eq!(p.side_to_move(), Color::White);
        assert_eq!(p.castling(), CastlingRights::ALL);
        assert_eq!(p.en_passant(), None);
        assert_eq!((p.halfmove_clock(), p.fullmove_number()), (0, 1));
    }
}
