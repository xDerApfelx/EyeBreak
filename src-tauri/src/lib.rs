mod config;
mod context;
mod flow;
mod idle;
mod lock;
mod overlay;
mod timer;
mod tray;
mod updater;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let settings = config::load(app.handle());

            // Autostart-Status ans OS angleichen
            {
                use tauri_plugin_autostart::ManagerExt;
                let autolaunch = app.autolaunch();
                let result = if settings.autostart {
                    autolaunch.enable()
                } else {
                    autolaunch.disable()
                };
                if let Err(e) = result {
                    eprintln!("Autostart konnte nicht gesetzt werden: {e}");
                }
            }

            app.manage(config::ConfigState(std::sync::Mutex::new(settings)));
            tray::setup_tray(app.handle())?;
            overlay::create_overlay_window(app.handle())?;
            flow::spawn_keyboard_watcher();
            context::spawn_scanner();
            timer::spawn(app.handle().clone());
            updater::spawn_update_checker(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Schließen des Settings-Fensters beendet die App nicht — sie lebt im Tray weiter
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            config::get_settings,
            config::set_settings,
            timer::get_timer_status,
            timer::snooze,
            timer::start_break_now,
            timer::skip_break
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
