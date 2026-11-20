//! Application state and every action the user can trigger.
//!
//! The game lives in `Option<AnyGame>`: each state change `take()`s it out, moves
//! it into the engine (which consumes it, see `Game::play`) and puts the result back.

use crate::error::AppError;
use crate::save;
use chess_core::{AnyGame, Color, Move, MoveError, PieceKind, Square};
use std::fmt::Display;
use std::path::PathBuf;

/// What keyboard input currently means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Moving the cursor and selecting pieces.
    Normal,
    /// Typing a move (`e2e4`) or a command (`fen …`) after `:`.
    Command,
    /// Typing a save name.
    SaveName,
    /// Typing the name of a save to load.
    LoadName,
    /// Choosing the piece a pawn promotes to.
    Promotion {
        /// Pawn's square.
        from: Square,
        /// Promotion square.
        to: Square,
    },
    /// Help overlay; any key closes it.
    Help,
}

/// Colour of a status-bar message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    /// Neutral or good news.
    Info,
    /// Something was rejected.
    Error,
}

/// A status-bar message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// Text to show.
    pub text: String,
    /// How to colour it.
    pub kind: MessageKind,
}

/// e2: White's king pawn, a natural place to start.
const START_CURSOR: Square = match Square::new(4, 1) {
    Some(sq) => sq,
    None => panic!("e2 is on the board"),
};

/// The whole TUI state.
pub struct App {
    /// The game. Always `Some` except for the instant it is moved into the engine.
    pub game: Option<AnyGame>,
    /// Square under the cursor.
    pub cursor: Square,
    /// Square of the selected piece, if any.
    pub selected: Option<Square>,
    /// Text typed in the current prompt.
    pub input: String,
    /// Current input mode.
    pub mode: Mode,
    /// Status-bar message.
    pub message: Option<Message>,
    /// Whether Black is at the bottom.
    pub flipped: bool,
    /// Show pieces as letters instead of Unicode glyphs.
    pub ascii: bool,
    /// Earlier games states, newest last.
    pub undo: Vec<AnyGame>,
    /// Cleared to quit the main loop.
    pub running: bool,
    /// Where save files go.
    pub save_dir: PathBuf,
}

impl App {
    /// An app with a new game (tests; `main` uses `with_game`).
    #[cfg(test)]
    pub fn new() -> App {
        App::with_game(AnyGame::new())
    }

    /// An app starting from `game`.
    pub fn with_game(game: AnyGame) -> App {
        App {
            game: Some(game),
            cursor: START_CURSOR,
            selected: None,
            input: String::new(),
            mode: Mode::Normal,
            message: Some(Message {
                text: "Welcome! Press ? for help.".into(),
                kind: MessageKind::Info,
            }),
            flipped: false,
            ascii: false,
            undo: Vec::new(),
            running: true,
            save_dir: PathBuf::from(save::SAVE_DIR),
        }
    }

    /// The current game.
    pub fn game(&self) -> Option<&AnyGame> {
        self.game.as_ref()
    }

    /// Shows an informational message.
    pub fn info(&mut self, text: impl Into<String>) {
        self.message = Some(Message {
            text: text.into(),
            kind: MessageKind::Info,
        });
    }

    /// Shows an error message.
    pub fn error(&mut self, err: impl Display) {
        self.message = Some(Message {
            text: err.to_string(),
            kind: MessageKind::Error,
        });
    }

    /// Moves the game out, applies `f`, and puts the game `f` returns back in.
    /// `f` always hands a game back: the new one with `Ok`, or the untouched one
    /// with `Err`. On success the old game goes on the undo stack.
    /// Returns whether `f` succeeded.
    fn update(&mut self, f: impl FnOnce(AnyGame) -> (AnyGame, Result<(), AppError>)) -> bool {
        let Some(game) = self.game.take() else {
            return false;
        };
        let snapshot = game.clone();
        let (game, outcome) = f(game);
        self.game = Some(game);
        match outcome {
            Ok(()) => {
                self.undo.push(snapshot);
                self.selected = None;
                true
            }
            Err(e) => {
                self.error(e);
                false
            }
        }
    }

    /// Plays `mv`, reporting the outcome in the status bar.
    pub fn try_play(&mut self, mv: Move) {
        let mover = self.game().and_then(AnyGame::side_to_move);
        let played = self.update(|g| match g.play(mv) {
            Ok(next) => (next, Ok(())),
            Err((g, e)) => (g, Err(e.into())),
        });
        if played {
            self.announce(mover, mv);
        }
    }

