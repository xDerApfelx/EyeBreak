//! Steam: steamlocate übernimmt Registry-Lookup, libraryfolders.vdf und
//! appmanifest_*.acf-Parsing (dieselbe Technik wie DLSS Swapper).

use std::path::PathBuf;

pub fn game_dirs() -> Vec<PathBuf> {
    let Ok(steam) = steamlocate::SteamDir::locate() else {
        return Vec::new();
    };
    let Ok(libraries) = steam.libraries() else {
        return Vec::new();
    };

    let mut dirs = Vec::new();
    for library in libraries.filter_map(Result::ok) {
        for app in library.apps().filter_map(Result::ok) {
            dirs.push(library.resolve_app_dir(&app));
        }
    }
    dirs
}
