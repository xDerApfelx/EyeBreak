//! Epic Games Store: JSON-Manifeste (.item) unter ProgramData enthalten
//! pro installiertem Spiel den "InstallLocation"-Pfad.

use std::fs;
use std::path::PathBuf;

pub fn game_dirs() -> Vec<PathBuf> {
    let manifest_dir = PathBuf::from(
        std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".into()),
    )
    .join("Epic\\EpicGamesLauncher\\Data\\Manifests");

    let Ok(entries) = fs::read_dir(manifest_dir) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("item"))
        })
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .filter_map(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .filter_map(|manifest| {
            manifest
                .get("InstallLocation")
                .and_then(|v| v.as_str())
                .map(PathBuf::from)
        })
        .collect()
}
