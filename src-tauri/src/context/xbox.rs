//! Xbox App / Game Pass: Spiele landen standardmäßig in <Laufwerk>:\XboxGames.
//! Ein einfacher Ordner-Scan über alle Laufwerke deckt den Normalfall ab.

use std::fs;
use std::path::PathBuf;

pub fn game_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for letter in b'A'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\XboxGames", letter as char));
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            if entry.path().is_dir() {
                dirs.push(entry.path());
            }
        }
    }
    dirs
}
