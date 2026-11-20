# PROGRESS.md: where we left off

> **Read this first when resuming.** It is the short handoff note between work sessions.
> PLAN.md is the full roadmap (checkboxes + Progress Log); this file says *what state the code is in
> right now* and *what to do next*. Update the "Current state" and "Next steps" sections at the end
> of every session.

_Last updated: 2026-11-20 (end of session 1)_

## Current state

| Phase | Status |
|---|---|
| 1. Workspace & primitives | ✅ done |
| 2. Board, Position, FEN | ✅ done |
| 3. Move generation, perft | ✅ done (perft d1–d4 all match) |
| 4. Phantom-type game layer | ✅ done |
| 5. Terminal UI (Ratatui) | ✅ done |
| 6. A1 polish & hand-in | ✅ done (README, docs, review) |
| 7. macro_rules! (`sq!`, `perft_tests!`, `pst!`) | ✅ done |
| 8. AI opponent (Arc, while let) | ⬜ **next** |
| 9. Slint GUI | ⬜ |
| 10. A2 hand-in | ⬜ |

Everything above is committed and pushed to `origin/main`.
**Assignment 1 is complete**; Assignment 2 has started (macros done).
154 tests pass (+5 slow perft tests with `--release -- --ignored`); clippy, fmt and `cargo doc`
(warnings as errors) are clean.

### How to run
- `cargo run -p chess-tui` (or `cargo run -p chess-tui -- "<FEN>"`). Press `?` in the app for keys.
- Keys: arrows/hjkl cursor, Enter/Space select+move, `:` type move or `fen <FEN>`, `u` undo, `U` undo all,
  `f` flip, `g` glyphs/letters, `n` new, `s` save, `o` load, `r` resign, `d` draw, `?` help, `q` quit.

## Next steps
1. **Owner:** play-test the TUI in a real terminal (full game to mate; illegal-move message;
   save → quit → load). Optionally add a real screenshot to the README (currently a text capture).
2. **Phase 8: AI opponent** (PLAN.md §5, Phase 8):
   - `chess-core/src/ai.rs`: material + piece-square evaluation (tables written with `pst!`),
     negamax + alpha-beta, iterative deepening, MVV-LVA move ordering, `stop: &AtomicBool` checked
     periodically. Tests: finds mate-in-1, doesn't hang the queen, returns quickly after stop.
   - `chess-gui/src/ai_worker.rs`: worker thread, `Arc<AtomicBool>` stop flag + `Arc<Mutex<SearchInfo>>`,
     `while let Ok(req) = rx.recv()` loop, replies on a second channel. Never hold the Mutex during search.
   - Optional: `a` key in the TUI to play vs the computer.
3. Phase 9: Slint GUI. Phase 10: A2 README/tests/review.

## Session log
- **2026-11-20**: Phases 1–7 in one session. Engine, FEN, move generation (perft-verified),
  phantom-type `Game<S>` + `AnyGame`, Ratatui TUI with save/load/undo, README with techniques table,
  three `macro_rules!` macros. Repo pushed to GitHub; PROGRESS.md handoff file created.

## Decisions worth remembering
- `Position::make_move` is **checked** (returns `Result<(), MoveError>`); the unchecked `apply` is `pub(crate)`.
- FEN parsing is strict (one king each, no back-rank pawns, castling/ep must be consistent,
  side not to move can't be in check).
- Threefold repetition and the 50-move rule end the game automatically (no "claim" step).
- `compile_fail,E0xxx` codes are **not** checked by stable rustdoc; they were verified manually.
- TUI undo stack stores whole `AnyGame` snapshots (not FEN strings) so history and repetition
  counts survive undo. Load key is `o` because `l` is cursor-right (hjkl).
- `sq!(z9)` is a compile error (E0080, const-eval panic); `perft_tests!` lines accept attributes such as
  `#[ignore = "..."]`; `pst!` rows are written rank 8 first.
- Git: commits authored only by the owner; no AI co-author lines (see AGENTS.md → Git & GitHub).

## Open questions for the owner
- Commit email `ridwan0gts@gmail.com` must be added to the GitHub account for commits to count on
  the profile (or switch the repo to the `…@users.noreply.github.com` address).
- Strict FEN validation vs. silently fixing sloppy castling/ep fields: currently strict.
- `Cargo.toml` declares `license = "MIT"` (added in Phase 1) but there is no LICENSE file: confirm the
  licence or remove the field.
- README links point at line numbers; after moving code run `sh scripts/check-readme-links.sh` and fix
  any link whose printed line is wrong.

## How to resume
```bash
git pull
cargo test --workspace            # should be all green
cargo run -p chess-tui            # play in the terminal
```
Then tell the agent: "read PROGRESS.md and continue".
