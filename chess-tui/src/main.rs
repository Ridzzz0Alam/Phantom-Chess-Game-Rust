//! Terminal chess front-end.
//!
//! Usage: `chess-tui [FEN]`: starts from the standard position, or from `FEN` if given.

mod app;
mod error;
mod input;
mod prompt;
mod save;
mod ui;

use app::App;
use chess_core::AnyGame;
use input::Keymap;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Validate the optional FEN before touching the terminal, so errors print normally.
    let game = match std::env::args().nth(1) {
        None => AnyGame::new(),
        Some(fen) => match AnyGame::from_fen(&fen) {
            Ok(game) => game,
            Err(e) => {
                eprintln!("chess-tui: {e}");
                return ExitCode::from(2);
            }
        },
    };

    // `init` enters raw mode / alternate screen and installs a panic hook that
    // restores the terminal; `restore` undoes it on normal exit.
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, App::with_game(game));
    ratatui::restore();

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("chess-tui: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(terminal: &mut DefaultTerminal, mut app: App) -> io::Result<()> {
    let keymap = Keymap::new();
    while app.running {
        terminal.draw(|frame| ui::draw(frame, &app, &keymap))?;
        // Windows reports both press and release; act on presses only.
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            keymap.handle(&mut app, key);
        }
    }
    Ok(())
}
