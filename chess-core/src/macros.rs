//! Declarative macros (`macro_rules!`):
//!
//! - [`sq!`](crate::sq): a [`Square`] literal checked at compile time.
//! - [`perft_tests!`](crate::perft_tests): generates one `#[test]` per perft line.
//! - [`pst!`](crate::pst): writes an 8×8 piece-square table with rank 8 on top.

use crate::square::Square;

/// A [`Square`] from its name, checked **at compile time**.
///
/// ```
/// use chess_core::{sq, Square};
/// assert_eq!(sq!(e4), Square::try_from("e4").unwrap());
/// const KING_START: Square = sq!(e1);
/// assert_eq!(KING_START.to_string(), "e1");
/// ```
///
/// A square that doesn't exist is a compile error, not a runtime one:
///
/// ```compile_fail,E0080
/// let _ = chess_core::sq!(z9); // error: evaluation panicked: sq!: no such square: z9
/// ```
#[macro_export]
macro_rules! sq {
    ($name:ident) => {{
        // Evaluated in a const item, so a bad name fails the build, never a running program.
        const SQ: $crate::Square = match $crate::Square::from_algebraic(stringify!($name)) {
            Some(sq) => sq,
            None => panic!(concat!("sq!: no such square: ", stringify!($name))),
        };
        SQ
    }};
}

/// Generates one `#[test]` function per line, each checking a perft node count.
///
/// Each line is `[attributes] name: fen, depth => nodes;`. Attributes such as
/// `#[ignore = "slow"]` are passed through to the generated test. Intended for
/// test code: a bad FEN or a wrong count fails the test.
///
/// ```
/// chess_core::perft_tests! {
///     start_d1: chess_core::STARTPOS_FEN, 1 => 20;
///     start_d2: chess_core::STARTPOS_FEN, 2 => 400;
/// }
/// # fn main() {} // the generated #[test] fns only exist under `cargo test`
/// ```
#[macro_export]
macro_rules! perft_tests {
    ($( $(#[$meta:meta])* $name:ident : $fen:expr, $depth:literal => $nodes:literal ; )*) => {
        $(
            $(#[$meta])*
            #[test]
            fn $name() {
                match $crate::Position::try_from($fen) {
                    Ok(position) => assert_eq!(
                        position.perft($depth),
                        $nodes,
                        "perft({}) of {}",
                        $depth,
                        $fen
                    ),
                    Err(e) => panic!("bad FEN in perft test {}: {e}", stringify!($name)),
                }
            }
        )*
    };
}

/// A piece-square table: one score per square, indexed by [`Square::index`].
pub type Pst = [i16; Square::COUNT];

/// Writes a [`Pst`] the way a board is drawn: eight rows, **rank 8 first**,
/// files a→h left to right. Wrong row or column counts are compile errors.
///
/// ```
/// use chess_core::{pst, sq, macros::Pst};
/// const CENTRE: Pst = pst![
///     [0, 0, 0, 0, 0, 0, 0, 0],
///     [0, 0, 0, 0, 0, 0, 0, 0],
///     [0, 0, 0, 0, 0, 0, 0, 0],
///     [0, 0, 0, 9, 9, 0, 0, 0],
///     [0, 0, 0, 9, 9, 0, 0, 0],
///     [0, 0, 0, 0, 0, 0, 0, 0],
///     [0, 0, 0, 0, 0, 0, 0, 0],
///     [5, 0, 0, 0, 0, 0, 0, 0],
/// ];
/// assert_eq!(CENTRE[sq!(e4).index()], 9);
/// assert_eq!(CENTRE[sq!(a1).index()], 5);
/// ```
#[macro_export]
macro_rules! pst {
    ($([$($value:expr),* $(,)?]),* $(,)?) => {
        $crate::macros::pst_from_rows([$([$($value),*]),*])
    };
}

/// Converts rows written rank 8 first (as [`pst!`](crate::pst) takes them) into a
/// [`Pst`] indexed by square. `const`, so tables can be constants.
#[must_use]
pub const fn pst_from_rows(rows: [[i16; 8]; 8]) -> Pst {
    let mut table = [0; Square::COUNT];
    let mut row = 0;
    while row < 8 {
        let mut file = 0;
        while file < 8 {
            let rank = 7 - row;
            table[Square::at(file as u8, rank as u8).index()] = rows[row][file];
            file += 1;
        }
        row += 1;
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sq_matches_runtime_parsing_on_every_square() {
        #[rustfmt::skip]
        let all = [
            sq!(a1), sq!(b1), sq!(c1), sq!(d1), sq!(e1), sq!(f1), sq!(g1), sq!(h1),
            sq!(a2), sq!(b2), sq!(c2), sq!(d2), sq!(e2), sq!(f2), sq!(g2), sq!(h2),
            sq!(a3), sq!(b3), sq!(c3), sq!(d3), sq!(e3), sq!(f3), sq!(g3), sq!(h3),
            sq!(a4), sq!(b4), sq!(c4), sq!(d4), sq!(e4), sq!(f4), sq!(g4), sq!(h4),
            sq!(a5), sq!(b5), sq!(c5), sq!(d5), sq!(e5), sq!(f5), sq!(g5), sq!(h5),
            sq!(a6), sq!(b6), sq!(c6), sq!(d6), sq!(e6), sq!(f6), sq!(g6), sq!(h6),
            sq!(a7), sq!(b7), sq!(c7), sq!(d7), sq!(e7), sq!(f7), sq!(g7), sq!(h7),
            sq!(a8), sq!(b8), sq!(c8), sq!(d8), sq!(e8), sq!(f8), sq!(g8), sq!(h8),
        ];
        // Written in index order, so sq! must agree with Square::all().
        assert!(all.into_iter().eq(Square::all()));
    }

    #[test]
    fn sq_in_const_context() {
        const E4: Square = sq!(e4);
        assert_eq!((E4.file(), E4.rank()), (4, 3));
    }

    #[test]
    fn from_algebraic_rejects_bad_names() {
        for bad in ["", "e", "e44", "z9", "i1", "a0", "a9", "E4"] {
            assert_eq!(Square::from_algebraic(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn pst_puts_rank_eight_first() {
        // Each square's value encodes its name: 10 * file + rank (1-based).
        const T: Pst = pst![
            [18, 28, 38, 48, 58, 68, 78, 88],
            [17, 27, 37, 47, 57, 67, 77, 87],
            [16, 26, 36, 46, 56, 66, 76, 86],
            [15, 25, 35, 45, 55, 65, 75, 85],
            [14, 24, 34, 44, 54, 64, 74, 84],
            [13, 23, 33, 43, 53, 63, 73, 83],
            [12, 22, 32, 42, 52, 62, 72, 82],
            [11, 21, 31, 41, 51, 61, 71, 81],
        ];
        for sq in Square::all() {
            let expected = 10 * (i16::from(sq.file()) + 1) + i16::from(sq.rank()) + 1;
            assert_eq!(T[sq.index()], expected, "{sq}");
        }
    }

    #[test]
    fn pst_accepts_trailing_commas_and_expressions() {
        let t: Pst = pst![
            [1, 1, 1, 1, 1, 1, 1, 1,],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [-5, 2 * 3, 0, 0, 0, 0, 0, 0],
        ];
        assert_eq!(t[sq!(h8).index()], 1);
        assert_eq!(t[sq!(a1).index()], -5);
        assert_eq!(t[sq!(b1).index()], 6);
    }
}
