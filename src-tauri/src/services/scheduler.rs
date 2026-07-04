use crate::models::reminder::Reminder;
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};

/// Start a background scheduler that polls for due reminders every 2 seconds.
/// Emits `reminder:due` events and sends native notifications.
pub fn start_scheduler(app: AppHandle, pool: SqlitePool) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));

        loop {
            interval.tick().await;

            let now = Utc::now().timestamp();
            // Only trigger reminders that just became due (within last 10 seconds),
            // skip reminders that were already overdue before the app started
            let cutoff = now - 10;

            let due: Vec<Reminder> = match sqlx::query_as::<_, Reminder>(
                "SELECT id, note_id, title, content, remind_at, completed, snoozed, deleted, cancelled, created_at, updated_at FROM reminders WHERE completed = 0 AND deleted = 0 AND cancelled = 0 AND snoozed = 0 AND remind_at <= ? AND remind_at >= ? ORDER BY remind_at ASC",
            )
            .bind(now)
            .bind(cutoff)
            .fetch_all(&pool)
            .await
            {
                Ok(rows) => rows,
                Err(_) => continue,
            };

            for reminder in &due {
                // Emit event to frontend — popup handles user action
                let _ = app.emit("reminder:due", serde_json::to_value(reminder).unwrap_or_default());

                // Mark as snoozed (auto-snooze) to prevent re-firing;
                // popup can then set snoozed=0 + new remind_at via snooze, or completed=1 via complete
                let _ = sqlx::query(
                    "UPDATE reminders SET snoozed = 1, updated_at = ? WHERE id = ?",
                )
                .bind(now)
                .bind(&reminder.id)
                .execute(&pool)
                .await;
            }
        }
    });
}