    fn announce(&mut self, mover: Option<Color>, mv: Move) {
        let Some(game) = self.game() else { return };
        let text = if let Some(result) = game.result() {
            result.to_string()
        } else if game.position().in_check() {
            format!(
                "{} played {mv}. Check!",
                mover.map_or("".into(), |c| c.to_string())
            )
        } else {
            format!("{} played {mv}", mover.map_or("".into(), |c| c.to_string()))
        };
        self.info(text);
    }

    /// Replaces the game (new game / load), keeping the old one undoable.
    pub(crate) fn replace_game(&mut self, game: AnyGame, note: impl Into<String>) {
        if self.update(|_| (game, Ok(()))) {
            self.info(note);
        }
    }

    // ---------- cursor & selection ----------

    /// Moves the cursor by screen direction (`dr = 1` is up), respecting board flip.
    pub fn move_cursor(&mut self, df: i8, dr: i8) {
        let (df, dr) = if self.flipped { (-df, -dr) } else { (df, dr) };
        if let Some(sq) = self.cursor.offset(df, dr) {
            self.cursor = sq;
        }
    }

    /// Enter / Space: select the piece under the cursor, or move the selected piece there.
    pub fn activate(&mut self) {
        let cursor = self.cursor;
        match self.selected {
            None => self.select(cursor),
            Some(from) if from == cursor => self.selected = None,
            Some(from) => {
                let Some(game) = self.game() else { return };
                let mover = game.side_to_move();
                let own_piece = game
                    .position()
                    .piece_at(cursor)
                    .is_some_and(|p| Some(p.color) == mover);
                if own_piece {
                    self.select(cursor);
                    return;
                }
                let needs_promotion = game
                    .legal_moves_from(from)
                    .iter()
                    .any(|m| m.to == cursor && m.promotion.is_some());
                if needs_promotion {
                    self.mode = Mode::Promotion { from, to: cursor };
                    self.info("Promote to: q, r, b or n (Esc cancels)");
                } else {
                    self.try_play(Move::new(from, cursor));
                }
            }
        }
    }

    fn select(&mut self, sq: Square) {
        let Some(game) = self.game() else { return };
        let Some(mover) = game.side_to_move() else {
            self.error(MoveError::GameOver);
            return;
        };
        let err = match game.position().piece_at(sq) {
            None => Some(MoveError::NoPieceOnSquare(sq).to_string()),
            Some(p) if p.color != mover => Some(MoveError::NotYourPiece(sq).to_string()),
            Some(p) if game.legal_moves_from(sq).is_empty() => {
                Some(format!("the {} on {sq} has no legal moves", p.kind))
            }
            Some(_) => None,
        };
        match err {
            Some(e) => self.error(e),
            None => {
                self.selected = Some(sq);
                self.message = None;
            }
        }
    }

    /// Squares the selected piece can move to.
    pub fn targets(&self) -> Vec<Square> {
        match (self.selected, self.game()) {
            (Some(from), Some(game)) => game.legal_moves_from(from).iter().map(|m| m.to).collect(),
            _ => Vec::new(),
        }
    }

    /// The most recent move, for highlighting.
    pub fn last_move(&self) -> Option<Move> {
        self.game().and_then(|g| g.history().last().copied())
    }

    /// Finishes a pending promotion with `kind`.
    pub fn promote(&mut self, kind: PieceKind) {
        if let Mode::Promotion { from, to } = self.mode {
            self.mode = Mode::Normal;
            self.try_play(Move::with_promotion(from, to, kind));
        }
    }

    /// Esc: leave any prompt and clear the selection.
    pub fn cancel(&mut self) {
        self.mode = Mode::Normal;
        self.input.clear();
        self.selected = None;
        self.message = None;
    }

    // ---------- game actions ----------

    /// Takes back the last action (move, new game, load, resign…).
    pub fn undo(&mut self) {
        match self.undo.pop() {
            Some(prev) => {
                self.game = Some(prev);
                self.selected = None;
                self.info("Undone");
            }
            None => self.error("nothing to undo"),
        }
    }

    /// Takes back everything, back to the first state of this session.
    pub fn undo_all(&mut self) {
        let mut first = None;
        while let Some(prev) = self.undo.pop() {
            first = Some(prev);
        }
        match first {
            Some(game) => {
                self.game = Some(game);
                self.selected = None;
                self.info("Undid everything");
            }
            None => self.error("nothing to undo"),
        }
    }

    /// Starts a new game (undoable).
    pub fn new_game(&mut self) {
        self.replace_game(AnyGame::new(), "New game");
        self.cursor = START_CURSOR;
    }

    /// The side to move resigns.
    pub fn resign(&mut self) {
        if self.update(|g| {
            if g.is_over() {
                (g, Err(MoveError::GameOver.into()))
            } else {
                (g.resign(), Ok(()))
            }
        }) {
            self.announce_result();
        }
    }

