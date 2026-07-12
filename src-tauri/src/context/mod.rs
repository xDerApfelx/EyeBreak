//! Kontext-Erkennung: Läuft im Vordergrund gerade ein Spiel oder normale Arbeit?
//!
//! Technik nach Vorbild von DLSS Swapper: pro Launcher werden die eigenen
//! Manifest-/Registry-Daten gelesen und daraus die Installationsordner aller
//! Spiele gesammelt. Der Vordergrund-Prozess wird gegen diese Liste geprüft;
//! als Fallback dient eine Vollbild-Heuristik.

#[cfg(windows)]
mod battlenet;
#[cfg(windows)]
mod epic;
#[cfg(windows)]
mod gog;
#[cfg(windows)]
mod steam;
#[cfg(windows)]
mod ubisoft;
#[cfg(windows)]
mod xbox;

use serde::Serialize;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UsageContext {
    Game,
    Work,
}

/// Kleingeschriebene Installationsordner aller bekannten Spiele.
static GAME_DIRS: RwLock<Vec<PathBuf>> = RwLock::new(Vec::new());

/// Alle Launcher scannen und den Index neu aufbauen.
#[cfg(windows)]
pub fn rescan() {
    let mut dirs: Vec<PathBuf> = Vec::new();
    dirs.extend(steam::game_dirs());
    dirs.extend(epic::game_dirs());
    dirs.extend(gog::game_dirs());
    dirs.extend(ubisoft::game_dirs());
    dirs.extend(xbox::game_dirs());
    dirs.extend(battlenet::game_dirs());

    let mut normalized: Vec<PathBuf> = dirs
        .into_iter()
        .map(|dir| PathBuf::from(dir.to_string_lossy().to_lowercase()))
        .collect();
    normalized.sort();
    normalized.dedup();

    println!(
        "Kontext-Erkennung: {} Spiele-Ordner über alle Launcher gefunden",
        normalized.len()
    );
    *GAME_DIRS.write().unwrap() = normalized;
}

#[cfg(not(windows))]
pub fn rescan() {}

/// Scannt beim Start und danach alle 15 Minuten (neue Installationen).
pub fn spawn_scanner() {
    std::thread::spawn(|| loop {
        rescan();
        std::thread::sleep(Duration::from_secs(15 * 60));
    });
}

/// Liegt der Pfad in einem bekannten Spiele-Ordner?
fn is_known_game(exe_path: &str) -> bool {
    let path = PathBuf::from(exe_path.to_lowercase());
    GAME_DIRS
        .read()
        .unwrap()
        .iter()
        .any(|dir| path.starts_with(dir))
}

/// Aktueller Nutzungskontext anhand des Vordergrund-Prozesses.
#[cfg(windows)]
pub fn current_context() -> UsageContext {
    match foreground_exe_path() {
        Some(exe) if is_known_game(&exe) => UsageContext::Game,
        // Fallback: unbekannter Prozess in Vollbild ist wahrscheinlich ein Spiel/Video
        _ if foreground_is_fullscreen() => UsageContext::Game,
        _ => UsageContext::Work,
    }
}

#[cfg(not(windows))]
pub fn current_context() -> UsageContext {
    UsageContext::Work
}

/// Pfad zur Executable des Vordergrund-Fensters.
#[cfg(windows)]
fn foreground_exe_path() -> Option<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        result.ok()?;
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// Nimmt das Vordergrund-Fenster den kompletten Monitor ein?
#[cfg(windows)]
fn foreground_is_fullscreen() -> bool {
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return false;
        }
        let mut rect = windows::Win32::Foundation::RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return false;
        }
        let mon = info.rcMonitor;
        rect.left <= mon.left && rect.top <= mon.top && rect.right >= mon.right && rect.bottom >= mon.bottom
    }
}
