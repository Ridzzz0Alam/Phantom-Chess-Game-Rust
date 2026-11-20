# phantom-chess

A complete chess game written in Rust, with a hand-written rules engine and a terminal UI.

Its headline feature is **phantom-type turns**: whose turn it is is part of the *type*.
A `Game<WhiteToMove>` can only turn into a `Game<BlackToMove>` (or a finished game) and back again,
and playing a move *consumes* the old game. So code in which the same side moves twice does not compile.

```text
┌ phantom-chess ───────────────────────────────────────────┐┌ Status ─────────────────────────┐
│8    ♜      ♞      ♝      ♛      ♚      ♝      ♞      ♜   ││Turn: Black                      │
│                                                          ││Move 2  (50-move clock 1)        │
│7    ♟      ♟      ♟      ♟             ♟      ♟      ♟   │└─────────────────────────────────┘
│                                                          │┌ Moves ──────────────────────────┐
│6    ·             ·                                      ││  1. e2e4  e7e5                  │
│                                                          ││  2. g1f3                        │
│5                                ♟                        ││                                 │
│                                                          ││                                 │
│4                                ♟                        ││                                 │
│                                                          ││                                 │
│3                                       ♞                 ││                                 │
│                                                          ││                                 │
│2    ♟      ♟      ♟      ♟             ♟      ♟      ♟   ││                                 │
│                                                          ││                                 │
│1    ♜      ♞      ♝      ♛      ♚      ♝             ♜   ││                                 │
│     a      b      c      d      e      f      g      h   ││                                 │
└──────────────────────────────────────────────────────────┘└─────────────────────────────────┘
arrows/hjkl move · Enter select · : type move · u undo · ? help · q quit
```
<sub>Text capture of the terminal UI (colours not shown). Black's knight on b8 is selected; the dots mark its legal moves.</sub>

