//! Move generation, attack detection, legality checks and perft.
//!
//! Strategy (PLAN §3.5): generate pseudo-legal moves per piece, then keep only the
//! moves after which the mover's king is not attacked. Castling additionally
//! requires that the king's start, transit and destination squares are safe.

use crate::color::Color;
use crate::error::MoveError;
use crate::mv::Move;
use crate::piece::{Piece, PieceKind};
use crate::position::{CastleSide, KING_START_FILE, Position};
use crate::square::Square;

/// (file, rank) steps of a knight.
const KNIGHT_STEPS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];
/// (file, rank) steps of a king; also the eight sliding directions.
const KING_STEPS: [(i8, i8); 8] = [
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
];
const ROOK_DIRS: [(i8, i8); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
const BISHOP_DIRS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

impl Position {
    /// Whether any piece of colour `by` attacks `sq` (regardless of pins).
    #[must_use]
    pub fn is_attacked(&self, sq: Square, by: Color) -> bool {
        let is = |s: Option<Square>, kind: PieceKind| {
            s.and_then(|s| self.board.get(s)) == Some(Piece::new(by, kind))
        };
        // A pawn of `by` attacks diagonally forward, so look one rank "behind" `sq`.
        let back = -by.pawn_direction();
        if is(sq.offset(-1, back), PieceKind::Pawn) || is(sq.offset(1, back), PieceKind::Pawn) {
            return true;
        }
        let steps_hit =
            |steps: &[(i8, i8)], kind| steps.iter().any(|&(df, dr)| is(sq.offset(df, dr), kind));
        if steps_hit(&KNIGHT_STEPS, PieceKind::Knight) || steps_hit(&KING_STEPS, PieceKind::King) {
            return true;
        }
        let ray_hit = |dirs: &[(i8, i8)], slider: PieceKind| {
            dirs.iter()
                .any(|&(df, dr)| match self.first_piece_on_ray(sq, df, dr) {
                    Some(p) => p.color == by && (p.kind == slider || p.kind == PieceKind::Queen),
                    None => false,
                })
        };
        ray_hit(&ROOK_DIRS, PieceKind::Rook) || ray_hit(&BISHOP_DIRS, PieceKind::Bishop)
    }

    /// The first piece met walking from `sq` (exclusive) in direction (`df`, `dr`).
    fn first_piece_on_ray(&self, sq: Square, df: i8, dr: i8) -> Option<Piece> {
        let mut cur = sq;
        while let Some(next) = cur.offset(df, dr) {
            if let Some(p) = self.board.get(next) {
                return Some(p);
            }
            cur = next;
        }
        None
    }

    /// Whether the side to move is in check.
    #[must_use]
    pub fn in_check(&self) -> bool {
        self.king_attacked(self.side_to_move)
    }

    /// Whether `color`'s king is attacked. A missing king counts as not attacked.
    pub(crate) fn king_attacked(&self, color: Color) -> bool {
        self.board
            .find_king(color)
            .is_some_and(|k| self.is_attacked(k, color.opposite()))
    }

    /// All pseudo-legal moves: they follow piece movement rules but may leave
    /// the mover's own king in check.
    #[must_use]
    pub fn pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        for (from, piece) in self.board.pieces_of(self.side_to_move) {
            self.piece_moves(from, piece, &mut moves);
        }
        moves
    }

    /// All legal moves for the side to move.
    #[must_use]
    pub fn legal_moves(&self) -> Vec<Move> {
        self.pseudo_legal_moves()
            .into_iter()
            .filter(|&mv| !self.leaves_king_in_check(mv))
            .collect()
    }

    /// Legal moves of the piece on `from` (empty if it is not the mover's piece).
    #[must_use]
    pub fn legal_moves_from(&self, from: Square) -> Vec<Move> {
        let mut moves = Vec::new();
        if let Some(piece) = self.board.get(from)
            && piece.color == self.side_to_move
        {
            self.piece_moves(from, piece, &mut moves);
        }
        moves.retain(|&mv| !self.leaves_king_in_check(mv));
        moves
    }

    /// Checks that `mv` is legal here, explaining why not if it isn't.
    pub fn check_move(&self, mv: Move) -> Result<(), MoveError> {
        let piece = self
            .board
            .get(mv.from)
            .ok_or(MoveError::NoPieceOnSquare(mv.from))?;
        if piece.color != self.side_to_move {
            return Err(MoveError::NotYourPiece(mv.from));
        }
        let mut candidates = Vec::new();
        self.piece_moves(mv.from, piece, &mut candidates);
        if !candidates.contains(&mv) {
            let needs_promotion = mv.promotion.is_none()
                && candidates
                    .iter()
                    .any(|c| c.to == mv.to && c.promotion.is_some());
            return Err(if needs_promotion {
                MoveError::MissingPromotion
            } else {
                MoveError::IllegalMove(mv)
            });
        }
        if self.leaves_king_in_check(mv) {
            return Err(MoveError::LeavesKingInCheck(mv));
        }
        Ok(())
    }

    /// Plays `mv` on a copy and reports whether the mover's king ends up attacked.
    fn leaves_king_in_check(&self, mv: Move) -> bool {
        let mover = self.side_to_move;
        let mut next = self.clone();
        next.apply(mv);
        next.king_attacked(mover)
    }

    /// Appends the pseudo-legal moves of `piece` standing on `from`.
    fn piece_moves(&self, from: Square, piece: Piece, out: &mut Vec<Move>) {
        match piece.kind {
            PieceKind::Pawn => self.pawn_moves(from, piece.color, out),
            PieceKind::Knight => self.step_moves(from, piece.color, &KNIGHT_STEPS, out),
            PieceKind::Bishop => self.slide_moves(from, piece.color, &BISHOP_DIRS, out),
            PieceKind::Rook => self.slide_moves(from, piece.color, &ROOK_DIRS, out),
            PieceKind::Queen => {
                self.slide_moves(from, piece.color, &ROOK_DIRS, out);
                self.slide_moves(from, piece.color, &BISHOP_DIRS, out);
            }
            PieceKind::King => {
                self.step_moves(from, piece.color, &KING_STEPS, out);
                self.castling_moves(from, piece.color, out);
            }
        }
    }

    /// Single-step pieces (knight, king): every on-board target not holding a friendly piece.
    fn step_moves(&self, from: Square, color: Color, steps: &[(i8, i8)], out: &mut Vec<Move>) {
        out.extend(
            steps
                .iter()
                .filter_map(|&(df, dr)| from.offset(df, dr))
                .filter(|&to| self.board.get(to).is_none_or(|p| p.color != color))
                .map(|to| Move::new(from, to)),
        );
    }

    /// Sliding pieces: walk each ray until the edge or a piece (capturing an enemy one).
    fn slide_moves(&self, from: Square, color: Color, dirs: &[(i8, i8)], out: &mut Vec<Move>) {
        for &(df, dr) in dirs {
            let mut cur = from;
            while let Some(to) = cur.offset(df, dr) {
                match self.board.get(to) {
                    None => out.push(Move::new(from, to)),
                    Some(p) => {
                        if p.color != color {
                            out.push(Move::new(from, to));
                        }
                        break;
                    }
                }
                cur = to;
            }
        }
    }

    fn pawn_moves(&self, from: Square, color: Color, out: &mut Vec<Move>) {
        let dir = color.pawn_direction();
        let start_rank = if color == Color::White { 1 } else { 6 };
        let last_rank = color.opposite().back_rank();
        // Every pawn move onto the last rank expands into the four promotions.
        let mut add = |to: Square| {
            if to.rank() == last_rank {
                out.extend(
                    PieceKind::PROMOTIONS
                        .iter()
                        .map(|&k| Move::with_promotion(from, to, k)),
                );
            } else {
                out.push(Move::new(from, to));
            }
        };

        if let Some(one) = from.offset(0, dir)
            && self.board.is_empty(one)
        {
            add(one);
            if from.rank() == start_rank
                && let Some(two) = one.offset(0, dir)
                && self.board.is_empty(two)
            {
                add(two);
            }
        }
        for df in [-1, 1] {
            let Some(to) = from.offset(df, dir) else {
                continue;
            };
            let enemy = self.board.get(to).is_some_and(|p| p.color != color);
            if enemy || self.en_passant == Some(to) {
                add(to);
            }
        }
    }

    fn castling_moves(&self, from: Square, color: Color, out: &mut Vec<Move>) {
        let back = color.back_rank();
        if from != Square::at(KING_START_FILE, back) {
            return;
        }
        let enemy = color.opposite();
        for side in CastleSide::BOTH {
            if !self.castling.has(color, side) {
                continue;
            }
            let rook_file = side.rook_file();
            if self.board.get(Square::at(rook_file, back))
                != Some(Piece::new(color, PieceKind::Rook))
            {
                continue;
            }
            // Every square strictly between king and rook must be empty.
            let (lo, hi) = (
                rook_file.min(KING_START_FILE),
                rook_file.max(KING_START_FILE),
            );
            if !(lo + 1..hi).all(|f| self.board.is_empty(Square::at(f, back))) {
                continue;
            }
            // King may not start in, pass through or land on an attacked square.
            let target = side.king_target_file();
            let (k_lo, k_hi) = (target.min(KING_START_FILE), target.max(KING_START_FILE));
            if (k_lo..=k_hi).any(|f| self.is_attacked(Square::at(f, back), enemy)) {
                continue;
            }
            out.push(Move::new(from, Square::at(target, back)));
        }
    }

    /// Counts leaf nodes of the legal move tree to `depth` (move-generator test).
    #[must_use]
    pub fn perft(&self, depth: u32) -> u64 {
        match depth {
            0 => 1,
            1 => self.legal_moves().len() as u64,
            _ => self
                .legal_moves()
                .into_iter()
                .map(|mv| {
                    let mut next = self.clone();
                    next.apply(mv);
                    next.perft(depth - 1)
                })
                .sum(),
        }
    }

    /// Perft split by root move, for comparing against a reference engine.
    /// Returns `(move, nodes)` pairs; the nodes sum to `perft(depth)`.
    #[must_use]
    pub fn divide(&self, depth: u32) -> Vec<(Move, u64)> {
        self.legal_moves()
            .into_iter()
            .map(|mv| {
                let mut next = self.clone();
                next.apply(mv);
                (mv, next.perft(depth.saturating_sub(1)))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::STARTPOS_FEN;

    fn pos(fen: &str) -> Position {
        Position::try_from(fen).unwrap()
    }
    fn sq(s: &str) -> Square {
        Square::try_from(s).unwrap()
    }

    #[test]
    fn startpos_has_twenty_moves() {
        assert_eq!(pos(STARTPOS_FEN).legal_moves().len(), 20);
    }

    #[test]
    fn attacks_by_each_piece_type() {
        let p = pos("4k3/8/8/3p4/8/5n2/8/R3K2B w - - 0 1");
        // Black pawn d5 attacks c4 and e4, not backwards (e6).
        assert!(p.is_attacked(sq("c4"), Color::Black));
        assert!(p.is_attacked(sq("e4"), Color::Black));
        assert!(!p.is_attacked(sq("e6"), Color::Black));
        // Knight f3 attacks e1 and h2.
        assert!(p.is_attacked(sq("e1"), Color::Black));
        // Rook a1 attacks along the rank up to the king, and the a-file.
        assert!(p.is_attacked(sq("d1"), Color::White));
        assert!(p.is_attacked(sq("a8"), Color::White));
        // Bishop h1 is blocked by the knight on f3.
        assert!(p.is_attacked(sq("f3"), Color::White));
        assert!(!p.is_attacked(sq("e4"), Color::White));
    }

    #[test]
    fn divide_sums_to_perft() {
        let p = Position::startpos();
        let total: u64 = p.divide(2).iter().map(|(_, n)| n).sum();
        assert_eq!(total, p.perft(2));
    }

    #[test]
    fn legal_moves_from_only_lists_that_piece() {
        let p = Position::startpos();
        let moves = p.legal_moves_from(sq("g1"));
        assert_eq!(moves.len(), 2);
        assert!(p.legal_moves_from(sq("e7")).is_empty());
        assert!(p.legal_moves_from(sq("e4")).is_empty());
    }
}
