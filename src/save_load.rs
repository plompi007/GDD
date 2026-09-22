//! M9/Sandbox save & load (docs/GDD.md's Sandbox spec): writes/reads a
//! `LevelFile` through the exact same JSON schema every official level
//! already parses (`level_file_format.rs`'s `Serialize`/`Deserialize`),
//! to a runtime `saves/` directory — unlike `level_catalog.rs`'s
//! `ALL_LEVELS`, a player's save can't be baked in at compile time via
//! `include_str!`, since it doesn't exist until they make it.
//!
//! **Android app-private storage, via bevy's own public API.**
//! [`default_saves_dir`] used to always resolve to `saves/` relative to
//! the current working directory — fine for a desktop build run from its
//! own directory, but on Android that's not a real writable, app-private
//! location (nothing guarantees the process's CWD is even writable
//! there). The real path needs the JNI Activity context
//! (`Context.getFilesDir()`), which isn't something a plain-Rust path
//! API can resolve on its own — but `bevy_winit`'s Android backend
//! already holds exactly that, via the `android-activity` crate it
//! depends on for the platform event loop, and re-exports it as
//! `bevy::window::ANDROID_APP: OnceLock<AndroidApp>` for app code to use
//! for precisely this kind of thing. `AndroidApp::internal_data_path()`
//! is the real `Context.getFilesDir()` equivalent, not a guess or an
//! env-var convention that happens to often work — so the `#[cfg(target_os
//! = "android")]` branch below is a real fix, not a placeholder. It can't
//! be exercised by `cargo test` on this (or any non-Android) dev
//! machine, though — like `render.rs`/`render_fx.rs`'s real-renderer-only
//! systems, it can only be verified by an actual run on-device (M3.5,
//! still open for other reasons — see docs/GDD.md).

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::level_file_format::LevelFile;

/// Real Android app-private storage — see this module's own doc comment.
/// Falls back to the same relative `saves/` the desktop branch uses if
/// `ANDROID_APP` hasn't been populated yet (shouldn't happen once the
/// app's event loop is running, but a missing directory to write into is
/// a much friendlier failure than a panic).
#[cfg(target_os = "android")]
pub fn default_saves_dir() -> PathBuf {
    bevy::window::ANDROID_APP
        .get()
        .and_then(|app| app.internal_data_path())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("saves")
}

#[cfg(not(target_os = "android"))]
pub fn default_saves_dir() -> PathBuf {
    PathBuf::from("saves")
}

/// Maximum length for a player-typed save name — generous enough for a
/// real title, short enough that it can't be used to build an absurdly
/// long path. `pub` so `ui/editor_tools.rs`'s live keystroke handler can
/// cap the buffer at the same length `sanitize_file_stem` enforces,
/// rather than letting it grow unbounded and only get truncated later.
pub const MAX_SAVE_NAME_LEN: usize = 40;

/// Filters a player-typed save name down to a safe filename component:
/// only ASCII letters/digits/space/`_`/`-` survive, trimmed, capped to
/// [`MAX_SAVE_NAME_LEN`]. Returns `None` if nothing safe is left (empty
/// input, or input that was entirely punctuation/whitespace/non-ASCII) —
/// callers fall back to a sensible default (the level's own id) rather
/// than writing to a file named `""`.
///
/// This exists because [`save_level`] joins its `file_stem` argument
/// directly onto a directory path — once save names come from a player
/// typing into a UI text field (`ui/editor_tools.rs`'s `SaveNameField`)
/// instead of always being the level's own `id`, an unfiltered name like
/// `"../../etc/passwd"` would be a real path-traversal write, not just
/// unpolished input handling.
pub fn sanitize_file_stem(raw: &str) -> Option<String> {
    let filtered: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '_' || *c == '-')
        .take(MAX_SAVE_NAME_LEN)
        .collect();
    let trimmed = filtered.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
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
    fn sanitize_file_stem_blocks_path_traversal_and_keeps_safe_names_intact() {
        assert_eq!(sanitize_file_stem("Bridge Run"), Some("Bridge Run".to_string()));
        assert_eq!(sanitize_file_stem("  padded  "), Some("padded".to_string()));
        assert_eq!(
            sanitize_file_stem("../../etc/passwd"),
            Some("etcpasswd".to_string()),
            "dots and slashes are stripped entirely, so it can never escape the saves dir"
        );
        assert_eq!(sanitize_file_stem(""), None);
        assert_eq!(sanitize_file_stem("   "), None);
        assert_eq!(sanitize_file_stem("!!!///$$$"), None);
        let long_input = "a".repeat(200);
        assert_eq!(sanitize_file_stem(&long_input).unwrap().len(), MAX_SAVE_NAME_LEN);
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