## Contents
- [Quick start](#quick-start)
- [Controls](#controls)
- [The phantom-type design](#the-phantom-type-design)
- [Techniques used](#techniques-used)
- [Architecture](#architecture)
- [Rules implemented](#rules-implemented)
- [Testing](#testing)
- [Roadmap](#roadmap)

## Quick start

You need a stable Rust toolchain that supports **edition 2024** (`rustup update stable`).

```bash
cargo run -p chess-tui                      # play from the starting position
cargo run -p chess-tui -- "<FEN>"           # or start from any position
cargo test --workspace                      # run the test suite
cargo doc -p chess-core --no-deps --open    # browse the engine's API docs
```

A terminal with true colour and Unicode support (e.g. Windows Terminal, iTerm2, most Linux terminals)
looks best. If the chess symbols render badly, press `g` to switch to letters.

## Controls

| Key | Action |
|---|---|
| `←↑↓→` / `h j k l` | Move the cursor |
| `Enter` / `Space` | Select a piece, then press again on a target square to move it |
| `Esc` | Cancel the selection or prompt |
| `:` | Type a move in coordinate notation (`e2e4`, promotion `e7e8q`), or `fen <FEN>` to load a position |
| `u` / `U` | Undo the last action / undo everything |
| `f` | Flip the board |
| `g` | Switch between chess symbols and letters |
| `n` | New game |
| `s` / `o` | Save the game to `saves/<name>.fen` / open (load) a save |
| `r` / `d` | Resign (side to move) / agree to a draw |
| `?` | Help screen (generated from the same key table that handles the keys) |
| `q` / `Ctrl-C` | Quit |

When a pawn reaches the last rank, a popup asks for the promotion piece (`q`, `r`, `b` or `n`).
Every rejected move or bad input is explained in the status bar, e.g. *"e2e5 is not a legal move"*,
*"the piece on e7 is not yours"*, *"invalid FEN board: expected 8 ranks, found 2"*. Nothing panics.

## The phantom-type design

```rust
pub enum WhiteToMove {}            // uninhabited marker types: they exist only at type level
pub enum BlackToMove {}

pub trait Side: sealed::Sealed {   // sealed: no third side can be added outside the crate
    const COLOR: Color;
    type Next: Side;               // WhiteToMove::Next = BlackToMove and vice versa
}

pub struct Game<S: Side> {
    position: Position,            // all fields private
    history: Vec<Move>,
    repetitions: HashMap<u64, u8>,
    _turn: PhantomData<S>,         // zero-sized; only carries the type parameter
}

impl<S: Side> Game<S> {
    pub fn play(self, mv: Move) -> Result<Played<S::Next>, (Self, MoveError)>;
}
```

- **`play` takes `self` by value.** After White moves, the `Game<WhiteToMove>` value is gone, and what
  you get back is a `Game<BlackToMove>`. Using the old value again is a compile error (`E0382`).
- **Illegal moves lose nothing.** On error the unchanged game is handed back together with the reason:
  `Err((game, MoveError::LeavesKingInCheck(mv)))`.
- **The typestate can't be bypassed.** The fields are private, the position is only reachable as `&Position`,
  and the only constructors are `Game::new()` (White to move), loading a FEN, and `play`.
- **`AnyGame` is the runtime boundary.** A UI doesn't know at compile time whose turn it is, so it stores
  `AnyGame { White(Game<WhiteToMove>), Black(Game<BlackToMove>), Over(FinishedGame) }`.
  The engine underneath is still typestate-checked: `AnyGame::play` just dispatches to `Game::play`.
  The TUI keeps it in an `Option<AnyGame>` and uses `take()` to move it into `play`.

Four doc tests prove that wrong code is rejected. Run them with `cargo test -p chess-core --doc`:

| Snippet | Fails with |
|---|---|
| `game.play(e4); game.play(d4);` (same side moves twice) | `E0382` use of moved value |
| passing `Game::new()` to a function expecting `Game<BlackToMove>` | `E0308` mismatched types |
| reading `game.position` directly | `E0616` private field |
| `impl Side for MyOwnSide` | `E0277` trait `Sealed` not implemented |

> Stable `rustdoc` checks that each snippet fails to compile, but not the exact error code (only nightly
> checks codes). Each snippet was also compiled on its own to confirm it fails with exactly the code listed.

## Techniques used

| Technique | Where |
|---|---|
| **Phantom type parameters** | [`Game<S>` with `PhantomData<S>`](chess-core/src/game.rs#L86), [sealed `Side` trait](chess-core/src/game.rs#L40), [`play` consumes `self`](chess-core/src/game.rs#L215), [`compile_fail` proof](chess-core/src/game.rs#L195), [`AnyGame` boundary](chess-core/src/game/any_game.rs#L19) |
| **TryFrom / TryInto** | [`Square: TryFrom<&str>`](chess-core/src/square.rs#L81) (`"e4"`), [`Square: TryFrom<u8>`](chess-core/src/square.rs#L97), [`Move: TryFrom<&str>`](chess-core/src/mv.rs#L52) (`"e7e8q"`), [`PieceKind: TryFrom<char>`](chess-core/src/piece.rs#L67), [`Piece: TryFrom<char>`](chess-core/src/piece.rs#L143), [`Position: TryFrom<&str>`](chess-core/src/fen.rs#L28) (FEN), [`text.try_into()?` in the TUI](chess-tui/src/prompt.rs#L69) |
| **Error handling** | Hand-written [`ParseError`](chess-core/src/error.rs#L10), [`FenError`](chess-core/src/error.rs#L35), [`MoveError`](chess-core/src/error.rs#L84) with `Display` + `std::error::Error`; [`From<ParseError> for FenError`](chess-core/src/error.rs#L72) so [`Piece::try_from(c)?`](chess-core/src/fen.rs#L140) works inside FEN parsing; the TUI's [`AppError`](chess-tui/src/error.rs#L9) wraps `io::Error` + all engine errors via [`From`](chess-tui/src/error.rs#L55); [`run_command`](chess-tui/src/prompt.rs#L63) uses `?` throughout |
| **Closures / higher-order functions** | Legal moves = [pseudo-legal moves filtered by a closure](chess-core/src/movegen.rs#L107); [iterator pipeline for knight/king moves](chess-core/src/movegen.rs#L179); [promotion-expanding closure](chess-core/src/movegen.rs#L213); key bindings are a [table of boxed closures](chess-tui/src/input.rs#L12) built by the HOF [`move_cursor` that returns a closure](chess-tui/src/input.rs#L42); the help screen is [generated from that table](chess-tui/src/ui.rs#L283); the app's [`update` takes a closure](chess-tui/src/app.rs#L136) to move the game in and out of `Option` |

Not required, but used: `while let` ([undo everything](chess-tui/src/app.rs#L300), ray walking in move generation),
`HashMap` for repetition counts, `std::hash` for position keys, `#[must_use]`, `#[expect(lint, reason)]`,
let-chains (edition 2024), and `#![forbid(unsafe_code)]` in the engine.

## Architecture

```
phantom-chess/                Cargo workspace
├── chess-core/               the rules engine (library, no dependencies, no unsafe)
│   ├── src/
│   │   ├── color.rs piece.rs square.rs mv.rs   primitive types + TryFrom parsing
│   │   ├── error.rs                            ParseError, FenError, MoveError
│   │   ├── board.rs position.rs                64-square mailbox board; full game state
│   │   ├── fen.rs                              FEN parsing (strict validation) and printing
│   │   ├── movegen.rs                          attacks, move generation, legality, perft
│   │   ├── outcome.rs                          results; mate/stalemate/draw rules
│   │   ├── game.rs  game/any_game.rs           phantom-type layer + runtime wrapper
│   └── tests/                                  fen, perft, rules, typestate
├── chess-tui/                terminal front-end (Ratatui)
│   └── src/  main.rs app.rs prompt.rs input.rs ui.rs save.rs error.rs
└── chess-gui/                graphical front-end (Assignment 2, not started)
```

There are three layers:
1. **`Position`**: the untyped engine (board, side to move, castling rights, en passant, clocks).
   Its public `make_move` checks the move and returns `Result<(), MoveError>`, so a `Position` can never be
   put into an illegal state from outside. The fast unchecked version is crate-private.
2. **`Game<S>`**: a thin typestate wrapper over `Position` that adds history and repetition counting.
   It contains no rules logic of its own.
3. **`AnyGame`**: the runtime enum the UI stores.

Legality uses a simple, verifiable strategy: generate pseudo-legal moves, play each on a copy, and
discard those that leave the mover's king attacked. Castling additionally checks that the king's start,
transit and destination squares are safe.

## Rules implemented

Everything in standard chess:
- all piece moves and captures
- castling: both sides, never out of, through or into check; rights are lost on king or rook moves and on rook captures
- en passant, including the rare case where it would expose the king along the rank
- promotion to queen, rook, bishop or knight
- check, checkmate and stalemate
- draws by the 50-move rule, threefold repetition and insufficient material
  (K v K, K+B v K, K+N v K, and K+B v K+B with same-coloured bishops); repetition only counts an
  en-passant square when an en-passant capture is actually possible
- resignation and draw by agreement as UI actions

FEN loading is strict and rejects impossible positions with a precise error:
- a missing or extra king
- a pawn on the first or last rank
- castling rights without the king and rook on their start squares
- an en-passant square with no pawn that just moved two squares
- the side *not* to move being in check

## Testing

```bash
cargo test --workspace                                # all fast tests (unit, integration, doc)
cargo test -p chess-core --release -- --ignored       # slow depth-4 perft tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

- **Perft** counts every legal move sequence to a fixed depth and compares the totals with
  published values. All of these match:

  | Position | Depth 1 | Depth 2 | Depth 3 | Depth 4 |
  |---|---|---|---|---|
  | Start position | 20 | 400 | 8,902 | 197,281 |
  | "Kiwipete" | 48 | 2,039 | 97,862 | 4,085,603 |
  | Position 3 | 14 | 191 | 2,812 | 43,238 |
  | Position 4 | 6 | 264 | 9,467 | 422,333 |
  | Position 5 | 44 | 1,486 | 62,379 | 2,103,487 |
  | Position 6 | 46 | 2,079 | 89,890 | not tested |

- **Rule scenarios** ([`tests/rules.rs`](chess-core/tests/rules.rs)) cover castling through, into and out of
  check, en passant, all four promotions, fool's mate, stalemate, and every `MoveError` reason.
- **Typestate** ([`tests/typestate.rs`](chess-core/tests/typestate.rs)) plays fool's mate through
  `Game<WhiteToMove>` → `Game<BlackToMove>` → … → `BlackWins(Checkmate)`, and tests each draw rule.
- **FEN** ([`tests/fen.rs`](chess-core/tests/fen.rs)) round-trips valid positions and checks that every kind of
  malformed FEN produces the right error variant.
- **TUI** unit tests drive the app through its key table and render frames with Ratatui's `TestBackend`
  at several terminal sizes.

## Roadmap

- **Assignment 1** (this release): the engine plus the terminal UI.
- **Assignment 2**:
  - `macro_rules!` macros: `sq!(e4)` square literals, generated perft tests, piece-square tables
  - an alpha-beta AI opponent on a worker thread (`Arc<AtomicBool>` stop flag, `Arc<Mutex<…>>` progress, `while let` message loop)
  - a Slint GUI
