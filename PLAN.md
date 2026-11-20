# PLAN.md — phantom-chess: Chess in Rust with Phantom-Type Turns

> A complete chess game written in Rust. **Assignment 1** is a terminal game (TUI, Ratatui).
> **Assignment 2** is the same engine with a GUI (Slint), an AI opponent on a background thread,
> and the extra techniques listed below.
>
> The headline feature is **phantom-type turns**: whose turn it is is part of the *type*
> (`Game<WhiteToMove>` / `Game<BlackToMove>`), so the compiler rejects code that lets the
> same side move twice.

---

## 0. How to use this file (for AI agents)

- Work **one phase at a time, top to bottom**. Do not start a phase until the previous phase's
  acceptance criteria pass.
- Every task has a tag:
  - **[Claude]**: design-heavy or tricky (type-level API, move legality, perft debugging, threads, GUI wiring).
  - **[Light]**: mechanical and well specified (boilerplate, Display impls, tests from given data, docs, rendering tweaks). Fine for GLM / DeepSeek.
- When you finish a task, tick its checkbox `[x]` and add one line to the **Progress Log** at the bottom.
- Do **not** change the public API of `chess-core` in a [Light] task. If it seems necessary, stop and write a note in the Progress Log instead.

---

## 1. Assignment requirements → where they live

### Required techniques (need ≥2 in A1, +2 more in A2)

| Technique | Assignment | Where in the code |
|---|---|---|
| **Phantom type parameters** | A1 | `Game<S: Side>` with `PhantomData<S>`, so turn order is enforced at compile time (`chess-core/src/game.rs`) |
| **TryFrom / TryInto** | A1 | `Square: TryFrom<&str>` (`"e4"`), `Square: TryFrom<u8>`, `Move: TryFrom<&str>` (`"e7e8q"`), `Position: TryFrom<&str>` (FEN), `PieceKind: TryFrom<char>` |
| **Error handling** | A1 | Custom `ParseError`, `MoveError`, `FenError`, `AppError` enums with `Display` + `std::error::Error` + `From` conversions, used with `?` throughout |
| **Closures / Higher-order functions** | A1 (bonus) | Move-gen pipelines: `pseudo_legal().filter(\|m\| !leaves_king_in_check(m))`, key-binding table `Vec<(KeyCode, Box<dyn Fn(&mut App)>)>` |
| **Arc** | A2 | AI search on a worker thread: `Arc<AtomicBool>` stop flag + `Arc<Mutex<SearchInfo>>` shared with the GUI |
| **while let** | A2 | `while let Ok(msg) = rx.recv()` in the AI worker loop; `while let Some(pos) = undo_stack.pop()` for "undo to start" |
| **macro_rules!** | A2 | `sq!(e4)` square literal, `perft_tests! { ... }` to generate test functions, `pst!` for piece-square tables |

A1 uses 4 techniques (more than the 2 required). A2 adds 3 new ones. Closures/HOF are a bonus in A1, so A2 is safely "+2" even if a grader doesn't count them.

### Course topics coverage (for the report / defence)

| Course topic | Where it shows up |
|---|---|
| Primitives, Custom types | `Color`, `PieceKind`, `Piece`, `Square(u8)`, `Move`, `CastlingRights` (bitflags as `u8`) |
| Variable bindings, Types, Conversion | `From`/`TryFrom` impls, `as` casts on square indices |
| Expressions, Flow of control | `match` on pieces/moves, labelled loops in sliding-move generation |
| Functions, Modules | One module per concern: `square`, `piece`, `board`, `movegen`, `game`, `fen`, `error` |
| Crates, Cargo | Cargo **workspace**: `chess-core` (lib), `chess-tui` (bin), `chess-gui` (bin, A2) |
| Attributes, Generics | `#[derive]`, `#[cfg(test)]`, `#[must_use]`, generic `Game<S>` |
| Scoping rules (ownership/borrowing), Traits | `make_move(self, …)` *consumes* the game (ownership moves the turn); sealed trait `Side` |
| macro_rules!, Error handling | See table above |
| Std library types / misc | `Vec`, `HashMap` (repetition count), `Option`/`Result`, threads, channels, `Arc`, `Mutex`, file I/O for save/load |
| Testing | Unit tests, doc tests, integration tests in `tests/`, **perft** correctness tests |
| Unsafe | Not needed. Mention in the report that the crate is `#![forbid(unsafe_code)]` and why |
| Compatibility, Meta | Edition 2024, `cargo doc`, `cargo clippy`, `cargo fmt`, rustdoc comments |

