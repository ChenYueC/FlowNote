use crate::models::Image;
use crate::AppState;
use chrono::Utc;
use std::fs;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

fn get_assets_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|p| p.join("assets"))
        .map_err(|e| format!("无法获取应用数据目录: {}", e))
}

#[tauri::command]
pub fn get_assets_dir_cmd(app: AppHandle) -> Result<String, String> {
    // Return asset: protocol URL instead of raw filesystem path
    let dir = get_assets_dir(&app)?;
    Ok(dir.to_string_lossy().to_string())
}

/// Validate that source_path is within an allowed directory (temp dir or app data dir)
fn validate_source_path(app: &AppHandle, source_path: &str) -> Result<(), String> {
    let path = std::path::Path::new(source_path);

    if !path.is_absolute() {
        return Err("仅允许从绝对路径导入图片".into());
    }

    if !path.exists() || !path.is_file() {
        return Err("源文件不存在或不是文件".into());
    }

    // Only allow image files
    let allowed_image_extensions = ["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico", "tiff", "avif"];
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if !allowed_image_extensions.contains(&ext.as_str()) {
        return Err(format!("不允许导入 .{} 类型的文件作为图片", ext));
    }

    // Only allow files from temp directory or app data directory
    if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
        if let Ok(canonical) = path.canonicalize() {
            if canonical.starts_with(&temp_dir) {
                return Ok(());
            }
        }
    }

    if let Ok(app_data_dir) = app.path().app_local_data_dir() {
        if let Ok(canonical_app_dir) = app_data_dir.canonicalize() {
            if let Ok(canonical) = path.canonicalize() {
                if canonical.starts_with(&canonical_app_dir) {
                    return Ok(());
                }
            }
        }
    }

    Err("仅允许从临时目录或应用数据目录导入图片".into())
}

#[tauri::command]
pub async fn save_image(
    app: AppHandle,
    state: State<'_, AppState>,
    note_id: String,
    source_path: String,
) -> Result<Image, String> {
    // Validate source path
    validate_source_path(&app, &source_path)?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let assets_dir = get_assets_dir(&app)?;

    fs::create_dir_all(&assets_dir).map_err(|e| e.to_string())?;

    let ext = std::path::Path::new(&source_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    let filename = format!("{}.{}", id, ext);
    let dest_path = assets_dir.join(&filename);

    fs::copy(&source_path, &dest_path).map_err(|e| e.to_string())?;

    let dest_str = dest_path.to_string_lossy().to_string();

    sqlx::query(
        "INSERT INTO images (id, note_id, path, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&note_id)
    .bind(&dest_str)
    .bind(now)
    .execute(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Image {
        id,
        note_id,
        path: dest_str,
        width: None,
        height: None,
        hash: None,
        created_at: now,
    })
}

/// Copy an image file (any format supported by the `image` crate) to the
/// system clipboard as RGBA pixels.
///
/// The frontend previously used `TauriImage.fromBytes` + `writeImage`, but
/// `Image::from_bytes` only supports PNG/ICO (gated by Cargo features), so
/// JPEG/GIF/WEBP/BMP images failed to copy. Here we decode the bytes with the
/// `image` crate (already a dependency) and write the RGBA buffer straight to
/// the clipboard via `arboard` (the same backend the clipboard-manager plugin
/// uses internally), so every common image format works.
#[tauri::command]
pub async fn copy_image_to_clipboard(app: AppHandle, source_path: String) -> Result<(), String> {
    // Reuse the existing path validation (allowed dirs + image extension allowlist)
    validate_source_path(&app, &source_path)?;

    let bytes = std::fs::read(&source_path).map_err(|e| format!("读取图片文件失败: {}", e))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("图片解码失败: {}", e))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let raw_bytes = rgba.into_raw();

    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| format!("剪贴板初始化失败: {}", e))?;
    clipboard
        .set_image(arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: std::borrow::Cow::Owned(raw_bytes),
        })
        .map_err(|e| format!("写入剪贴板失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_note_images(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Vec<Image>, String> {
    sqlx::query_as::<_, Image>("SELECT * FROM images WHERE note_id = ? ORDER BY created_at DESC")
        .bind(&note_id)
        .fetch_all(&state.db.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_image(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let image = sqlx::query_as::<_, Image>("SELECT * FROM images WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;

    // Delete file first — if this fails, don't delete DB record (avoid orphan files)
    fs::remove_file(&image.path).map_err(|e| format!("删除图片文件失败: {}", e))?;

    sqlx::query("DELETE FROM images WHERE id = ?")
        .bind(&id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
