# CLAUDE.md — phantom-chess

@AGENTS.md

## Claude-specific notes
The shared rules, commands and conventions are imported above from AGENTS.md; they all apply.
This section only adds what's specific to Claude.

### Your role
You own the **[Claude]** tasks in PLAN.md: the phantom-type API (`game.rs`), move generation and legality
(`movegen.rs`, `position.rs`, `fen.rs`), perft debugging, the TUI app state machine, the AI worker
thread (Arc / channels), and the Slint bridge. [Light] tasks are normally done by cheaper models
(GLM / DeepSeek). Only do them if the user asks you to, or if they block a [Claude] task.

### Token discipline
- Read only the PLAN.md phase you're working on (use line ranges / grep), not the whole file.
- Don't re-read files you just edited. Don't print whole files in replies.
- Prefer `cargo test -p chess-core <filter>` over the whole suite while iterating.
- When a perft count is wrong, use `divide(depth)` on the failing position and compare per-move counts
  against known-good values to find the buggy move type. Don't guess-and-rerun.

### Design guard rails
- `Game<S>::play(self, mv) -> Result<Played<S::Next>, (Self, MoveError)>`: it consumes `self` and returns the game on error. Keep this signature; it's the centrepiece of the assignment.
- `Position` is the untyped engine; `Game<S>` is a thin typestate wrapper over it. Don't duplicate rules logic in `game.rs`.
- `AnyGame` is the only runtime boundary. The UI stores `Option<AnyGame>` and uses `take()` to move it into `play`.
- AI threading (A2): never hold the `Mutex` during search; the GUI thread must never block on the worker.
- Keep a `compile_fail` doc test proving a double move doesn't compile.

### Handing off to light models
When you finish a [Claude] task that unblocks [Light] tasks, make sure those tasks in PLAN.md are
specific enough for a weaker model: name the file, the function signatures, and the expected test
cases. Add a short note in the Progress Log if anything changed from the plan.
