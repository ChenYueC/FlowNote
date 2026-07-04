use crate::models::reminder::{CreateReminderInput, Reminder, ReminderCounts, ReminderListResponse, UpdateReminderInput};
use crate::AppState;
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn create_reminder(
    state: State<'_, AppState>,
    input: CreateReminderInput,
) -> Result<Reminder, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let content = input.content.unwrap_or_default();

    // Check for duplicate: same minute, non-deleted, non-cancelled
    let minute_start = input.remind_at - (input.remind_at % 60);
    let minute_end = minute_start + 59;
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reminders WHERE deleted = 0 AND cancelled = 0 AND remind_at >= ? AND remind_at <= ?",
    )
    .bind(minute_start)
    .bind(minute_end)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    if existing > 0 {
        return Err("该时间已有提醒，请选择其他时间".into());
    }

    sqlx::query(
        "INSERT INTO reminders (id, note_id, title, content, remind_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.note_id)
    .bind(&input.title)
    .bind(&content)
    .bind(input.remind_at)
    .bind(now)
    .bind(now)
    .execute(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Reminder {
        id,
        note_id: input.note_id,
        title: input.title,
        content,
        remind_at: input.remind_at,
        completed: 0,
        snoozed: 0,
        deleted: 0,
        cancelled: 0,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub async fn get_reminder(
    state: State<'_, AppState>,
    id: String,
) -> Result<Reminder, String> {
    sqlx::query_as::<_, Reminder>("SELECT id, note_id, title, content, remind_at, completed, snoozed, deleted, cancelled, created_at, updated_at FROM reminders WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_reminders(
    state: State<'_, AppState>,
    note_id: Option<String>,
    filter: Option<String>,
) -> Result<ReminderListResponse, String> {
    let cols = "id, note_id, title, content, remind_at, completed, snoozed, deleted, cancelled, created_at, updated_at";
    let filter_type = filter.as_deref().unwrap_or("all");
    let now = Utc::now().timestamp();
    let overdue_cutoff = now - 65;
    let today_start = now - (now % 86400);
    let today_end = today_start + 86400;

    // Build dynamic WHERE clause with parameterized bindings
    let mut where_parts: Vec<String> = vec!["deleted = 0".to_string()];
    let mut count_where_parts: Vec<String> = vec!["deleted = 0".to_string()];

    if let Some(nid) = &note_id {
        where_parts.push("note_id = ?".to_string());
        count_where_parts.push("note_id = ?".to_string());
    }

    match filter_type {
        "today" => {
            where_parts.push("cancelled = 0".to_string());
            where_parts.push("remind_at >= ?".to_string());
            where_parts.push("remind_at < ?".to_string());
        }
        "overdue" => {
            where_parts.push("completed = 0".to_string());
            where_parts.push("cancelled = 0".to_string());
            where_parts.push("remind_at < ?".to_string());
        }
        "cancelled" => {
            where_parts.push("cancelled = 1".to_string());
        }
        "completed" => {
            where_parts.push("completed = 1".to_string());
        }
        _ => {}
    }

    let where_clause = where_parts.join(" AND ");

    // ORDER BY uses parameterized values for robustness
    let order = match filter_type {
        "all" => format!(
            "ORDER BY CASE WHEN completed = 0 AND cancelled = 0 AND remind_at >= ? THEN 0 WHEN completed = 0 AND cancelled = 0 AND remind_at < ? THEN 1 WHEN cancelled = 1 THEN 2 ELSE 3 END ASC, CASE WHEN completed = 0 AND cancelled = 0 AND remind_at >= ? THEN remind_at ELSE -remind_at END ASC"
        ),
        _ => "ORDER BY remind_at DESC".to_string(),
    };

    let sql = format!("SELECT {} FROM reminders WHERE {} {}", cols, where_clause, order);

    // Build and bind the main query
    let mut query = sqlx::query_as::<_, Reminder>(&sql);
    if note_id.is_some() {
        query = query.bind(note_id.as_deref().unwrap());
    }
    match filter_type {
        "today" => {
            query = query.bind(today_start).bind(today_end);
        }
        "overdue" => {
            query = query.bind(overdue_cutoff);
        }
        "all" => {
            // ORDER BY has 3 parameterized overdue_cutoff references
            query = query.bind(overdue_cutoff).bind(overdue_cutoff).bind(overdue_cutoff);
        }
        _ => {}
    }
    let reminders = query.fetch_all(&state.db.pool).await.map_err(|e| e.to_string())?;

    // Count queries — use conditional aggregation in a single query for efficiency
    let count_where = count_where_parts.join(" AND ");
    let count_sql = format!(
        "SELECT \
            COUNT(*) as all_count, \
            SUM(CASE WHEN cancelled = 0 AND remind_at >= ? AND remind_at < ? THEN 1 ELSE 0 END) as today_count, \
            SUM(CASE WHEN completed = 0 AND cancelled = 0 AND remind_at < ? THEN 1 ELSE 0 END) as overdue_count, \
            SUM(CASE WHEN cancelled = 1 THEN 1 ELSE 0 END) as cancelled_count, \
            SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) as completed_count \
        FROM reminders WHERE {}",
        count_where
    );

    let mut count_query = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(&count_sql)
        .bind(today_start)
        .bind(today_end)
        .bind(overdue_cutoff);
    if note_id.is_some() {
        count_query = count_query.bind(note_id.as_deref().unwrap());
    }
    let (count_all, count_today, count_overdue, count_cancelled, count_completed) =
        count_query.fetch_one(&state.db.pool).await.map_err(|e| e.to_string())?;

    Ok(ReminderListResponse {
        reminders,
        counts: ReminderCounts {
            all: count_all,
            today: count_today,
            overdue: count_overdue,
            cancelled: count_cancelled,
            completed: count_completed,
        },
    })
}

#[tauri::command]
pub async fn update_reminder(
    state: State<'_, AppState>,
    input: UpdateReminderInput,
) -> Result<Reminder, String> {
    let now = Utc::now().timestamp();

    // Use transaction to ensure atomicity
    let mut tx = state.db.pool.begin().await.map_err(|e| e.to_string())?;

    // Always update updated_at
    sqlx::query("UPDATE reminders SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&input.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(title) = &input.title {
        sqlx::query("UPDATE reminders SET title = ? WHERE id = ?")
            .bind(title)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(content) = &input.content {
        sqlx::query("UPDATE reminders SET content = ? WHERE id = ?")
            .bind(content)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(remind_at) = input.remind_at {
        sqlx::query("UPDATE reminders SET remind_at = ?, snoozed = 0 WHERE id = ?")
            .bind(remind_at)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(completed) = input.completed {
        sqlx::query("UPDATE reminders SET completed = ? WHERE id = ?")
            .bind(completed)
            .bind(&input.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    // Fetch outside transaction to get the final state
    get_reminder(state, input.id).await
}

#[tauri::command]
pub async fn delete_reminder(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let now = Utc::now().timestamp();
    sqlx::query("UPDATE reminders SET deleted = 1, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn snooze_reminder(
    state: State<'_, AppState>,
    id: String,
    minutes: i64,
) -> Result<Reminder, String> {
    let now = Utc::now().timestamp();
    let new_remind_at = now + minutes * 60;

    sqlx::query(
        "UPDATE reminders SET remind_at = ?, snoozed = 1, completed = 0, updated_at = ? WHERE id = ?",
    )
    .bind(new_remind_at)
    .bind(now)
    .bind(&id)
    .execute(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    get_reminder(state, id).await
}

#[tauri::command]
pub async fn complete_reminder(
    state: State<'_, AppState>,
    id: String,
) -> Result<Reminder, String> {
    let now = Utc::now().timestamp();

    sqlx::query("UPDATE reminders SET completed = 1, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;

    get_reminder(state, id).await
}

#[tauri::command]
pub async fn cancel_reminder(
    state: State<'_, AppState>,
    id: String,
) -> Result<Reminder, String> {
    let now = Utc::now().timestamp();
    sqlx::query("UPDATE reminders SET cancelled = 1, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(&id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| e.to_string())?;
    get_reminder(state, id).await
}
