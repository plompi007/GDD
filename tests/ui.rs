//! docs/GDD.md §3.9.7 / `ui::tokens`: the bundled Rubik font files are
//! subsetted to the Hebrew alphabet + Latin + basic punctuation (not the
//! full Hebrew Unicode block — no niqqud/geresh beyond the two already
//! requested) because a full-block subset would be needlessly large for a
//! game whose only Hebrew text is level/part JSON content. This test
//! fails loudly the moment new content uses a Hebrew character outside
//! that subset, instead of silently rendering as tofu on a real device.

use std::collections::HashSet;
use std::path::Path;

/// Exactly the character set requested when the fonts under
/// `assets/fonts/` were subsetted (see that directory's `OFL.txt` for
/// license terms) — keep this in sync if the subset is ever regenerated.
fn font_subset() -> HashSet<char> {
    "אבגדהוזחטיכךלמםנןסעפףצץקרשת0123456789 .,:!?()-״׳"
        .chars()
        .collect()
}

fn is_hebrew(c: char) -> bool {
    ('\u{0590}'..='\u{05FF}').contains(&c)
}

fn collect_strings(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) => out.push(s.clone()),
        serde_json::Value::Array(items) => {
            for item in items {
                collect_strings(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                collect_strings(v, out);
            }
        }
        _ => {}
    }
}

fn check_dir(dir: &Path, subset: &HashSet<char>, missing: &mut Vec<(String, char)>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir({dir:?}): {e}")) {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            check_dir(&path, subset, missing);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        let json: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        let mut strings = Vec::new();
        collect_strings(&json, &mut strings);
        for s in strings {
            for c in s.chars() {
                if is_hebrew(c) && !subset.contains(&c) {
                    missing.push((path.display().to_string(), c));
                }
            }
        }
    }
}

#[test]
fn all_hebrew_json_content_is_covered_by_the_bundled_font_subset() {
    let subset = font_subset();
    let mut missing = Vec::new();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    check_dir(&Path::new(manifest_dir).join("levels"), &subset, &mut missing);
    check_dir(&Path::new(manifest_dir).join("data/parts"), &subset, &mut missing);
    assert!(
        missing.is_empty(),
        "Hebrew characters outside the bundled Rubik subset (would render as tofu on device): {missing:?}"
    );
}
