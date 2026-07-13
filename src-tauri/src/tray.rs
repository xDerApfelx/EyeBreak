use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

const TRAY_ID: &str = "main-tray";
const BASE_ICON: &[u8] = include_bytes!("../icons/32x32.png");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayStatus {
    /// Grün — alles läuft
    Ok,
    /// Gelb — Update verfügbar
    UpdateAvailable,
    /// Rot — etwas ist schiefgelaufen (für künftiges Fehler-Reporting reserviert)
    #[allow(dead_code)]
    Error,
}

impl TrayStatus {
    fn dot_color(self) -> [u8; 4] {
        match self {
            TrayStatus::Ok => [46, 204, 113, 255],
            TrayStatus::UpdateAvailable => [241, 196, 15, 255],
            TrayStatus::Error => [231, 76, 60, 255],
        }
    }

    fn tooltip(self, lang: &str) -> &'static str {
        match (self, lang) {
            (TrayStatus::Ok, "en") => "EyeBreak — running",
            (TrayStatus::Ok, _) => "EyeBreak — läuft",
            (TrayStatus::UpdateAvailable, "en") => "EyeBreak — update available",
            (TrayStatus::UpdateAvailable, _) => "EyeBreak — Update verfügbar",
            (TrayStatus::Error, "en") => "EyeBreak — error, check log",
            (TrayStatus::Error, _) => "EyeBreak — Fehler, Log prüfen",
        }
    }
}

/// Menü-Beschriftungen je Sprache: (Einstellungen, Updates, Beenden)
fn menu_labels(lang: &str) -> (&'static str, &'static str, &'static str) {
    match lang {
        "en" => ("Settings", "Check for updates", "Quit"),
        _ => ("Einstellungen", "Nach Updates suchen", "Beenden"),
    }
}

/// Basis-Icon mit Status-Punkt unten rechts einfärben.
fn status_icon(status: TrayStatus) -> Option<tauri::image::Image<'static>> {
    let decoded = image::load_from_memory(BASE_ICON).ok()?;
    let mut rgba = decoded.into_rgba8();
    let (width, height) = rgba.dimensions();

    let radius = (width as f32 * 0.22) as i32;
    let center_x = width as i32 - radius - 1;
    let center_y = height as i32 - radius - 1;
    let color = status.dot_color();

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let x = center_x + dx;
                let y = center_y + dy;
                if x >= 0 && y >= 0 && (x as u32) < width && (y as u32) < height {
                    rgba.put_pixel(x as u32, y as u32, image::Rgba(color));
                }
            }
        }
    }

    Some(tauri::image::Image::new_owned(
        rgba.into_raw(),
        width,
        height,
    ))
}

/// Setzt Ampel-Farbe und Tooltip des Tray-Icons.
pub fn set_status(app: &AppHandle, status: TrayStatus) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    if let Some(icon) = status_icon(status) {
        let _ = tray.set_icon(Some(icon));
    }
    let lang = current_language(app);
    let _ = tray.set_tooltip(Some(status.tooltip(&lang)));
}

fn current_language(app: &AppHandle) -> String {
    app.state::<crate::config::ConfigState>()
        .0
        .lock()
        .unwrap()
        .language
        .clone()
}

/// Baut das Tray-Menü in der gegebenen Sprache neu auf (nach Sprachwechsel).
pub fn update_language(app: &AppHandle, lang: &str) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    if let Ok(menu) = build_menu(app, lang) {
        let _ = tray.set_menu(Some(menu));
    }
    let _ = tray.set_tooltip(Some(TrayStatus::Ok.tooltip(lang)));
}

fn build_menu(app: &AppHandle, lang: &str) -> tauri::Result<Menu<tauri::Wry>> {
    let (settings_label, update_label, quit_label) = menu_labels(lang);
    let settings_item = MenuItem::with_id(app, "settings", settings_label, true, None::<&str>)?;
    let update_item = MenuItem::with_id(app, "update", update_label, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    Menu::with_items(
        app,
        &[
            &settings_item,
            &update_item,
            &PredefinedMenuItem::separator(app)?,
            &quit_item,
        ],
    )
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let lang = current_language(app);
    let menu = build_menu(app, &lang)?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(status_icon(TrayStatus::Ok).unwrap_or_else(|| app.default_window_icon().unwrap().clone()))
        .tooltip(TrayStatus::Ok.tooltip(&lang))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => show_settings(app),
            "update" => open_release_page(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_settings(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn show_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Öffnet die Release-Seite des gefundenen Updates (oder die Projektseite).
fn open_release_page(app: &AppHandle) {
    use tauri_plugin_opener::OpenerExt;
    let url = crate::updater::LATEST_RELEASE_URL
        .read()
        .unwrap()
        .clone()
        .or_else(|| {
            crate::updater::UPDATE_REPO
                .map(|repo| format!("https://github.com/{repo}/releases"))
        });
    if let Some(url) = url {
        let _ = app.opener().open_url(url, None::<&str>);
    }
}
