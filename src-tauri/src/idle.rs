/// Sekunden seit dem letzten Maus-/Tastatur-Input, systemweit.
#[cfg(windows)]
pub fn idle_seconds() -> u64 {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    unsafe {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut info).as_bool() {
            let now = GetTickCount();
            // dwTime läuft nach ~49 Tagen über — wrapping_sub liefert trotzdem die korrekte Differenz
            u64::from(now.wrapping_sub(info.dwTime)) / 1000
        } else {
            0
        }
    }
}

/// Linux (Tier 2): unter X11 liefert `xprintidle` die Idle-Zeit.
/// Unter Wayland blockiert das Sicherheitsmodell globales Input-Monitoring —
/// dort gilt "nie idle" (Timer läuft durch statt falsch zu pausieren).
#[cfg(target_os = "linux")]
pub fn idle_seconds() -> u64 {
    use std::sync::OnceLock;
    static WAYLAND_WARNED: OnceLock<()> = OnceLock::new();

    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        WAYLAND_WARNED.get_or_init(|| {
            eprintln!(
                "Wayland erkannt: Idle-Erkennung ist hier nicht möglich — der Timer läuft ohne Idle-Pause weiter."
            );
        });
        return 0;
    }

    std::process::Command::new("xprintidle")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .map(|millis| millis / 1000)
        .unwrap_or(0)
}

/// macOS (Tier 2): HIDIdleTime aus dem IO-Registry — funktioniert ohne
/// Sonderberechtigungen (Nanosekunden → Sekunden).
#[cfg(target_os = "macos")]
pub fn idle_seconds() -> u64 {
    let output = std::process::Command::new("sh")
        .args([
            "-c",
            "ioreg -c IOHIDSystem | awk '/HIDIdleTime/ {print $NF; exit}'",
        ])
        .output();
    output
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .map(|nanos| nanos / 1_000_000_000)
        .unwrap_or(0)
}
