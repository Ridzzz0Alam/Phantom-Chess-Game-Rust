# AGENTS.md — phantom-chess

Instructions for any coding agent working in this repo (GLM, DeepSeek, Claude, etc.).
The full roadmap is in **PLAN.md**. Read only the phase you are working on, not the whole file.

## Project in one paragraph
A complete chess game in Rust for a university course. `chess-core` is a library with the rules engine;
the turn order is enforced with **phantom types** (`Game<WhiteToMove>` / `Game<BlackToMove>`).
`chess-tui` is the terminal front-end (Ratatui). `chess-gui` is the Slint GUI for Assignment 2.
The code must visibly demonstrate: phantom types, TryFrom/TryInto, error handling, closures/HOFs,
and later Arc, while let, macro_rules!.

## Commands
```bash
cargo build --workspace
cargo test --workspace                       # fast tests
cargo test -p chess-core --release -- --ignored   # slow perft tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo run -p chess-tui
cargo run -p chess-gui                       # Assignment 2
```
Before saying a task is done, **all four must pass**: build, test, clippy, fmt.

## Picking a task
- Tasks in PLAN.md are tagged **[Light]** or **[Claude]**.
- If you are not Claude: **only do [Light] tasks.** If a [Light] task turns out to need changes
  to `movegen.rs`, `game.rs`, or any public signature in `chess-core`, stop and write the reason
  in the PLAN.md Progress Log instead of improvising.
- Do one task per session/request. Keep diffs small.

## Hard rules
1. **No chess crates** (`chess`, `shakmaty`, `pleco`, …). The engine is hand-written.
2. **No `thiserror` / `anyhow` in `chess-core`.** Errors are hand-written enums with `Display` + `std::error::Error`.
3. **No `unwrap()` / `expect()` / `panic!` in non-test `chess-core` code.** Return `Result`/`Option`.
4. `#![forbid(unsafe_code)]` stays in `chess-core`.
5. Never make `Game<S>`'s fields public or add a way to change `side_to_move` from outside. That would break the typestate guarantee.
6. Add dependencies with `cargo add`, not by hand-editing versions.
7. Don't refactor code outside the task's scope. Don't rename public items.
8. Don't delete or weaken existing tests to make them pass. Perft numbers in PLAN.md are ground truth.

## Code conventions
- Edition 2024, `rustfmt` defaults.
- One concept per module (see layout in PLAN.md §2). Files under ~400 lines.
- `Square(u8)`: 0 = a1, 7 = h1, 56 = a8, 63 = h8. Use `Square::new(file, rank)` / `offset(df, dr)`, never raw index arithmetic outside `square.rs`.
- Public items get a `///` doc comment; one-line is fine.
- Tests: unit tests in a `#[cfg(test)] mod tests` at the bottom of the file; scenario tests in `chess-core/tests/`.
- Test names describe behaviour: `castling_through_attacked_square_is_illegal`.
- TUI: never panic on user input; show errors in the status bar.

## Where things are
| What | File |
|---|---|
| Square / piece / move types + `TryFrom` | `chess-core/src/{square,piece,mv}.rs` |
| Errors | `chess-core/src/error.rs` |
| FEN parse/print | `chess-core/src/fen.rs` |
| Move generation, legality, perft | `chess-core/src/movegen.rs` |
| Phantom-type game API, `AnyGame` | `chess-core/src/game.rs` |
| TUI state / drawing / keys / save | `chess-tui/src/{app,ui,input,save}.rs` |
| GUI (A2) | `chess-gui/ui/board.slint`, `chess-gui/src/*.rs` |

## Git & GitHub
- Remote: `origin` = `git@github.com:Ridzzz0Alam/Phantom-Chess-Game-Rust.git`, branch `main`.
- Commit or push only when the user asks. Run the four checks (build, test, clippy, fmt) before committing.
- Commits are authored **only** by the repo owner's own git identity (`git config user.name` / `user.email`).
  Never change that identity and never pass `--author`.
- **No AI attribution anywhere:** no `Co-Authored-By:` trailers, no "Generated with …" lines, no model
  names in commit messages, PR titles or PR descriptions. AI agents must not show up as co-authors or in
  the repository's Contributors list.
- Never commit secrets or local tool settings (`.env`, `.claude/settings.local.json`, …).

## When you finish
1. Tick the task's checkbox in PLAN.md.
2. Add one line to PLAN.md → Progress Log: `YYYY-MM-DD | <model> | <phase.task> | <short note>`.
3. Summarise what changed in ≤5 bullet points. Do not paste whole files back.
