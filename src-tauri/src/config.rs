use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Locker,
    Normal,
    Streng,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// Länge eines aktiven Intervalls in Minuten (Standard: 60)
    pub interval_minutes: u32,
    /// Länge der Augenpause in Minuten (Standard: 5)
    pub break_minutes: u32,
    /// Inaktivität in Minuten, ab der der Timer pausiert (Standard: 5)
    pub idle_buffer_minutes: u32,
    pub difficulty: Difficulty,
    /// "system" | "light" | "dark"
    pub theme: String,
    pub autostart: bool,
    /// Overlay dauerhaft zeigen statt nur in den letzten 10 Minuten
    pub overlay_always_visible: bool,
    /// "de" | "en"
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            interval_minutes: 60,
            break_minutes: 5,
            idle_buffer_minutes: 5,
            difficulty: Difficulty::Normal,
            theme: "system".into(),
            autostart: true,
            overlay_always_visible: false,
            language: "de".into(),
        }
    }
}

pub struct ConfigState(pub Mutex<Settings>);

fn config_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("config.json"))
}

pub fn load(app: &AppHandle) -> Settings {
    let settings = config_path(app)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    sanitize(settings)
}

/// Hält handeditierte Configs im gültigen Bereich — ein Intervall von 0
/// würde den Timer sonst in eine Endlosschleife aus Sofort-Pausen schicken.
fn sanitize(mut settings: Settings) -> Settings {
    settings.interval_minutes = settings.interval_minutes.clamp(1, 480);
    settings.break_minutes = settings.break_minutes.clamp(1, 60);
    settings.idle_buffer_minutes = settings.idle_buffer_minutes.clamp(1, 60);
    if !["de", "en"].contains(&settings.language.as_str()) {
        settings.language = "de".into();
    }
    settings
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = config_path(app).ok_or("Konfigurationsverzeichnis nicht ermittelbar")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings(state: State<ConfigState>) -> Settings {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_settings(
    app: AppHandle,
    state: State<ConfigState>,
    settings: Settings,
) -> Result<(), String> {
    let settings = sanitize(settings);
    // Autostart-Änderung sofort im OS registrieren
    {
        use tauri_plugin_autostart::ManagerExt;
        let autolaunch = app.autolaunch();
        let result = if settings.autostart {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        if let Err(e) = result {
            // Im Dev-Build zeigt der Autostart auf das Dev-Binary — nicht fatal
            eprintln!("Autostart konnte nicht gesetzt werden: {e}");
        }
    }
    save(&app, &settings)?;
    *state.0.lock().unwrap() = settings.clone();
    // Neue Intervall-/Pausenwerte gelten ab sofort — Timer neu starten
    crate::timer::reset(&app);
    // Tray-Beschriftung folgt der Sprache, alle Fenster bekommen die neuen Settings
    crate::tray::update_language(&app, &settings.language);
    use tauri::Emitter;
    let _ = app.emit("settings-changed", settings);
    Ok(())
}
