use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewWindow};

pub const OVERLAY_LABEL: &str = "overlay";
const OVERLAY_WIDTH: f64 = 380.0;
const OVERLAY_HEIGHT: f64 = 150.0;
const MARGIN: f64 = 16.0;
/// So lange muss der Cursor über dem Overlay verweilen, bevor es interaktiv wird —
/// verhindert, dass ein kurzes Drüberfahren im Spiel Klicks schluckt.
const HOVER_DWELL: Duration = Duration::from_millis(300);

/// Merker, ob das Overlay gerade sichtbar sein soll (steuert das Cursor-Polling).
pub static OVERLAY_VISIBLE: AtomicBool = AtomicBool::new(false);
/// Blocking-Modus (Streng-Pause): Overlay ist Vollbild und schluckt allen Input.
static OVERLAY_BLOCKING: AtomicBool = AtomicBool::new(false);

pub fn create_overlay_window(app: &AppHandle) -> tauri::Result<()> {
    let window = tauri::WebviewWindowBuilder::new(
        app,
        OVERLAY_LABEL,
        tauri::WebviewUrl::App("overlay.html".into()),
    )
    .title("AugenSchonen Overlay")
    .inner_size(OVERLAY_WIDTH, OVERLAY_HEIGHT)
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .focused(false)
    .shadow(false)
    .visible(false)
    .build()?;

    // Oben rechts auf dem primären Monitor platzieren
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let scale = monitor.scale_factor();
        let size = monitor.size().to_logical::<f64>(scale);
        let _ = window.set_position(LogicalPosition::new(
            size.width - OVERLAY_WIDTH - MARGIN,
            MARGIN,
        ));
    }

    // Per Default klick-durchlässig — Spiel/Programm dahinter bekommt die Maus
    let _ = window.set_ignore_cursor_events(true);

    spawn_hover_watcher(app.clone());
    Ok(())
}

/// Zeigt/versteckt das Overlay. Wird vom Timer-Tick gesteuert.
pub fn set_visible(app: &AppHandle, visible: bool) {
    if OVERLAY_VISIBLE.swap(visible, Ordering::Relaxed) == visible {
        return; // keine Änderung
    }
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        if visible {
            let _ = window.set_ignore_cursor_events(true);
            let _ = window.show();
        } else {
            let _ = window.hide();
        }
    }
}

/// Streng-Modus: Overlay wird Vollbild, fängt Maus + Fokus, bis die Pause vorbei ist.
pub fn set_blocking(app: &AppHandle, blocking: bool) {
    if OVERLAY_BLOCKING.swap(blocking, Ordering::Relaxed) == blocking {
        return;
    }
    let Some(window) = app.get_webview_window(OVERLAY_LABEL) else {
        return;
    };
    if blocking {
        let _ = window.set_ignore_cursor_events(false);
        let _ = window.set_fullscreen(true);
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("overlay-interactive", true);
    } else {
        let _ = window.set_fullscreen(false);
        let _ = window.set_size(LogicalSize::new(OVERLAY_WIDTH, OVERLAY_HEIGHT));
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let scale = monitor.scale_factor();
            let size = monitor.size().to_logical::<f64>(scale);
            let _ = window.set_position(LogicalPosition::new(
                size.width - OVERLAY_WIDTH - MARGIN,
                MARGIN,
            ));
        }
        let _ = window.set_ignore_cursor_events(true);
        let _ = window.emit("overlay-interactive", false);
    }
}

pub fn is_blocking() -> bool {
    OVERLAY_BLOCKING.load(Ordering::Relaxed)
}

fn overlay_contains_cursor(window: &WebviewWindow) -> bool {
    let (Ok(cursor), Ok(pos), Ok(size)) = (
        window.cursor_position(),
        window.outer_position(),
        window.outer_size(),
    ) else {
        return false;
    };
    let x = cursor.x as i32;
    let y = cursor.y as i32;
    x >= pos.x
        && x < pos.x + size.width as i32
        && y >= pos.y
        && y < pos.y + size.height as i32
}

/// Beobachtet die Cursor-Position: verweilt der Cursor auf dem sichtbaren Overlay,
/// wird es interaktiv (Buttons klickbar); verlässt er es, wieder klick-durchlässig.
fn spawn_hover_watcher(app: AppHandle) {
    std::thread::spawn(move || {
        let mut hover_since: Option<Instant> = None;
        let mut interactive = false;

        loop {
            std::thread::sleep(Duration::from_millis(150));
            // Im Blocking-Modus (Streng-Pause) regelt set_blocking die Interaktivität
            if !OVERLAY_VISIBLE.load(Ordering::Relaxed) || is_blocking() {
                hover_since = None;
                interactive = false;
                continue;
            }
            let Some(window) = app.get_webview_window(OVERLAY_LABEL) else {
                continue;
            };

            let inside = overlay_contains_cursor(&window);
            if inside {
                let since = *hover_since.get_or_insert_with(Instant::now);
                if !interactive && since.elapsed() >= HOVER_DWELL {
                    interactive = true;
                    let _ = window.set_ignore_cursor_events(false);
                    let _ = window.emit("overlay-interactive", true);
                }
            } else {
                hover_since = None;
                if interactive {
                    interactive = false;
                    let _ = window.set_ignore_cursor_events(true);
                    let _ = window.emit("overlay-interactive", false);
                }
            }
        }
    });
}
