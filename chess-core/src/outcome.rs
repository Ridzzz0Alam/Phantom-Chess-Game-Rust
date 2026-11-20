//! Game results and the rules that end a game: mate, stalemate, the 50-move
//! rule, threefold repetition and insufficient material.

use crate::color::Color;
use crate::mv::Move;
use crate::piece::PieceKind;
use crate::position::Position;
use std::fmt;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Why the winner won.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WinReason {
    /// The loser's king is in check and they have no legal move.
    Checkmate,
    /// The loser resigned.
    Resignation,
}

/// Why the game was drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DrawReason {
    /// The side to move has no legal move but is not in check.
    Stalemate,
    /// 50 moves by each side without a capture or pawn move.
    FiftyMoveRule,
    /// The same position occurred three times.
    ThreefoldRepetition,
    /// Neither side can possibly checkmate.
    InsufficientMaterial,
    /// Both players agreed to a draw.
    Agreement,
}

/// The final result of a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameResult {
    /// White won.
    WhiteWins(WinReason),
    /// Black won.
    BlackWins(WinReason),
    /// Nobody won.
    Draw(DrawReason),
}

impl GameResult {
    /// `color` wins for `reason`.
    #[must_use]
    pub const fn win_for(color: Color, reason: WinReason) -> GameResult {
        match color {
            Color::White => GameResult::WhiteWins(reason),
            Color::Black => GameResult::BlackWins(reason),
        }
    }

    /// The winning colour, or `None` for a draw.
    #[must_use]
    pub const fn winner(self) -> Option<Color> {
        match self {
            GameResult::WhiteWins(_) => Some(Color::White),
            GameResult::BlackWins(_) => Some(Color::Black),
            GameResult::Draw(_) => None,
        }
    }

    /// PGN score string: `1-0`, `0-1` or `1/2-1/2`.
    #[must_use]
    pub const fn score(self) -> &'static str {
        match self {
            GameResult::WhiteWins(_) => "1-0",
            GameResult::BlackWins(_) => "0-1",
            GameResult::Draw(_) => "1/2-1/2",
        }
    }
}

impl fmt::Display for WinReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            WinReason::Checkmate => "checkmate",
            WinReason::Resignation => "resignation",
        })
    }
}

impl fmt::Display for DrawReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DrawReason::Stalemate => "stalemate",
            DrawReason::FiftyMoveRule => "the 50-move rule",
            DrawReason::ThreefoldRepetition => "threefold repetition",
            DrawReason::InsufficientMaterial => "insufficient material",
            DrawReason::Agreement => "agreement",
        })
    }
}

/// e.g. `1-0 White wins by checkmate`, `1/2-1/2 Draw by stalemate`.
impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameResult::WhiteWins(r) => write!(f, "{} White wins by {r}", self.score()),
            GameResult::BlackWins(r) => write!(f, "{} Black wins by {r}", self.score()),
            GameResult::Draw(r) => write!(f, "{} Draw by {r}", self.score()),
        }
    }
}

impl Position {
    /// Whether neither side has enough material to checkmate: K v K, K+B v K,
    /// K+N v K, or K+B v K+B with both bishops on the same square colour.
    #[must_use]
    pub fn is_insufficient_material(&self) -> bool {
        let mut minors = Vec::new();
        for (sq, piece) in self.board.iter() {
            match piece.kind {
                PieceKind::King => {}
                PieceKind::Knight | PieceKind::Bishop => minors.push((sq, piece)),
                PieceKind::Pawn | PieceKind::Rook | PieceKind::Queen => return false,
            }
        }
        match minors.as_slice() {
            [] | [_] => true,
            [(s1, p1), (s2, p2)] => {
                let shade = |s: crate::Square| (s.file() + s.rank()) % 2;
                p1.kind == PieceKind::Bishop
                    && p2.kind == PieceKind::Bishop
                    && p1.color != p2.color
                    && shade(*s1) == shade(*s2)
            }
            _ => false,
        }
    }