    /// Both players agree to a draw.
    pub fn draw(&mut self) {
        if self.update(|g| {
            if g.is_over() {
                (g, Err(MoveError::GameOver.into()))
            } else {
                (g.agree_draw(), Ok(()))
            }
        }) {
            self.announce_result();
        }
    }

    fn announce_result(&mut self) {
        if let Some(result) = self.game().and_then(AnyGame::result) {
            self.info(result.to_string());
        }
    }

    /// Turns the board around.
    pub fn flip(&mut self) {
        self.flipped = !self.flipped;
    }

    /// Switches between Unicode glyphs and letters.
    pub fn toggle_ascii(&mut self) {
        self.ascii = !self.ascii;
    }

    /// Shows the help overlay.
    pub fn show_help(&mut self) {
        self.mode = Mode::Help;
    }

    /// Leaves the app.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::{GameResult, Piece, WinReason};

    fn sq(s: &str) -> Square {
        Square::try_from(s).unwrap()
    }

    fn click(app: &mut App, s: &str) {
        app.cursor = sq(s);
        app.activate();
    }

    #[test]
    fn select_and_move_with_cursor() {
        let mut app = App::new();
        click(&mut app, "e2");
        assert_eq!(app.selected, Some(sq("e2")));
        assert_eq!(app.targets().len(), 2);
        click(&mut app, "e4");
        assert_eq!(app.selected, None);
        assert_eq!(app.last_move(), Some(Move::new(sq("e2"), sq("e4"))));
        assert_eq!(
            app.game().and_then(AnyGame::side_to_move),
            Some(Color::Black)
        );
    }

    #[test]
    fn selecting_opponent_piece_or_empty_square_shows_error() {
        let mut app = App::new();
        click(&mut app, "e7");
        assert_eq!(app.selected, None);
        assert_eq!(
            app.message.as_ref().map(|m| m.kind),
            Some(MessageKind::Error)
        );
        click(&mut app, "e4");
        assert_eq!(app.selected, None);
    }

    #[test]
    fn illegal_destination_keeps_game_and_shows_reason() {
        let mut app = App::new();
        click(&mut app, "e2");
        click(&mut app, "e5");
        assert!(app.game().is_some());
        assert!(app.undo.is_empty());
        let msg = app.message.clone().unwrap();
        assert_eq!(msg.kind, MessageKind::Error);
        assert!(msg.text.contains("not a legal move"), "{}", msg.text);
    }

    #[test]
    fn promotion_asks_for_piece() {
        let mut app = App::with_game(AnyGame::from_fen("8/P6k/8/8/8/8/8/K7 w - - 0 1").unwrap());
        click(&mut app, "a7");
        click(&mut app, "a8");
        assert_eq!(
            app.mode,
            Mode::Promotion {
                from: sq("a7"),
                to: sq("a8")
            }
        );
        app.promote(PieceKind::Knight);
        assert_eq!(
            app.game().unwrap().position().piece_at(sq("a8")),
            Some(Piece::new(Color::White, PieceKind::Knight))
        );
    }

    #[test]
    fn fools_mate_then_undo() {
        let mut app = App::new();
        for m in ["f2f3", "e7e5", "g2g4", "d8h4"] {
            app.try_play(m.try_into().unwrap());
        }
        assert_eq!(
            app.game().and_then(AnyGame::result),
            Some(GameResult::BlackWins(WinReason::Checkmate))
        );
        assert!(app.message.as_ref().unwrap().text.contains("checkmate"));
        app.undo();
        assert!(!app.game().unwrap().is_over());
        app.undo_all();
        assert!(app.game().unwrap().history().is_empty());
        assert!(app.undo.is_empty());
    }

    #[test]
    fn resign_is_undoable() {
        let mut app = App::new();
        app.resign();
        assert_eq!(
            app.game().and_then(AnyGame::result),
            Some(GameResult::BlackWins(WinReason::Resignation))
        );
        app.resign();
        assert_eq!(app.message.as_ref().unwrap().kind, MessageKind::Error);
        app.undo();
        assert!(!app.game().unwrap().is_over());
    }

    #[test]
    fn cursor_respects_flip_and_edges() {
        let mut app = App::new();
        app.cursor = sq("a1");
        app.move_cursor(-1, 0);
        assert_eq!(app.cursor, sq("a1"));
        app.move_cursor(0, 1);
        assert_eq!(app.cursor, sq("a2"));
        app.flip();
        app.move_cursor(0, 1); // "up" on a flipped board is towards rank 1
        assert_eq!(app.cursor, sq("a1"));
    }
}
