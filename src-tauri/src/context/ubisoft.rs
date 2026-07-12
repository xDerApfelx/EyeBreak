//! Ubisoft Connect: installierte Spiele stehen unter
//! HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\<id> mit "InstallDir".

use std::path::PathBuf;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

pub fn game_dirs() -> Vec<PathBuf> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(installs) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher\\Installs")
    else {
        return Vec::new();
    };

    installs
        .enum_keys()
        .filter_map(Result::ok)
        .filter_map(|id| installs.open_subkey(&id).ok())
        .filter_map(|key| key.get_value::<String, _>("InstallDir").ok())
        .map(PathBuf::from)
        .collect()
}