---

## 2. Tech stack & setup

- Rust stable, **edition 2024** (`rustup update stable`).
- VS Code extensions: `rust-analyzer`, `CodeLLDB` (debugging), `Even Better TOML`.
- Crates (add with `cargo add`, do not hand-pin versions):
  - `chess-tui`: `ratatui` (uses its bundled `crossterm` backend)
  - `chess-gui` (A2): `slint` (+ `slint-build` as build-dependency)
  - Dev-only (optional): none needed
- **No chess libraries** (`chess`, `shakmaty`, `pleco` etc.). The rules engine is the point of the assignment.
- **No `thiserror` / `anyhow` in `chess-core`.** Errors are hand-written to demonstrate error handling. (`anyhow` is allowed in the binaries' `main` only if wanted.)

### Workspace layout

```
phantom-chess/
├── Cargo.toml                # [workspace] members = ["chess-core", "chess-tui", "chess-gui"]
├── PLAN.md  CLAUDE.md  AGENTS.md  README.md
├── chess-core/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs            # #![forbid(unsafe_code)], pub use of the public API
│   │   ├── color.rs          # Color { White, Black }, opposite()
│   │   ├── piece.rs          # PieceKind, Piece, TryFrom<char>, to_unicode()
│   │   ├── square.rs         # Square(u8), TryFrom<&str>, TryFrom<u8>, file(), rank(), offset()
│   │   ├── mv.rs             # Move { from, to, promotion: Option<PieceKind> }, MoveKind flags, TryFrom<&str>
│   │   ├── board.rs          # Board: [Option<Piece>; 64]
│   │   ├── position.rs       # Position: board + side + castling + ep + clocks
│   │   ├── fen.rs            # TryFrom<&str> for Position, to_fen()
│   │   ├── movegen.rs        # pseudo-legal + legal generation, is_attacked()
│   │   ├── game.rs           # Side trait, WhiteToMove/BlackToMove, Game<S>, AnyGame, GameStatus
│   │   ├── error.rs          # ParseError, FenError, MoveError
│   │   ├── macros.rs         # (A2) sq!, perft_tests!
│   │   └── ai.rs             # (A2) evaluation + alpha-beta search
│   └── tests/
│       ├── perft.rs
│       ├── rules.rs          # castling, en passant, promotion, mate, stalemate
│       └── typestate.rs      # compile-fail docs / examples
├── chess-tui/
│   └── src/ main.rs, app.rs, ui.rs, input.rs, save.rs
└── chess-gui/                # (A2)
    ├── build.rs
    ├── ui/board.slint
    └── src/ main.rs, bridge.rs, ai_worker.rs
```

---

## 3. Core design (read before writing code)

### 3.1 Basic types
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color { White, Black }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceKind { Pawn, Knight, Bishop, Rook, Queen, King }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece { pub color: Color, pub kind: PieceKind }

/// 0 = a1, 7 = h1, 56 = a8, 63 = h8
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Square(u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move { pub from: Square, pub to: Square, pub promotion: Option<PieceKind> }
```
- `Board` is a **mailbox** `[Option<Piece>; 64]`. It's simple and readable, and fast enough (no bitboards).
- `Position` holds `board`, `side_to_move: Color`, `castling: CastlingRights`, `en_passant: Option<Square>`, `halfmove_clock: u16`, `fullmove_number: u16`.
- `Position` is the **untyped** runtime state and is fully `Clone`. Move generation lives here.

### 3.2 The phantom-type layer (headline feature)
```rust
mod sealed { pub trait Sealed {} }

pub trait Side: sealed::Sealed {
    const COLOR: Color;
    type Next: Side;
}
pub enum WhiteToMove {}   // uninhabited marker types
pub enum BlackToMove {}
impl Side for WhiteToMove { const COLOR: Color = Color::White; type Next = BlackToMove; }
impl Side for BlackToMove { const COLOR: Color = Color::Black; type Next = WhiteToMove; }

pub struct Game<S: Side> {
    position: Position,          // private, so the typestate can't be bypassed
    history: Vec<Move>,
    repetitions: HashMap<u64, u8>,
    _turn: PhantomData<S>,
}

pub enum Played<N: Side> {
    Ongoing(Game<N>),
    Finished(FinishedGame),      // checkmate / stalemate / draw
}

impl<S: Side> Game<S> {
    /// Consumes `self`: once White has moved, the `Game<WhiteToMove>` value is gone.
    pub fn play(self, mv: Move) -> Result<Played<S::Next>, (Self, MoveError)>;
    pub fn legal_moves(&self) -> Vec<Move>;
    pub fn position(&self) -> &Position;
}

impl Game<WhiteToMove> { pub fn new() -> Self; }
```
Key points to get right:
- `play` **consumes** `self` and on error **gives the game back** `(Self, MoveError)`, so an illegal move never loses state.
- `Position.side_to_move` must always equal `S::COLOR`. Add a `debug_assert!` in `play`.
- The UI loop doesn't know the turn at compile time, so we provide a runtime wrapper:
  ```rust
  pub enum AnyGame {
      White(Game<WhiteToMove>),
      Black(Game<BlackToMove>),
      Over(FinishedGame),
  }
  impl AnyGame { pub fn play(self, mv: Move) -> Result<AnyGame, (AnyGame, MoveError)>; }
  ```
  `AnyGame` is the only thing the UI stores. Explain in the README that the *engine* is typestate-safe and `AnyGame` is the boundary to the dynamic world.
- Loading a FEN returns `AnyGame` (the side to move is only known at runtime).
- Put a **`compile_fail` doc test** on `Game::play` that shows `game.play(m1); game.play(m2);` failing (use-after-move). This proves the feature to the examiner.

### 3.3 Errors
```rust
pub enum ParseError { BadSquare(String), BadPiece(char), BadMove(String) }
pub enum FenError   { WrongFieldCount(usize), BadBoard(String), BadSide(String), BadCastling(String), BadEnPassant(String), BadNumber(String) }
pub enum MoveError  { NoPieceOnSquare(Square), NotYourPiece(Square), IllegalMove(Move), LeavesKingInCheck(Move), MissingPromotion, GameOver }
```
All of them implement `Display` and `std::error::Error`. Add `impl From<ParseError> for FenError` where useful so `?` works. The TUI has its own `AppError` wrapping `io::Error`, `FenError`, `ParseError`, `MoveError`.

### 3.4 Rules scope (A1 must be **complete chess**)
- All piece moves, captures
- Castling (both sides; not out of / through / into check; rights lost on king/rook move or rook capture)
- En passant
- Promotion (to Q/R/B/N)
- Check, checkmate, stalemate
- Draws: 50-move rule, threefold repetition (hash the position via `std::hash` or a simple FEN-without-clocks string), insufficient material (K v K, K+B v K, K+N v K)
- Resign / offer-draw is a UI action

### 3.5 Legality strategy (simple and correct)
1. Generate pseudo-legal moves per piece (closures for direction lists).
2. For each, make it on a cloned `Position`, then reject if the own king `is_attacked`.
3. Castling: additionally check the king's start, transit and destination squares aren't attacked.

This is slower than bitboards but plenty fast for perft depth 4 and a depth-4 AI.

---

## 4. Phases — Assignment 1 (TUI)

### Phase 1 — Workspace & primitives
- [x] [Light] Create the workspace, three member crates (`chess-gui` can be an empty bin for now), `#![forbid(unsafe_code)]` in core, `.gitignore`, `rustfmt.toml` (default is fine).
- [x] [Light] `color.rs`, `piece.rs` with `TryFrom<char>` (FEN letters: `PNBRQK` / `pnbrqk`) and `to_unicode()` (♔♕♖♗♘♙ / ♚♛♜♝♞♟).
- [x] [Light] `square.rs`: `Square::new(file, rank) -> Option<Square>`, `file()`, `rank()`, `offset(df, dr) -> Option<Square>`, `Display` ("e4"), `TryFrom<&str>`, `TryFrom<u8>` (rejects ≥ 64).
- [x] [Light] `error.rs` with the enums from §3.3 + `Display` + `Error`.
- [x] [Light] `mv.rs`: `Move` + `TryFrom<&str>` for UCI-style `"e2e4"`, `"e7e8q"`; `Display`.
- [x] [Light] Unit tests for all `TryFrom` impls, including error cases.

**Acceptance:** `cargo test -p chess-core` green; `cargo clippy --workspace -- -D warnings` clean.

### Phase 2 — Board, Position, FEN
- [x] [Light] `board.rs`: `get`, `set`, `remove`, `iter()` over occupied squares, `find_king(color)`.
- [x] [Claude] `position.rs` + `CastlingRights` (4 bits in a `u8`) + `Position::startpos()`.
- [x] [Claude] `fen.rs`: `TryFrom<&str> for Position` with precise `FenError`s; `Position::to_fen()`.
- [x] [Light] Tests: startpos FEN round-trip, 5+ valid FENs round-trip, 6+ malformed FENs each give the expected error variant.

**Acceptance:** FEN round-trip tests pass.

### Phase 3 — Move generation & legality (hardest part)
- [x] [Claude] `movegen.rs`: pseudo-legal generation for all pieces (sliding pieces via direction arrays + closure/iterator), pawn pushes/double pushes/captures/promotions/en passant, castling.
- [x] [Claude] `is_attacked(square, by: Color)`.
- [x] [Claude] `Position::make_move(&mut self, mv)`: updates board, castling rights, en passant, clocks, side to move.
- [x] [Claude] `Position::legal_moves() -> Vec<Move>` using the strategy in §3.5.
- [x] [Claude] `perft(depth)` + `divide(depth)` helper for debugging.
- [x] [Light] `tests/perft.rs` with these known values:

| Position | FEN | d1 | d2 | d3 | d4 |
|---|---|---|---|---|---|
| Start | `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1` | 20 | 400 | 8902 | 197281 |
| Kiwipete | `r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1` | 48 | 2039 | 97862 | 4085603 |
| Position 3 | `8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1` | 14 | 191 | 2812 | 43238 |

  Mark depth-4 tests `#[ignore]` (run with `cargo test --release -- --ignored`).
- [x] [Light] `tests/rules.rs`: scenario tests for castling through check, en passant, all four promotions, fool's mate (checkmate), a known stalemate position.

**Acceptance:** all perft d1–d3 pass in debug mode; d4 passes in `--release`.

### Phase 4 — Typestate game layer
- [x] [Claude] `game.rs`: `Side`, markers, `Game<S>`, `Played<N>`, `FinishedGame`, `GameResult { WhiteWins(Reason), BlackWins(Reason), Draw(Reason) }`.
- [x] [Claude] Game-over detection: mate, stalemate, 50-move, threefold, insufficient material.
- [x] [Claude] `AnyGame` wrapper + `AnyGame::from_fen(&str) -> Result<AnyGame, FenError>`.
- [x] [Light] `compile_fail` doc test showing double-move is rejected; normal doc test showing a valid two-move sequence.
- [x] [Light] `tests/typestate.rs`: play fool's mate through `Game<_>` types → `Played::Finished` with `BlackWins(Checkmate)`.

**Acceptance:** `cargo test --workspace --doc` shows the `compile_fail` test passing.

### Phase 5 — Terminal UI (Ratatui)
- [ ] [Claude] `app.rs`: `App { game: Option<AnyGame>, cursor: Square, selected: Option<Square>, input: String, mode: Mode, message: Option<String>, flipped: bool, undo: Vec<String /*FEN*/> }`. Use `Option::take()` to move `AnyGame` in and out for `play`.
- [ ] [Light] `ui.rs`: layout with board (left), move list (right, SAN not required, UCI is fine), status bar (turn, check, result), input line.
- [ ] [Light] Board rendering: alternating square colours, unicode pieces, highlight cursor, selected square and legal destinations of the selected piece, last move.
- [ ] [Claude] `input.rs`: key-binding table as `Vec<(KeyCode, &'static str, fn(&mut App))>` or boxed closures (HOF technique), with help screen generated from the same table.
  - Arrows / `hjkl` move cursor, `Enter`/`Space` select & move, `Esc` cancel, `:` type a move (`e2e4`), `u` undo, `f` flip board, `n` new game, `s` save, `l` load, `r` resign, `?` help, `q` quit.
  - Promotion: popup asking Q/R/B/N.
- [ ] [Light] `save.rs`: save/load the current FEN to `saves/*.fen`. All errors go to the status bar, never panic.
- [ ] [Light] Terminal restore on panic (ratatui's `ratatui::init()` / `ratatui::restore()` handle this).

**Acceptance:** a full game can be played start to checkmate with keyboard only; illegal moves show a readable error message; save → quit → load restores the position.

### Phase 6 — A1 polish & hand-in
- [ ] [Light] README: build/run instructions, controls, screenshots/asciinema, **"Techniques used" table** (copy §1) with file/line links.
- [ ] [Light] Rustdoc on every public item in `chess-core`; `cargo doc --no-deps` builds without warnings.
- [ ] [Light] `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] [Claude] Final review pass: no `unwrap()`/`expect()` in non-test core code; the typestate can't be bypassed via public API.

**A1 Definition of Done:** all tests pass, clippy clean, a complete game is playable in the terminal, README explains the phantom-type design.

---

## 5. Phases — Assignment 2 (GUI + new techniques)

### Phase 7 — macro_rules!
- [ ] [Light] `sq!(e4)` → `Square` at compile time (match on ident tokens a1..h8, or `stringify!` + const parse).
- [ ] [Claude] `perft_tests! { startpos_d3: "<fen>", 3 => 8902, ... }` generating one `#[test]` per line; replace the hand-written perft tests.
- [ ] [Light] `pst!` macro to declare 8×8 piece-square tables readably (rank 8 at top) for the AI.

### Phase 8 — AI opponent (Arc, while let)
- [ ] [Claude] `ai.rs`: material + piece-square evaluation, negamax with alpha-beta, iterative deepening, MVV-LVA move ordering. Depth 3–5 is plenty.
- [ ] [Claude] Search takes `stop: &AtomicBool` and checks it periodically.
- [ ] [Claude] `chess-gui/src/ai_worker.rs`: worker thread holding `Arc<AtomicBool>` (stop) and `Arc<Mutex<SearchInfo>>` (depth, best move so far, nodes) shared with the UI; loop `while let Ok(req) = rx.recv() { ... }`; replies over a second channel.
- [ ] [Light] Optional: plug the AI into the TUI too (`a` toggles "play vs computer").

### Phase 9 — Slint GUI
- [ ] [Claude] `board.slint`: 8×8 grid of squares (component with `piece`, `highlighted`, `selected` properties), side panel with move list, status, buttons (New, Undo, Flip, Save, Load, Play vs AI toggle, difficulty slider).
- [ ] [Claude] `bridge.rs`: map `AnyGame` → Slint `VecModel` of squares; click handlers (select/move), promotion dialog; AI results posted back via `slint::invoke_from_event_loop`.
- [ ] [Light] Piece graphics: use Unicode glyphs in a large font first; optionally swap to free SVG pieces (e.g. the CC-licensed "cburnett" set — check the license and credit it in README).
- [ ] [Light] Polish: last-move highlight, check highlight on king, game-over overlay.

**Fallback:** if Slint gives trouble, use **Dioxus desktop**. Only `chess-gui` changes; `chess-core` stays the same.

### Phase 10 — A2 hand-in
- [ ] [Light] Update README techniques table (now 7 techniques), GUI screenshots.
- [ ] [Light] Tests for macros (`sq!` values), AI sanity tests (finds mate-in-1, doesn't hang the queen in a simple position), stop-flag test (search returns quickly after stop).
- [ ] [Claude] Final review: no deadlocks (never hold the `Mutex` across a search iteration), UI never blocks on the AI.

---

## 6. Stretch goals (only if time remains)
- SAN move notation (`Nf3`, `exd5`, `O-O`) in the move list
- PGN export
- Clocks / time control
- Zobrist hashing for repetition detection
- Opening book from a tiny embedded table

---

## 7. Demo script (for the defence)
1. Show `Game<WhiteToMove>` → `play` → `Game<BlackToMove>` in code; run the `compile_fail` doc test.
2. Show `Square::try_from("z9")` error and FEN error messages in the app.
3. Play fool's mate in the TUI.
4. Run `cargo test --release -- --ignored` (perft d4).
5. (A2) GUI vs AI; show the worker thread, `Arc`, the stop button, `while let` loop, and the `sq!`/`perft_tests!` macros.

---

## 8. Progress Log
<!-- One line per finished task: YYYY-MM-DD | model | phase.task | note -->
2026-11-20 | claude-opus-5-5 | 1.1–1.6 | Workspace (resolver 3, edition 2024), color/piece/square/error/mv + 28 unit tests + 2 doc tests; all four checks green.
2026-11-20 | claude-opus-5-5 | 1 note | Extras beyond plan: `Square::{index, all, COUNT}`, `FromStr` for Square/Move, `Color::{ALL, back_rank, pawn_direction}`, `PieceKind::{ALL, PROMOTIONS, to_char, is_promotion_target}`, `Piece::{new, to_fen_char}`. `From<ParseError> for FenError` deferred to Phase 2 (fen.rs decides the mapping). `MoveKind` flags not added (not needed yet).
2026-11-20 | claude-opus-5-5 | 2.1–2.4 | board.rs, position.rs (CastlingRights u8 bits + CastleSide), fen.rs (TryFrom/FromStr/to_fen, STARTPOS_FEN), tests/fen.rs (8 valid round-trips, 9 BadBoard + every other variant). 52 unit/integration + 3 doc tests green.
2026-11-20 | claude-opus-5-5 | 2 note | FEN parsing is strict: exactly 6 fields; one king per side; no back-rank pawns; castling letters need king+rook on start squares; ep square needs a just-pushed enemy pawn; fullmove ≥ 1. Not yet checked: side NOT to move being in check (needs is_attacked, add in Phase 3). Position fields are pub(crate) with getters only. `From<ParseError> for FenError` added (BadPiece→BadBoard, BadSquare→BadEnPassant). `Square::at` is a pub(crate) masked const constructor for in-crate constants.
2026-11-20 | claude-opus-5-5 | 3.1–3.7 | movegen.rs (is_attacked, pseudo_legal_moves, legal_moves, legal_moves_from, check_move, perft, divide) + Position::make_move/apply. Perft d1–d3 green in debug for 6 positions (plan's 3 + CPW positions 4–6); d4 for 5 positions green in --release (0.8 s). tests/rules.rs: 22 scenarios.
2026-11-20 | claude-opus-5-5 | 3 note | API change vs plan: `Position::make_move(&mut self, mv) -> Result<(), MoveError>` validates (via `check_move`) and leaves the position unchanged on error; the unchecked `apply` is pub(crate). Castling through check is reported as `IllegalMove` (castle is never generated), not `LeavesKingInCheck`. FEN parsing now also rejects "side not to move is in check" (BadBoard). For Phase 4: `game.rs` should call `position.make_move` and map nothing — MoveError variants already match. En-passant square is set after EVERY double push (perft convention); for threefold repetition, build the key from board+side+castling+ep-only-if-a-legal-ep-capture-exists.
2026-11-20 | claude-opus-5-5 | 4.1–4.5 | game.rs (sealed Side, WhiteToMove/BlackToMove, Game<S>, Played<N>, FinishedGame, resign/agree_draw), game/any_game.rs (AnyGame + From impls via Side::into_any), outcome.rs (GameResult/WinReason/DrawReason, is_insufficient_material, repetition_key, game_result). 4 compile_fail doc tests (double move E0382, wrong side E0308, private field E0616, sealed trait E0277); tests/typestate.rs 12 tests.
2026-11-20 | claude-opus-5-5 | 4 note | Stable rustdoc does NOT check compile_fail error codes (only nightly does), so each compile_fail snippet was verified manually to fail with exactly its labelled code. Reason enum split into WinReason {Checkmate, Resignation} + DrawReason {Stalemate, FiftyMoveRule, ThreefoldRepetition, InsufficientMaterial, Agreement}. Threefold/50-move end the game automatically (no claim step). Insufficient material also covers K+B v K+B same-coloured bishops. `#[expect(clippy::result_large_err)]` on both `play` fns: the by-value error is the required API. For Phase 5: UI stores Option<AnyGame>; use AnyGame::{play, legal_moves_from, result, side_to_move, resign, agree_draw, to_fen, from_fen}.
