//! Forsyth–Edwards Notation: parsing (`TryFrom<&str> for Position`) and printing (`to_fen`).

use crate::board::Board;
use crate::color::Color;
use crate::error::FenError;
use crate::piece::{Piece, PieceKind};
use crate::position::{CastleSide, CastlingRights, KING_START_FILE, Position};
use crate::square::Square;
use std::str::FromStr;

/// FEN of the standard starting position.
pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Parses a 6-field FEN string.
///
/// Besides syntax, it checks that the position makes sense: one king per side,
/// no pawns on the first or last rank, castling rights only where the king and
/// rook are on their start squares, and an en-passant square only right after
/// a matching double pawn push.
///
/// ```
/// use chess_core::{FenError, Position};
/// let pos = Position::try_from("8/8/8/8/8/8/8/K6k w - - 0 1").unwrap();
/// assert_eq!(pos.to_fen(), "8/8/8/8/8/8/8/K6k w - - 0 1");
/// assert_eq!(Position::try_from("8/8 w - - 0 1"),
///            Err(FenError::BadBoard("expected 8 ranks, found 2".into())));
/// ```
impl TryFrom<&str> for Position {
    type Error = FenError;

    fn try_from(fen: &str) -> Result<Position, FenError> {
        let fields: Vec<&str> = fen.split_whitespace().collect();
        let &[board, side, castling, ep, half, full] = fields.as_slice() else {
            return Err(FenError::WrongFieldCount(fields.len()));
        };

        let board = parse_board(board)?;
        let side_to_move = parse_side(side)?;
        let castling = parse_castling(castling, &board)?;
        let en_passant = parse_en_passant(ep, &board, side_to_move)?;
        let halfmove_clock = parse_number(half)?;
        let fullmove_number = parse_number(full)?;
        if fullmove_number == 0 {
            return Err(FenError::BadNumber(full.to_string()));
        }

        let pos = Position {
            board,
            side_to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
        };
        // The side that just moved cannot have left its own king in check.
        if pos.king_attacked(side_to_move.opposite()) {
            return Err(FenError::BadBoard(format!(
                "{} is in check but it is {} to move",
                side_to_move.opposite(),
                side_to_move
            )));
        }
        Ok(pos)
    }
}

impl FromStr for Position {
    type Err = FenError;

    fn from_str(s: &str) -> Result<Position, FenError> {
        Position::try_from(s)
    }
}

impl Position {
    /// The position as a FEN string.
    #[must_use]
    pub fn to_fen(&self) -> String {
        let mut out = String::with_capacity(90);
        for rank in (0..8).rev() {
            let mut empty = 0u8;
            for file in 0..8 {
                match self.board.get(Square::at(file, rank)) {
                    Some(piece) => {
                        if empty > 0 {
                            out.push(char::from(b'0' + empty));
                            empty = 0;
                        }
                        out.push(piece.to_fen_char());
                    }
                    None => empty += 1,
                }
            }
            if empty > 0 {
                out.push(char::from(b'0' + empty));
            }
            if rank > 0 {
                out.push('/');
            }
        }
        let side = match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        };
        let ep = self
            .en_passant
            .map_or_else(|| "-".to_string(), |sq| sq.to_string());
        out.push_str(&format!(
            " {side} {} {ep} {} {}",
            self.castling, self.halfmove_clock, self.fullmove_number
        ));
        out
    }
}

fn parse_board(field: &str) -> Result<Board, FenError> {
    let ranks: Vec<&str> = field.split('/').collect();
    if ranks.len() != 8 {
        return Err(FenError::BadBoard(format!(
            "expected 8 ranks, found {}",
            ranks.len()
        )));
    }

    let mut board = Board::empty();
    // FEN lists rank 8 first.
    for (rank, text) in (0..8u8).rev().zip(ranks) {
        let too_long = || FenError::BadBoard(format!("rank {} has more than 8 squares", rank + 1));
        let mut file = 0u8;
        for c in text.chars() {
            match c.to_digit(10) {
                Some(n @ 1..=8) => file = file.saturating_add(n as u8),
                Some(_) => {
                    return Err(FenError::BadBoard(format!(
                        "invalid empty-square count '{c}' on rank {}",
                        rank + 1
                    )));
                }
                None => {
                    let piece = Piece::try_from(c)?;
                    let sq = Square::new(file, rank).ok_or_else(too_long)?;
                    board.set(sq, piece);
                    file += 1;
                }
            }
            if file > 8 {
                return Err(too_long());
            }
        }
        if file != 8 {
            return Err(FenError::BadBoard(format!(
                "rank {} has {file} squares, expected 8",
                rank + 1
            )));
        }
    }

    validate_board(&board)?;
    Ok(board)
}

