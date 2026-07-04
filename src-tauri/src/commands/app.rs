use tauri::{AppHandle, Manager};
use std::fs;
use winreg::enums::*;
use winreg::RegKey;

fn settings_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

/// Validate that a path is safe to open — must be an existing file with an allowed extension
fn validate_open_path(path: &str) -> Result<(), String> {
    let allowed_extensions = [
        "png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico",
        "mp4", "mp3", "wav", "ogg", "flac",
        "pdf", "txt", "md", "json", "csv",
        "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "zip", "tar", "gz", "7z", "rar",
    ];

    let p = std::path::Path::new(path);

    // Must be an absolute path
    if !p.is_absolute() {
        return Err("仅允许打开绝对路径".into());
    }

    // Must exist
    if !p.exists() {
        return Err("文件不存在".into());
    }

    // Must be a file, not a directory
    if !p.is_file() {
        return Err("仅允许打开文件".into());
    }

    // Must have an allowed extension
    let ext = p.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if !allowed_extensions.contains(&ext.as_str()) {
        return Err(format!("不允许打开 .{} 类型的文件", ext));
    }

    Ok(())
}

#[tauri::command]
pub async fn exit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub async fn hide_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    validate_open_path(&path)?;
    open::that(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_settings(app: AppHandle) -> Result<serde_json::Value, String> {
    let path = settings_path(&app)?;
    if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(serde_json::json!({}))
    }
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, data: serde_json::Value) -> Result<(), String> {
    let path = settings_path(&app)?;
    let content = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

const AUTOSTART_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "FlowNote";

#[tauri::command]
pub async fn get_autostart() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags(AUTOSTART_KEY, KEY_READ).map_err(|e| e.to_string())?;
    match key.get_value::<String, _>(APP_NAME) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn set_autostart(enable: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(AUTOSTART_KEY).map_err(|e| e.to_string())?;
    if enable {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_str = exe.to_string_lossy().to_string();
        key.set_value(APP_NAME, &exe_str).map_err(|e| e.to_string())
    } else {
        // Only ignore "value not found" errors
        match key.delete_value(APP_NAME) {
            Ok(()) => Ok(()),
            Err(e) => {
                // ERROR_FILE_NOT_FOUND (2) or ERROR_PATH_NOT_FOUND (3) are acceptable
                let code = e.raw_os_error().unwrap_or(0);
                if code == 2 || code == 3 {
                    Ok(())
                } else {
                    Err(e.to_string())
                }
            }
        }
    }
}

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::Win32::UI::Shell::ITaskbarList3;
use windows::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, WS_EX_TOOLWINDOW, HWND_TOP, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_NOACTIVATE};
use windows::core::GUID;

// CLSID for TaskbarList: {56FDF344-FD6D-11d0-958A-006097C9A090}
const CLSID_TASKBARLIST: GUID = GUID::from_u128(0x56fdf344_fd6d_11d0_958a_006097c9a090);

fn get_taskbar() -> Option<ITaskbarList3> {
    unsafe { CoCreateInstance(&CLSID_TASKBARLIST, None, CLSCTX_ALL).ok() }
}

/// Hide the window from the taskbar (WS_EX_TOOLWINDOW + ITaskbarList3).
#[tauri::command]
pub async fn hide_taskbar_icon(window: tauri::Window) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    let hwnd = HWND(hwnd.0 as _);
    unsafe {
        // Set WS_EX_TOOLWINDOW to prevent taskbar entry
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_TOOLWINDOW.0 as isize);
        SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        // Also use ITaskbarList3 as backup
        if let Some(taskbar) = get_taskbar() {
            taskbar.DeleteTab(hwnd).ok();
        }
    }
    Ok(())
}

/// Show the window in the taskbar (remove WS_EX_TOOLWINDOW + ITaskbarList3).
#[tauri::command]
pub async fn show_taskbar_icon(window: tauri::Window) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    let hwnd = HWND(hwnd.0 as _);
    unsafe {
        // Remove WS_EX_TOOLWINDOW
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style & !(WS_EX_TOOLWINDOW.0 as isize));
        SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        // Also use ITaskbarList3 as backup
        if let Some(taskbar) = get_taskbar() {
            taskbar.AddTab(hwnd).ok();
        }
    }
    Ok(())
}
