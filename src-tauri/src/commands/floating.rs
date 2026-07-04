use crate::models::floating_window::{
    CreateFloatingWindowInput, FloatingWindow, UpdateWindowStateInput,
};
use crate::AppState;
use chrono::Utc;
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

#[tauri::command]
pub async fn create_floating_window(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    input: CreateFloatingWindowInput,
) -> Result<FloatingWindow, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let width = input.width.unwrap_or(320);
    let height = input.height.unwrap_or(240);
    let label = format!("floating-{}", id);

    let note = sqlx::query_as::<_, crate::models::Note>(
        "SELECT * FROM notes WHERE id = ?",
    )
    .bind(&input.note_id)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    // Insert floating window record
    sqlx::query(
        "INSERT INTO floating_windows (id, note_id, type, title, x, y, width, height, created_at, updated_at) VALUES (?, ?, 'normal', ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.note_id)
    .bind(&note.title)
    .bind(input.x)
    .bind(input.y)
    .bind(width)
    .bind(height)
    .bind(now)
    .bind(now)
    .execute(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    // Create the actual window
    let url = format!(
        "index.html?window=floating&note_id={}&window_id={}",
        input.note_id, id
    );

    WebviewWindowBuilder::new(&app, &label, WebviewUrl::App(url.into()))
        .title(&note.title)
        .inner_size(width as f64, height as f64)
        .position(input.x as f64, input.y as f64)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .resizable(true)
        .visible(true)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(FloatingWindow {
        id,
        note_id: input.note_id,
        window_type: "normal".into(),
        title: Some(note.title),
        x: input.x,
        y: input.y,
        width,
        height,
        minimized: 0,
        pinned: 1,
        auto_hide: 0,
        locked: 0,
        opacity: 1.0,
        monitor_id: None,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub async fn close_floating_window(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    window_id: String,
) -> Result<(), String> {
    let label = format!("floating-{}", window_id);

    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
    }

    sqlx::query("DELETE FROM floating_windows WHERE id = ?")
        .bind(&window_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_window_state(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    input: UpdateWindowStateInput,
) -> Result<(), String> {
    let now = Utc::now().timestamp();

    let label = format!("floating-{}", input.id);

    // Update the actual window
    if let Some(window) = app.get_webview_window(&label) {
        if let Some(pinned) = input.pinned {
            window.set_always_on_top(pinned == 1).map_err(|e| e.to_string())?;
        }
        if let (Some(x), Some(y)) = (input.x, input.y) {
            window
                .set_position(tauri::PhysicalPosition::new(x, y))
                .map_err(|e| e.to_string())?;
        }
        if let (Some(w), Some(h)) = (input.width, input.height) {
            window
                .set_size(tauri::PhysicalSize::new(w as u32, h as u32))
                .map_err(|e| e.to_string())?;
        }
    }

    // Use transaction for atomic database updates
    let mut tx = state.db.pool.begin().await.map_err(|e| e.to_string())?;

    // Always update timestamp
    sqlx::query("UPDATE floating_windows SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&input.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // Update position
    if let (Some(x), Some(y)) = (input.x, input.y) {
        sqlx::query("UPDATE floating_windows SET x = ?, y = ? WHERE id = ?")
            .bind(x)
            .bind(y)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Update pinned
    if let Some(pinned) = input.pinned {
        sqlx::query("UPDATE floating_windows SET pinned = ? WHERE id = ?")
            .bind(pinned)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Update opacity (stored in DB, applied via frontend CSS)
    if let Some(opacity) = input.opacity {
        sqlx::query("UPDATE floating_windows SET opacity = ? WHERE id = ?")
            .bind(opacity)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_floating_windows(
    state: State<'_, AppState>,
) -> Result<Vec<FloatingWindow>, String> {
    sqlx::query_as::<_, FloatingWindow>(
        "SELECT id, note_id, type AS window_type, title, x, y, width, height, minimized, pinned, auto_hide, locked, opacity, monitor_id, created_at, updated_at FROM floating_windows ORDER BY updated_at DESC",
    )
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| e.to_string())
}
