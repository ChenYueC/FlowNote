mod commands;
mod db;
mod models;
mod services;
mod window;

use db::Database;
use std::sync::Arc;
use tauri::Manager;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tokio::sync::Mutex;
use window::FloatingWindowManager;

pub struct AppState {
    pub db: Database,
    pub window_manager: Arc<Mutex<FloatingWindowManager>>,
}

/// Recursively copy a directory. Used as fallback when rename fails (cross-volume).
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Migrate old DB from AppData\Local\FlowNote to AppData\Local\com.flownote.app
            // Use recursive copy instead of rename to handle cross-volume moves on Windows
            let new_data_dir = app.path().app_local_data_dir()?;
            let old_data_dir = dirs::data_local_dir()
                .map(|d| d.join("FlowNote"))
                .unwrap_or_default();
            if old_data_dir.exists() && !new_data_dir.exists() {
                // Try rename first (fast, same volume)
                if std::fs::rename(&old_data_dir, &new_data_dir).is_err() {
                    // Fallback: copy recursively then remove old (cross-volume)
                    if let Err(e) = copy_dir_recursive(&old_data_dir, &new_data_dir) {
                        eprintln!("Migration failed: {}. Old data preserved at {:?}", e, old_data_dir);
                    } else {
                        let _ = std::fs::remove_dir_all(&old_data_dir);
                    }
                }
            }

            let db_path = db::get_db_path(app.handle())?;
            let db = tauri::async_runtime::block_on(Database::new(&db_path))?;
            let window_manager = Arc::new(Mutex::new(FloatingWindowManager::new()));

            app.manage(AppState {
                db,
                window_manager,
            });

            // Register global shortcut: Alt+Space to toggle main window
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            let app_handle = app.handle().clone();
            app.global_shortcut()
                .on_shortcut("Alt+Space", move |_app, _shortcut, _event| {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                })?;

            // Build system tray — left-click show, right-click custom menu popup
            let tray_app_handle = app.handle().clone();
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().expect("Default window icon must be set in tauri.conf.json"))
                .tooltip("FlowNote")
                .on_tray_icon_event(move |tray, event| {
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            if let Some(window) = tray_app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        TrayIconEvent::Click {
                            button: MouseButton::Right,
                            button_state: MouseButtonState::Up,
                            position,
                            ..
                        } => {
                            let app = tray.app_handle();
                            let label = "tray-menu";
                            let menu_win = app.get_webview_window(label);
                            if let Some(win) = menu_win {
                                let _ = win.set_position(tauri::PhysicalPosition::new(
                                    position.x as i32 + 4,
                                    position.y as i32 - 34,
                                ));
                                let _ = win.show();
                                let _ = win.set_focus();
                            } else {
                                let win = tauri::WebviewWindowBuilder::new(
                                    app,
                                    label,
                                    tauri::WebviewUrl::App("index.html?window=tray-menu".into()),
                                )
                                .title("")
                                .inner_size(100.0, 36.0)
                                .decorations(false)
                                .transparent(true)
                                .shadow(false)
                                .always_on_top(true)
                                .skip_taskbar(true)
                                .resizable(false)
                                .visible(false)
                                .build();
                                if let Ok(w) = win {
                                    let _ = w.set_position(tauri::PhysicalPosition::new(
                                        position.x as i32 + 4,
                                        position.y as i32 - 34,
                                    ));
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Start reminder background scheduler
            let pool = app.state::<AppState>().db.pool.clone();
            services::scheduler::start_scheduler(app.handle().clone(), pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::exit_app,
            commands::app::hide_main_window,
            commands::app::open_file,
            commands::app::load_settings,
            commands::app::save_settings,
            commands::app::get_autostart,
            commands::app::set_autostart,
            commands::app::hide_taskbar_icon,
            commands::app::show_taskbar_icon,
            commands::notes::create_note,
            commands::notes::create_daily_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::get_note,
            commands::notes::list_notes,
            commands::notes::list_timeline_notes,
            commands::notes::list_timeline_items,
            commands::notes::search_notes,
            commands::notes::toggle_favorite,
            commands::notes::move_note_to_workspace,
            commands::notes::list_workspaces,
            commands::notes::create_workspace,
            commands::notes::delete_workspace,
            commands::notes::rename_workspace,
            commands::floating::create_floating_window,
            commands::floating::close_floating_window,
            commands::floating::update_window_state,
            commands::floating::list_floating_windows,
            commands::images::save_image,
            commands::images::get_note_images,
            commands::images::delete_image,
            commands::images::get_assets_dir_cmd,
            commands::images::copy_image_to_clipboard,
            commands::reminders::create_reminder,
            commands::reminders::get_reminder,
            commands::reminders::list_reminders,
            commands::reminders::update_reminder,
            commands::reminders::delete_reminder,
            commands::reminders::snooze_reminder,
            commands::reminders::complete_reminder,
            commands::reminders::cancel_reminder,
            commands::screenshot::capture_screenshot,
            commands::screenshot::capture_fullscreen_screenshot,
            commands::screenshot::select_and_capture_screenshot,
            commands::screenshot::select_screen_area,
            commands::screenshot::capture_fixed_area,
            commands::screenshot::get_all_monitors,
            commands::updater::check_update,
            commands::updater::download_and_install,
            commands::updater::show_update_confirm,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