fn validate_board(board: &Board) -> Result<(), FenError> {
    for color in Color::ALL {
        let kings = board
            .pieces_of(color)
            .filter(|(_, p)| p.kind == PieceKind::King)
            .count();
        if kings != 1 {
            return Err(FenError::BadBoard(format!(
                "{color} must have exactly one king, found {kings}"
            )));
        }
    }
    if let Some((sq, _)) = board
        .iter()
        .find(|(sq, p)| p.kind == PieceKind::Pawn && (sq.rank() == 0 || sq.rank() == 7))
    {
        return Err(FenError::BadBoard(format!("pawn on back rank square {sq}")));
    }
    Ok(())
}

fn parse_side(field: &str) -> Result<Color, FenError> {
    match field {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(FenError::BadSide(field.to_string())),
    }
}

fn parse_castling(field: &str, board: &Board) -> Result<CastlingRights, FenError> {
    let bad = || FenError::BadCastling(field.to_string());
    let mut rights = CastlingRights::NONE;
    if field == "-" {
        return Ok(rights);
    }
    for c in field.chars() {
        let (color, side) = match c {
            'K' => (Color::White, CastleSide::KingSide),
            'Q' => (Color::White, CastleSide::QueenSide),
            'k' => (Color::Black, CastleSide::KingSide),
            'q' => (Color::Black, CastleSide::QueenSide),
            _ => return Err(bad()),
        };
        if rights.has(color, side) {
            return Err(bad()); // duplicate letter
        }
        // The right only makes sense if king and rook are still on their start squares.
        let back = color.back_rank();
        let king_ok = board.get(Square::at(KING_START_FILE, back))
            == Some(Piece::new(color, PieceKind::King));
        let rook_ok = board.get(Square::at(side.rook_file(), back))
            == Some(Piece::new(color, PieceKind::Rook));
        if !king_ok || !rook_ok {
            return Err(bad());
        }
        rights.insert(color, side);
    }
    Ok(rights)
}

fn parse_en_passant(field: &str, board: &Board, side: Color) -> Result<Option<Square>, FenError> {
    if field == "-" {
        return Ok(None);
    }
    let bad = || FenError::BadEnPassant(field.to_string());
    let target = Square::try_from(field)?;

    // The opponent just double-pushed: their pawn passed over `target`.
    let mover = side.opposite();
    let expected_rank = match mover {
        Color::White => 2,
        Color::Black => 5,
    };
    if target.rank() != expected_rank {
        return Err(bad());
    }
    let dir = mover.pawn_direction();
    let pawn_sq = target.offset(0, dir).ok_or_else(bad)?;
    let origin = target.offset(0, -dir).ok_or_else(bad)?;
    let pawn_there = board.get(pawn_sq) == Some(Piece::new(mover, PieceKind::Pawn));
    if !pawn_there || !board.is_empty(target) || !board.is_empty(origin) {
        return Err(bad());
    }
    Ok(Some(target))
}

fn parse_number(field: &str) -> Result<u16, FenError> {
    field
        .parse::<u16>()
        .map_err(|_| FenError::BadNumber(field.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_fen_matches_startpos() {
        assert_eq!(Position::try_from(STARTPOS_FEN), Ok(Position::startpos()));
        assert_eq!(Position::startpos().to_fen(), STARTPOS_FEN);
    }

    #[test]
    fn extra_whitespace_is_tolerated() {
        let messy = "  rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR   w KQkq -  0 1 ";
        assert_eq!(Position::try_from(messy), Ok(Position::startpos()));
    }

    #[test]
    fn huge_digit_run_does_not_overflow() {
        let fen = "88888888888888888888888888888888/8/8/8/8/8/8/K6k w - - 0 1";
        assert!(matches!(
            Position::try_from(fen),
            Err(FenError::BadBoard(_))
        ));
    }
}
