/// Sperrt die OS-Session (Streng-Eskalation). Nichts wird geschlossen oder
/// beendet — nach Wiederanmeldung ist alles unverändert da.
#[cfg(windows)]
pub fn lock_session() {
    unsafe {
        let _ = windows::Win32::System::Shutdown::LockWorkStation();
    }
}

#[cfg(target_os = "linux")]
pub fn lock_session() {
    // systemd-logind, funktioniert auf den meisten modernen Distros
    let _ = std::process::Command::new("loginctl")
        .arg("lock-session")
        .status();
}

/// macOS hat keine offizielle öffentliche Lock-API (Tier 2) —
/// dort passiert bewusst nichts statt auf private APIs zurückzugreifen.
#[cfg(target_os = "macos")]
pub fn lock_session() {}
