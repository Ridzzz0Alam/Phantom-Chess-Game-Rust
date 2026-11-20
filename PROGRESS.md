# PROGRESS.md: where we left off

> **Read this first when resuming.** It is the short handoff note between work sessions.
> PLAN.md is the full roadmap (checkboxes + Progress Log); this file says *what state the code is in
> right now* and *what to do next*. Update the "Current state" and "Next steps" sections at the end
> of every session.

_Last updated: 2026-11-20_

## Current state

| Phase | Status |
|---|---|
| 1. Workspace & primitives | ✅ done, pushed |
| 2. Board, Position, FEN | ✅ done, pushed |
| 3. Move generation, perft | ✅ done, pushed (perft d1–d4 all match) |
| 4. Phantom-type game layer | ✅ done, pushed (`09f5944`) |
| 5. Terminal UI (Ratatui) | ✅ done, pushed (`438f3b0`) |
| 6. A1 polish & hand-in | ✅ done (README, docs, review), **not committed yet** |
| 7. macro_rules! (`sq!`, `perft_tests!`, `pst!`) | ⬜ next |
| 8. AI opponent (Arc, while let) | ⬜ |
| 9. Slint GUI | ⬜ |
| 10. A2 hand-in | ⬜ |

**Assignment 1 is functionally complete.** 133 tests pass (+5 slow perft tests in `--release -- --ignored`);
clippy, fmt and `cargo doc` (warnings as errors) are clean.

### How to run
- `cargo run -p chess-tui` (or `cargo run -p chess-tui -- "<FEN>"`). Press `?` in the app for keys.
- Keys: arrows/hjkl cursor, Enter/Space select+move, `:` type move or `fen <FEN>`, `u` undo, `U` undo all,
  `f` flip, `g` glyphs/letters, `n` new, `s` save, `o` load, `r` resign, `d` draw, `?` help, `q` quit.

## Next steps
1. Owner play-tests the TUI in a real terminal (full game to mate; illegal move message; save → quit → load).
   Optionally add a real screenshot to the README (currently a text capture).
2. Commit + push Phase 6 (when the owner asks).
3. Assignment 2, Phase 7: `sq!` macro, `perft_tests!` (replace hand-written perft tests), `pst!`.
   Then Phase 8 (AI + worker thread) and Phase 9 (Slint GUI).

## Decisions worth remembering
- `Position::make_move` is **checked** (returns `Result<(), MoveError>`); the unchecked `apply` is `pub(crate)`.
- FEN parsing is strict (one king each, no back-rank pawns, castling/ep must be consistent,
  side not to move can't be in check).
- Threefold repetition and the 50-move rule end the game automatically (no "claim" step).
- `compile_fail,E0xxx` codes are **not** checked by stable rustdoc; they were verified manually.
- TUI undo stack stores whole `AnyGame` snapshots (not FEN strings) so history and repetition
  counts survive undo. Load key is `o` because `l` is cursor-right (hjkl).
- Git: commits authored only by the owner; no AI co-author lines (see AGENTS.md → Git & GitHub).

## Open questions for the owner
- Commit email `ridwan0gts@gmail.com` must be added to the GitHub account for commits to count on
  the profile (or switch the repo to the `…@users.noreply.github.com` address).
- Strict FEN validation vs. silently fixing sloppy castling/ep fields: currently strict.
- `Cargo.toml` declares `license = "MIT"` (added in Phase 1) but there is no LICENSE file: confirm the
  licence or remove the field.
- README links point at line numbers; re-check them after big edits (see PLAN.md Progress Log, phase 6 note).

## How to resume
```bash
git pull
cargo test --workspace            # should be all green
cargo run -p chess-tui            # play in the terminal
```
Then tell the agent: "read PROGRESS.md and continue".
