use crate::models::note::{CreateNoteInput, Note, UpdateNoteInput};
use crate::models::timeline::TimelineItem;
use crate::AppState;
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn create_note(
    state: State<'_, AppState>,
    input: CreateNoteInput,
) -> Result<Note, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let workspace = input.workspace.unwrap_or_else(|| "default".into());

    let note_type = "add".to_string();

    sqlx::query(
        "INSERT INTO notes (id, title, content, workspace, type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&input.content)
    .bind(&workspace)
    .bind(&note_type)
    .bind(now)
    .bind(now)
    .execute(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Note {
        id,
        title: input.title,
        content: input.content,
        workspace: Some(workspace),
        created_at: now,
        updated_at: now,
        archived: 0,
        favorite: 0,
        r#type: note_type,
        color_ranges: "[]".to_string(),
    })
}

#[tauri::command]
pub async fn create_daily_note(
    state: State<'_, AppState>,
) -> Result<Note, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let workspace = "daily";
    let note_type = "add";

    // Use INSERT ... SELECT WHERE NOT EXISTS to avoid TOCTOU race condition
    sqlx::query(
        "INSERT INTO notes (id, title, content, workspace, type, created_at, updated_at)
         SELECT ?, ?, '', ?, ?, ?, ?
         WHERE NOT EXISTS (SELECT 1 FROM notes WHERE title = ? AND workspace = 'daily')",
    )
    .bind(&id)
    .bind(&today)
    .bind(workspace)
    .bind(note_type)
    .bind(now)
    .bind(now)
    .bind(&today)
    .execute(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    // Fetch the note (either just created or already existing)
    let note = sqlx::query_as::<_, Note>(
        "SELECT * FROM notes WHERE title = ? AND workspace = 'daily' LIMIT 1",
    )
    .bind(&today)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(note)
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, AppState>,
    input: UpdateNoteInput,
) -> Result<Note, String> {
    let now = Utc::now().timestamp();

    // Use transaction to ensure atomicity
    let mut tx = state.db.pool.begin().await.map_err(|e| e.to_string())?;

    // Always update updated_at
    sqlx::query("UPDATE notes SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&input.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(title) = &input.title {
        sqlx::query("UPDATE notes SET title = ? WHERE id = ?")
            .bind(title)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(content) = &input.content {
        sqlx::query("UPDATE notes SET content = ? WHERE id = ?")
            .bind(content)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(color_ranges) = &input.color_ranges {
        sqlx::query("UPDATE notes SET color_ranges = ? WHERE id = ?")
            .bind(color_ranges)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    get_note(state, input.id).await
}

#[tauri::command]
pub async fn delete_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let now = Utc::now().timestamp();
    sqlx::query("UPDATE notes SET archived = 1, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, id: String) -> Result<Note, String> {
    sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_notes(
    state: State<'_, AppState>,
    workspace: Option<String>,
) -> Result<Vec<Note>, String> {
    let ws = workspace.unwrap_or_else(|| "default".into());
    sqlx::query_as::<_, Note>(
        "SELECT * FROM notes WHERE archived = 0 AND workspace = ? ORDER BY updated_at DESC",
    )
    .bind(&ws)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_timeline_notes(
    state: State<'_, AppState>,
    page: Option<u32>,
) -> Result<Vec<Note>, String> {
    let p = page.unwrap_or(1).max(1);
    let limit: u32 = 30;
    let offset = (p - 1) * limit;
    sqlx::query_as::<_, Note>(
        "SELECT * FROM notes ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_timeline_items(
    state: State<'_, AppState>,
    page: Option<u32>,
) -> Result<Vec<TimelineItem>, String> {
    let p = page.unwrap_or(1).max(1);
    let limit: u32 = 30;
    let offset = (p - 1) * limit;
    sqlx::query_as::<_, TimelineItem>(
        "SELECT id, title, content, created_at, 'note' AS item_type, type AS item_action, archived FROM notes
         UNION ALL
         SELECT id, title, content, created_at, 'reminder' AS item_type, '' AS item_action, deleted AS archived FROM reminders
         ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_notes(
    state: State<'_, AppState>,
    query: String,
    workspace: Option<String>,
) -> Result<Vec<Note>, String> {
    let ws = workspace.unwrap_or_else(|| "default".into());
    // Escape LIKE wildcards in user input
    let escaped = query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    let pattern = format!("%{}%", escaped);
    sqlx::query_as::<_, Note>(
        "SELECT * FROM notes WHERE archived = 0 AND workspace = ? AND (title LIKE ? ESCAPE '\\' OR content LIKE ? ESCAPE '\\') ORDER BY updated_at DESC",
    )
    .bind(&ws)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_favorite(state: State<'_, AppState>, id: String) -> Result<Note, String> {
    let note = get_note_internal(&state, &id).await?;
    let new_fav = if note.favorite == 1 { 0 } else { 1 };
    let now = Utc::now().timestamp();

    sqlx::query("UPDATE notes SET favorite = ?, updated_at = ? WHERE id = ?")
        .bind(new_fav)
        .bind(now)
        .bind(&id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;

    get_note(state, id).await
}

#[tauri::command]
pub async fn move_note_to_workspace(
    state: State<'_, AppState>,
    id: String,
    workspace: String,
) -> Result<Note, String> {
    let now = Utc::now().timestamp();
    sqlx::query("UPDATE notes SET workspace = ?, updated_at = ? WHERE id = ?")
        .bind(&workspace)
        .bind(now)
        .bind(&id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;

    get_note(state, id).await
}

#[tauri::command]
pub async fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM workspaces ORDER BY name",
    )
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

#[tauri::command]
pub async fn create_workspace(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let now = Utc::now().timestamp();
    sqlx::query("INSERT OR IGNORE INTO workspaces (name, created_at) VALUES (?, ?)")
        .bind(&name)
        .bind(now)
        .execute(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_workspace(
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    // Use transaction for atomic multi-step operation
    let mut tx = state.db.pool.begin().await.map_err(|e| e.to_string())?;

    let now = Utc::now().timestamp();
    sqlx::query("UPDATE notes SET workspace = 'default', updated_at = ? WHERE workspace = ?")
        .bind(now)
        .bind(&name)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM workspaces WHERE name = ? AND name != 'default'")
        .bind(&name)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn rename_workspace(
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let mut tx = state.db.pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("UPDATE workspaces SET name = ? WHERE name = ? AND name != 'default'")
        .bind(&new_name)
        .bind(&old_name)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE notes SET workspace = ? WHERE workspace = ?")
        .bind(&new_name)
        .bind(&old_name)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn get_note_internal(state: &State<'_, AppState>, id: &str) -> Result<Note, String> {
    sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| e.to_string())
}
