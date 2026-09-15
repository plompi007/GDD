//! M9/Sandbox save & load (docs/GDD.md's Sandbox spec): writes/reads a
//! `LevelFile` through the exact same JSON schema every official level
//! already parses (`level_file_format.rs`'s `Serialize`/`Deserialize`),
//! to a runtime `saves/` directory — unlike `level_catalog.rs`'s
//! `ALL_LEVELS`, a player's save can't be baked in at compile time via
//! `include_str!`, since it doesn't exist until they make it.
//!
//! **Desktop-only for now.** [`default_saves_dir`] resolves to a `saves/`
//! folder relative to the current working directory, which is fine for a
//! desktop build run from its own directory but is *not* yet a real
//! Android app-private storage path — that needs the JNI Activity
//! context (`Context.getFilesDir()`), not something any plain-Rust path
//! API can resolve. Flagged as a known gap in docs/GDD.md's Sandbox
//! spec, not solved here; every function below still takes an explicit
//! directory/path rather than assuming this default, so the Android
//! resolution can plug in later without changing this module at all.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::level_file_format::LevelFile;

/// Desktop-only — see this module's own doc comment.
pub fn default_saves_dir() -> PathBuf {
    PathBuf::from("saves")
}

#[derive(Debug)]
pub enum SaveLoadError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for SaveLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveLoadError::Io(e) => write!(f, "save/load I/O error: {e}"),
            SaveLoadError::Json(e) => write!(f, "save/load JSON error: {e}"),
        }
    }
}

impl std::error::Error for SaveLoadError {}

impl From<std::io::Error> for SaveLoadError {
    fn from(e: std::io::Error) -> Self {
        SaveLoadError::Io(e)
    }
}

impl From<serde_json::Error> for SaveLoadError {
    fn from(e: serde_json::Error) -> Self {
        SaveLoadError::Json(e)
    }
}

/// Writes `level` as pretty-printed JSON to `<dir>/<file_stem>.json`,
/// creating `dir` if it doesn't exist yet. Returns the path actually
/// written to, since a caller building a "My Levels" list wants it
/// without re-deriving the naming rule itself.
pub fn save_level(level: &LevelFile, dir: &Path, file_stem: &str) -> Result<PathBuf, SaveLoadError> {
    fs::create_dir_all(dir)?;
    let path = dir.join(file_stem).with_extension("json");
    let json = serde_json::to_string_pretty(level)?;
    fs::write(&path, json)?;
    Ok(path)
}

/// Reads a `LevelFile` back from an exact path — a saved level (from
/// [`save_level`]) or one imported from elsewhere (docs/GDD.md's
/// Sandbox "share" = plain JSON file import). Goes through the *same*
/// `serde_json::from_str::<LevelFile>` every built-in level already
/// parses with, so a malformed file fails the same way a malformed
/// built-in level would — no separate error path for "someone else's
/// file" versus "our own save".
pub fn load_level(path: &Path) -> Result<LevelFile, SaveLoadError> {
    let json = fs::read_to_string(path)?;
    let level = serde_json::from_str(&json)?;
    Ok(level)
}

/// Every `.json` file directly inside `dir` (not recursive), for a "My
/// Levels" list — empty (not an error) if `dir` doesn't exist yet, since
/// "no saves yet" is the normal first-run state, not a failure.
pub fn list_saved_levels(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    paths.sort();
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level_catalog::ALL_LEVELS;

    /// A fresh, collision-free scratch directory per test, cleaned up
    /// afterward — this module works with real filesystem I/O (that's
    /// the point), so it needs a real temp directory rather than the
    /// in-memory `App`/ECS state every other test in this project uses.
    struct ScratchDir(PathBuf);

    impl ScratchDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "chainworks_save_load_test_{name}_{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&dir);
            ScratchDir(dir)
        }
    }

    impl Drop for ScratchDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn saving_then_loading_round_trips_a_real_level() {
        let scratch = ScratchDir::new("round_trip");
        let original: LevelFile =
            serde_json::from_str(ALL_LEVELS[0]).expect("ALL_LEVELS[0] must parse");

        let path = save_level(&original, &scratch.0, "my_save").expect("save must succeed");
        assert_eq!(path.file_name().unwrap(), "my_save.json");

        let reloaded = load_level(&path).expect("load must succeed");
        assert_eq!(reloaded.id, original.id);
        assert_eq!(reloaded.preplaced_parts.len(), original.preplaced_parts.len());
        assert_eq!(reloaded.win_conditions.len(), original.win_conditions.len());
    }

    #[test]
    fn listing_an_empty_or_missing_directory_is_not_an_error() {
        let scratch = ScratchDir::new("empty_listing");
        // Directory doesn't exist at all yet.
        assert!(list_saved_levels(&scratch.0).is_empty());

        // Directory exists but has nothing saved in it.
        fs::create_dir_all(&scratch.0).unwrap();
        assert!(list_saved_levels(&scratch.0).is_empty());
    }

    #[test]
    fn listing_finds_saved_levels_and_ignores_non_json_files() {
        let scratch = ScratchDir::new("listing");
        let level: LevelFile =
            serde_json::from_str(ALL_LEVELS[0]).expect("ALL_LEVELS[0] must parse");
        save_level(&level, &scratch.0, "a_save").unwrap();
        save_level(&level, &scratch.0, "b_save").unwrap();
        fs::write(scratch.0.join("notes.txt"), "not a level").unwrap();

        let found = list_saved_levels(&scratch.0);
        assert_eq!(found.len(), 2, "should find exactly the two .json saves, not notes.txt");
        assert!(found.iter().all(|p| p.extension().unwrap() == "json"));
    }

    #[test]
    fn loading_a_malformed_file_fails_the_same_way_a_malformed_built_in_level_would() {
        let scratch = ScratchDir::new("malformed");
        fs::create_dir_all(&scratch.0).unwrap();
        let bad_path = scratch.0.join("bad.json");
        fs::write(&bad_path, "{ this is not valid level JSON").unwrap();

        let result = load_level(&bad_path);
        assert!(matches!(result, Err(SaveLoadError::Json(_))));
    }
}
