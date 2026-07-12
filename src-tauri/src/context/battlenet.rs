//! Battle.net: statt das Protobuf-Format der product.db nachzubauen, lesen wir
//! die Uninstall-Registry — Battle.net-Spiele tragen dort "Battle.net" im
//! UninstallString und haben ein "InstallLocation".

use std::path::PathBuf;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

pub fn game_dirs() -> Vec<PathBuf> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(uninstall) =
        hklm.open_subkey("SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
    else {
        return Vec::new();
    };

    uninstall
        .enum_keys()
        .filter_map(Result::ok)
        .filter_map(|id| uninstall.open_subkey(&id).ok())
        .filter(|key| {
            key.get_value::<String, _>("UninstallString")
                .is_ok_and(|s| s.contains("Battle.net"))
        })
        .filter_map(|key| key.get_value::<String, _>("InstallLocation").ok())
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .collect()
}
