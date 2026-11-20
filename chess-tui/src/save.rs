//! Save and load games as FEN files in a `saves/` directory.

use crate::error::AppError;
use chess_core::AnyGame;
use std::fs;
use std::path::{Path, PathBuf};

/// Default directory for save files, relative to the working directory.
pub const SAVE_DIR: &str = "saves";

/// Rejects empty names and anything that could escape the save directory.
fn check_name(name: &str) -> Result<&str, AppError> {
    let ok = !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(name)
    } else {
        Err(AppError::BadFileName(name.to_string()))
    }
}

fn path_for(dir: &Path, name: &str) -> Result<PathBuf, AppError> {
    Ok(dir.join(format!("{}.fen", check_name(name)?)))
}

/// Writes the game's current position to `<dir>/<name>.fen`, creating `dir` if needed.
pub fn save_to(dir: &Path, name: &str, game: &AnyGame) -> Result<PathBuf, AppError> {
    let path = path_for(dir, name)?;
    fs::create_dir_all(dir)?;
    fs::write(&path, format!("{}\n", game.to_fen()))?;
    Ok(path)
}

/// Reads `<dir>/<name>.fen` and starts a game from it.
pub fn load_from(dir: &Path, name: &str) -> Result<AnyGame, AppError> {
    let text = fs::read_to_string(path_for(dir, name)?)?;
    Ok(AnyGame::from_fen(text.trim())?)
}

/// Names (without `.fen`) of the saves in `dir`, sorted. Empty if `dir` doesn't exist.
pub fn list(dir: &Path) -> Result<Vec<String>, AppError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|e| e == "fen"))
        .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("phantom-chess-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn save_then_load_restores_position() {
        let dir = temp_dir("roundtrip");
        let game = AnyGame::new()
            .play("e2e4".try_into().unwrap())
            .unwrap_or_else(|(_, e)| panic!("{e}"));
        save_to(&dir, "game1", &game).unwrap();
        let loaded = load_from(&dir, "game1").unwrap();
        assert_eq!(loaded.to_fen(), game.to_fen());
        assert_eq!(list(&dir).unwrap(), vec!["game1".to_string()]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn bad_names_are_rejected() {
        let dir = temp_dir("names");
        for name in ["", "../evil", "a/b", "with space", "dot.fen"] {
            assert!(
                matches!(
                    save_to(&dir, name, &AnyGame::new()),
                    Err(AppError::BadFileName(_))
                ),
                "{name:?}"
            );
        }
        assert!(!dir.exists());
    }

    #[test]
    fn loading_missing_file_is_io_error() {
        let dir = temp_dir("missing");
        assert!(matches!(load_from(&dir, "nope"), Err(AppError::Io(_))));
    }

    #[test]
    fn loading_corrupt_file_is_fen_error() {
        let dir = temp_dir("corrupt");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("bad.fen"), "not a fen").unwrap();
        assert!(matches!(load_from(&dir, "bad"), Err(AppError::Fen(_))));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_of_missing_dir_is_empty() {
        assert!(list(&temp_dir("nodir")).unwrap().is_empty());
    }
}
