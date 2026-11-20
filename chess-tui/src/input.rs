//! Key bindings as data: a table of keys → boxed closures.
//!
//! The same table drives both key handling and the help screen, so they can't
//! drift apart. Cursor actions are produced by a higher-order function
//! ([`move_cursor`]) that returns a closure.

use crate::app::{App, Mode};
use chess_core::PieceKind;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Something a key does.
pub type Action = Box<dyn Fn(&mut App)>;

/// One row of the key table.
pub struct Binding {
    /// Keys that trigger it.
    pub keys: Vec<KeyCode>,
    /// How the keys are shown in the help screen.
    pub label: &'static str,
    /// What it does, for the help screen.
    pub description: &'static str,
    /// The action itself.
    pub action: Action,
}

/// Builds a binding from any closure or function.
fn bind(
    keys: impl IntoIterator<Item = KeyCode>,
    label: &'static str,
    description: &'static str,
    action: impl Fn(&mut App) + 'static,
) -> Binding {
    Binding {
        keys: keys.into_iter().collect(),
        label,
        description,
        action: Box::new(action),
    }
}

/// Higher-order function: returns a closure that moves the cursor by (`df`, `dr`).
fn move_cursor(df: i8, dr: i8) -> impl Fn(&mut App) {
    move |app| app.move_cursor(df, dr)
}

/// The key table for normal mode.
pub struct Keymap {
    bindings: Vec<Binding>,
}

impl Default for Keymap {
    fn default() -> Self {
        Keymap::new()
    }
}

impl Keymap {
    /// The default bindings.
    pub fn new() -> Keymap {
        use KeyCode::{Char, Down, Enter, Esc, Left, Right, Up};
        let bindings = vec![
            bind([Up, Char('k')], "↑ / k", "cursor up", move_cursor(0, 1)),
            bind(
                [Down, Char('j')],
                "↓ / j",
                "cursor down",
                move_cursor(0, -1),
            ),
            bind(
                [Left, Char('h')],
                "← / h",
                "cursor left",
                move_cursor(-1, 0),
            ),
            bind(
                [Right, Char('l')],
                "→ / l",
                "cursor right",
                move_cursor(1, 0),
            ),
            bind(
                [Enter, Char(' ')],
                "Enter / Space",
                "select piece / move it here",
                App::activate,
            ),
            bind([Esc], "Esc", "cancel selection", App::cancel),
            bind(
                [Char(':')],
                ":",
                "type a move (e2e4) or fen <FEN>",
                App::begin_command,
            ),
            bind([Char('u')], "u", "undo", App::undo),
            bind([Char('U')], "U", "undo everything", App::undo_all),
            bind([Char('f')], "f", "flip board", App::flip),
            bind([Char('g')], "g", "glyphs ↔ letters", App::toggle_ascii),
            bind([Char('n')], "n", "new game", App::new_game),
            bind([Char('s')], "s", "save game", App::begin_save),
            bind([Char('o')], "o", "open (load) a save", App::begin_load),
            bind([Char('r')], "r", "resign (side to move)", App::resign),
            bind([Char('d')], "d", "agree to a draw", App::draw),
            bind([Char('?')], "?", "this help", App::show_help),
            bind([Char('q')], "q", "quit", App::quit),
        ];
        Keymap { bindings }
    }

    /// All bindings, in help-screen order.
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// Dispatches one key press according to the current mode.
    pub fn handle(&self, app: &mut App, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            app.quit();
            return;
        }
        match app.mode {
            Mode::Normal => {
                if let Some(binding) = self.bindings.iter().find(|b| b.keys.contains(&key.code)) {
                    (binding.action)(app);
                }
            }
            Mode::Help => app.mode = Mode::Normal,
            Mode::Promotion { .. } => match key.code {
                KeyCode::Esc => app.cancel(),
                KeyCode::Char(c) => {
                    if let Ok(kind) = PieceKind::try_from(c)
                        && kind.is_promotion_target()
                    {
                        app.promote(kind);
                    }
                }
                _ => {}
            },
            Mode::Command | Mode::SaveName | Mode::LoadName => match key.code {
                KeyCode::Enter => app.submit_input(),
                KeyCode::Esc => app.cancel(),
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Char(c) => app.type_char(c),
                _ => {}
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::Square;

    fn press(keymap: &Keymap, app: &mut App, code: KeyCode) {
        keymap.handle(app, KeyEvent::new(code, KeyModifiers::NONE));
    }

    #[test]
    fn no_key_is_bound_twice() {
        let keymap = Keymap::new();
        let mut seen = Vec::new();
        for b in keymap.bindings() {
            for k in &b.keys {
                assert!(!seen.contains(k), "{k:?} bound twice");
                seen.push(*k);
            }
        }
    }

    #[test]
    fn keyboard_only_move() {
        let keymap = Keymap::new();
        let mut app = App::new(); // cursor starts on e2
        press(&keymap, &mut app, KeyCode::Enter);
        press(&keymap, &mut app, KeyCode::Char('k'));
        press(&keymap, &mut app, KeyCode::Up);
        assert_eq!(app.cursor, Square::try_from("e4").unwrap());
        press(&keymap, &mut app, KeyCode::Char(' '));
        assert_eq!(app.game().unwrap().history().len(), 1);
    }

    #[test]
    fn typing_in_command_mode_does_not_trigger_bindings() {
        let keymap = Keymap::new();
        let mut app = App::new();
        press(&keymap, &mut app, KeyCode::Char(':'));
        for c in "qfe2e4".chars() {
            press(&keymap, &mut app, KeyCode::Char(c));
        }
        assert!(app.running, "'q' must be typed, not quit");
        assert!(!app.flipped);
        assert_eq!(app.input, "qfe2e4");
        press(&keymap, &mut app, KeyCode::Backspace);
        assert_eq!(app.input, "qfe2e");
        press(&keymap, &mut app, KeyCode::Esc);
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn ctrl_c_always_quits() {
        let keymap = Keymap::new();
        let mut app = App::new();
        app.begin_command();
        keymap.handle(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(!app.running);
    }
}
