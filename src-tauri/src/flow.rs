//! Flow-Erkennung: misst die Tipp-Geschwindigkeit über einen globalen
//! Keyboard-Hook. Wer beim Ablauf des Intervalls gerade schnell und anhaltend
//! tippt (Schreibfluss), bekommt die Pause minutenweise aufgeschoben —
//! hart gedeckelt, damit die Stunden-Regel nicht ausgehebelt wird.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Ab dieser Anschlagszahl in den letzten 60s gilt "im Flow" (~2 Anschläge/s anhaltend)
pub const FLOW_THRESHOLD_PER_MIN: usize = 120;
/// Mehr als so viele Sekunden Aufschub pro Intervall gibt es nicht
pub const FLOW_CAP_SECS: u64 = 15 * 60;

static KEYSTROKES: Mutex<VecDeque<Instant>> = Mutex::new(VecDeque::new());

fn record_keystroke() {
    let now = Instant::now();
    let mut strokes = KEYSTROKES.lock().unwrap();
    strokes.push_back(now);
    while let Some(front) = strokes.front() {
        if now.duration_since(*front) > Duration::from_secs(60) {
            strokes.pop_front();
        } else {
            break;
        }
    }
}

/// Anschläge in den letzten 60 Sekunden.
pub fn keystrokes_last_minute() -> usize {
    let now = Instant::now();
    let mut strokes = KEYSTROKES.lock().unwrap();
    while let Some(front) = strokes.front() {
        if now.duration_since(*front) > Duration::from_secs(60) {
            strokes.pop_front();
        } else {
            break;
        }
    }
    strokes.len()
}

pub fn is_in_flow() -> bool {
    keystrokes_last_minute() >= FLOW_THRESHOLD_PER_MIN
}

#[cfg(windows)]
pub fn spawn_keyboard_watcher() {
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    };

    unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            let msg = wparam.0 as u32;
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                record_keystroke();
            }
        }
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }

    std::thread::spawn(|| unsafe {
        // Der Hook braucht einen Thread mit Message-Loop
        if SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0).is_err() {
            eprintln!("Keyboard-Hook konnte nicht installiert werden — Flow-Erkennung inaktiv");
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

/// Tier-2: auf macOS/Linux vorerst keine Flow-Erkennung (Phase 9).
#[cfg(not(windows))]
pub fn spawn_keyboard_watcher() {}
