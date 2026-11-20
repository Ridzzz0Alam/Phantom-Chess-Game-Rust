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
| 4. Phantom-type game layer | ✅ done, pushed (commit `09f5944`) |
| 5. Terminal UI (Ratatui) | ✅ code done, all checks green, **not committed yet**; owner to play-test |
| 6. A1 polish & hand-in | ⬜ not started |
| 7–10. Assignment 2 (macros, AI, Slint GUI) | ⬜ not started |

### Phase 5 summary
- Run: `cargo run -p chess-tui` (or `cargo run -p chess-tui -- "<FEN>"`). Press `?` in the app for keys.
- Files: `chess-tui/src/{main,app,prompt,input,ui,save,error}.rs`. Saves go to `saves/<name>.fen`.
- Keys: arrows/hjkl cursor, Enter/Space select+move, `:` type move or `fen <FEN>`, `u` undo, `U` undo all,
  `f` flip, `g` glyphs/letters, `n` new, `s` save, `o` load, `r` resign, `d` draw, `?` help, `q` quit.
- Manual acceptance still to confirm in a real terminal: play to checkmate; illegal move shows a message;
  save → quit → load restores the position.

## Next steps
1. Owner play-tests the TUI; fix anything that looks off (colours/glyph width vary by terminal; `g` switches
   to letters).
2. Commit + push Phase 5 (when the owner asks).
3. Phase 6: README (techniques table with file links, controls, screenshots, phantom-type explanation),
   rustdoc pass (`cargo doc --no-deps` clean), final review. Replace the unrelated TensorFlow notes
   currently in `README.md`.

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

## How to resume
```bash
git pull
cargo test --workspace            # should be all green
cargo run -p chess-tui            # play in the terminal
```
Then tell the agent: "read PROGRESS.md and continue".
