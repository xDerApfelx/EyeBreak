//! Update-Check gegen GitHub Releases.
//!
//! v1 benachrichtigt nur (gelber Tray-Punkt + Release-Seite öffnen).
//! Volles Silent-Auto-Update via tauri-plugin-updater folgt, sobald das
//! GitHub-Repo existiert und Signatur-Schlüssel eingerichtet sind
//! (`npm run tauri signer generate`) — siehe README.

use crate::tray::{self, TrayStatus};
use std::sync::RwLock;
use std::time::Duration;
use tauri::AppHandle;

/// "owner/repo" auf GitHub. None = Update-Check deaktiviert.
pub const UPDATE_REPO: Option<&str> = Some("xDerApfelx/EyeBreak");

/// URL der neuesten Release-Seite, sobald ein Update gefunden wurde.
pub static LATEST_RELEASE_URL: RwLock<Option<String>> = RwLock::new(None);

pub fn spawn_update_checker(app: AppHandle) {
    let Some(repo) = UPDATE_REPO else {
        return;
    };
    std::thread::spawn(move || loop {
        match check_for_update(repo) {
            Ok(Some(url)) => {
                *LATEST_RELEASE_URL.write().unwrap() = Some(url);
                tray::set_status(&app, TrayStatus::UpdateAvailable);
            }
            Ok(None) => {}
            Err(e) => eprintln!("Update-Check fehlgeschlagen: {e}"),
        }
        std::thread::sleep(Duration::from_secs(6 * 60 * 60));
    });
}

/// Gibt die Release-URL zurück, wenn eine neuere Version existiert.
fn check_for_update(repo: &str) -> Result<Option<String>, String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let response: serde_json::Value = ureq::get(&url)
        .set("User-Agent", "EyeBreak")
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;

    let tag = response
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or("kein tag_name in der Antwort")?;
    let latest = parse_version(tag);
    let current = parse_version(env!("CARGO_PKG_VERSION"));

    if latest > current {
        let html_url = response
            .get("html_url")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("https://github.com/{repo}/releases/latest"))
            .to_string();
        Ok(Some(html_url))
    } else {
        Ok(None)
    }
}

/// "v1.2.3" → (1, 2, 3); fehlende Teile werden 0
fn parse_version(raw: &str) -> (u64, u64, u64) {
    let mut parts = raw
        .trim_start_matches('v')
        .split('.')
        .map(|p| p.parse::<u64>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}
