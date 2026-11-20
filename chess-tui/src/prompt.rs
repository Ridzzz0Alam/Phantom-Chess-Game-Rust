//! Text prompts: the `:` move/command line, and save / load names.

use crate::app::{App, Mode};
use crate::error::AppError;
use crate::save;
use chess_core::{AnyGame, Move, MoveError};

/// Longest text accepted in an input prompt.
const MAX_INPUT: usize = 120;

impl App {
    /// Opens the `:` move/command prompt.
    pub fn begin_command(&mut self) {
        self.begin_prompt(Mode::Command);
        self.info("Type a move like e2e4 or e7e8q, or: fen <FEN>");
    }

    /// Opens the save-name prompt.
    pub fn begin_save(&mut self) {
        self.begin_prompt(Mode::SaveName);
        self.info("Save as (letters, digits, - and _; empty = quicksave)");
    }

    /// Opens the load prompt, listing existing saves.
    pub fn begin_load(&mut self) {
        self.begin_prompt(Mode::LoadName);
        match save::list(&self.save_dir) {
            Ok(names) if names.is_empty() => self.info("No saves yet"),
            Ok(names) => self.info(format!("Saves: {}", names.join(", "))),
            Err(e) => self.error(e),
        }
    }

    fn begin_prompt(&mut self, mode: Mode) {
        self.mode = mode;
        self.input.clear();
        self.selected = None;
    }

    /// Adds a typed character to the prompt.
    pub fn type_char(&mut self, c: char) {
        if self.input.len() < MAX_INPUT {
            self.input.push(c);
        }
    }

    /// Enter in a prompt.
    pub fn submit_input(&mut self) {
        let text = std::mem::take(&mut self.input);
        let text = text.trim();
        let mode = std::mem::replace(&mut self.mode, Mode::Normal);
        let result = match mode {
            Mode::Command => self.run_command(text),
            Mode::SaveName => self.save(if text.is_empty() { "quicksave" } else { text }),
            Mode::LoadName => self.load(if text.is_empty() { "quicksave" } else { text }),
            _ => Ok(()),
        };
        if let Err(e) = result {
            self.error(e);
        }
    }

    fn run_command(&mut self, text: &str) -> Result<(), AppError> {
        if let Some(fen) = text.strip_prefix("fen ") {
            let game = AnyGame::from_fen(fen.trim())?;
            self.replace_game(game, "Position loaded from FEN");
            return Ok(());
        }
        let mv: Move = text.try_into()?;
        self.try_play(mv);
        Ok(())
    }

    fn save(&mut self, name: &str) -> Result<(), AppError> {
        let game = self
            .game
            .as_ref()
            .ok_or(AppError::Move(MoveError::GameOver))?;
        let path = save::save_to(&self.save_dir, name, game)?;
        self.info(format!("Saved to {}", path.display()));
        Ok(())
    }

    fn load(&mut self, name: &str) -> Result<(), AppError> {
        let game = save::load_from(&self.save_dir, name)?;
        self.replace_game(game, format!("Loaded '{name}'"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::MessageKind;

    fn type_command(app: &mut App, text: &str) {
        app.begin_command();
        text.chars().for_each(|c| app.type_char(c));
        app.submit_input();
    }

    #[test]
    fn typed_moves_and_parse_errors() {
        let mut app = App::new();
        type_command(&mut app, "z9e4");
        assert!(app.message.as_ref().unwrap().text.contains("invalid move"));
        type_command(&mut app, "e2e4");
        assert_eq!(app.game().unwrap().history().len(), 1);
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn fen_command_reports_fen_errors() {
        let mut app = App::new();
        type_command(&mut app, "fen 8/8 w - - 0 1");
        let msg = app.message.clone().unwrap();
        assert_eq!(msg.kind, MessageKind::Error);
        assert!(msg.text.contains("expected 8 ranks"), "{}", msg.text);
    }
}