    /// Hash identifying the position for repetition: board, side to move,
    /// castling rights, and the en-passant square only if an en-passant capture
    /// is actually legal (the clocks are ignored).
    #[must_use]
    pub fn repetition_key(&self) -> u64 {
        let ep = self.en_passant.filter(|&target| {
            self.legal_moves().iter().any(|m: &Move| {
                m.to == target
                    && self
                        .board
                        .get(m.from)
                        .is_some_and(|p| p.kind == PieceKind::Pawn)
            })
        });
        let mut h = DefaultHasher::new();
        (&self.board, self.side_to_move, self.castling, ep).hash(&mut h);
        h.finish()
    }

    /// The result if the game is over by rule, given how many times the current
    /// position has occurred. Checkmate and stalemate take priority.
    #[must_use]
    pub fn game_result(&self, occurrences: u8) -> Option<GameResult> {
        if self.legal_moves().is_empty() {
            return Some(if self.in_check() {
                GameResult::win_for(self.side_to_move.opposite(), WinReason::Checkmate)
            } else {
                GameResult::Draw(DrawReason::Stalemate)
            });
        }
        if self.is_insufficient_material() {
            Some(GameResult::Draw(DrawReason::InsufficientMaterial))
        } else if occurrences >= 3 {
            Some(GameResult::Draw(DrawReason::ThreefoldRepetition))
        } else if self.halfmove_clock >= 100 {
            Some(GameResult::Draw(DrawReason::FiftyMoveRule))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(fen: &str) -> Position {
        Position::try_from(fen).unwrap()
    }

    #[test]
    fn insufficient_material_cases() {
        assert!(pos("4k3/8/8/8/8/8/8/4K3 w - - 0 1").is_insufficient_material());
        assert!(pos("4k3/8/8/8/8/8/8/2B1K3 w - - 0 1").is_insufficient_material());
        assert!(pos("4k3/8/8/8/8/8/8/1N2K3 w - - 0 1").is_insufficient_material());
        // Bishops on same-coloured squares (c1 dark, f8 dark).
        assert!(pos("4kb2/8/8/8/8/8/8/2B1K3 w - - 0 1").is_insufficient_material());
    }

    #[test]
    fn sufficient_material_cases() {
        assert!(!Position::startpos().is_insufficient_material());
        assert!(!pos("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1").is_insufficient_material());
        assert!(!pos("4k3/8/8/8/8/8/8/R3K3 w - - 0 1").is_insufficient_material());
        // Two knights, opposite-coloured bishops, bishop + knight: mate is possible.
        assert!(!pos("4k3/8/8/8/8/8/8/1N2KN2 w - - 0 1").is_insufficient_material());
        assert!(!pos("4k1b1/8/8/8/8/8/8/2B1K3 w - - 0 1").is_insufficient_material());
        assert!(!pos("4k3/8/8/8/8/8/8/1NB1K3 w - - 0 1").is_insufficient_material());
    }

    #[test]
    fn useless_en_passant_square_does_not_change_repetition_key() {
        // After 1. e4 no black pawn can take on e3.
        let with_ep = pos("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        let without = pos("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
        assert_eq!(with_ep.repetition_key(), without.repetition_key());
    }

    #[test]
    fn usable_en_passant_square_changes_repetition_key() {
        let with_ep = pos("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
        let without = pos("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3");
        assert_ne!(with_ep.repetition_key(), without.repetition_key());
    }

    #[test]
    fn result_display() {
        assert_eq!(
            GameResult::BlackWins(WinReason::Checkmate).to_string(),
            "0-1 Black wins by checkmate"
        );
        assert_eq!(
            GameResult::Draw(DrawReason::Stalemate).to_string(),
            "1/2-1/2 Draw by stalemate"
        );
    }

    #[test]
    fn game_result_detects_each_rule() {
        assert_eq!(
            pos("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").game_result(1),
            Some(GameResult::Draw(DrawReason::Stalemate))
        );
        assert_eq!(
            pos("4k3/8/8/8/8/8/4P3/4K3 w - - 100 80").game_result(1),
            Some(GameResult::Draw(DrawReason::FiftyMoveRule))
        );
        assert_eq!(
            Position::startpos().game_result(3),
            Some(GameResult::Draw(DrawReason::ThreefoldRepetition))
        );
        assert_eq!(Position::startpos().game_result(2), None);
    }
}
