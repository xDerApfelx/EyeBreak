//! GOG Galaxy: installierte Spiele registrieren sich unter
//! HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\<id> mit "path".

use std::path::PathBuf;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

pub fn game_dirs() -> Vec<PathBuf> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(games) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\GOG.com\\Games") else {
        return Vec::new();
    };

    games
        .enum_keys()
        .filter_map(Result::ok)
        .filter_map(|id| games.open_subkey(&id).ok())
        .filter_map(|key| key.get_value::<String, _>("path").ok())
        .map(PathBuf::from)
        .collect()
}
