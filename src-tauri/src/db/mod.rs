use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use tauri::{AppHandle, Manager};

pub struct Database {
    pub pool: SqlitePool,
}

pub fn get_db_path(app: &AppHandle) -> Result<std::path::PathBuf, tauri::Error> {
    let data_dir = app.path().app_local_data_dir()?;

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| tauri::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    Ok(data_dir.join("flownote.db"))
}

impl Database {
    pub async fn new(path: &Path) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let db = Self { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL DEFAULT '',
                workspace TEXT NOT NULL DEFAULT 'default',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                archived INTEGER NOT NULL DEFAULT 0,
                favorite INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                path TEXT NOT NULL,
                width INTEGER,
                height INTEGER,
                hash TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE
            )",
        )
        .execute(&self.pool)
        .await?;

        // Migration: add workspace column if missing
        let _ = sqlx::query(
            "ALTER TABLE notes ADD COLUMN workspace TEXT NOT NULL DEFAULT 'default'",
        )
        .execute(&self.pool)
        .await;

        // Migration: add type column if missing
        let _ = sqlx::query(
            "ALTER TABLE notes ADD COLUMN type TEXT NOT NULL DEFAULT ''",
        )
        .execute(&self.pool)
        .await;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS floating_windows (
                id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                type TEXT NOT NULL DEFAULT 'normal',
                title TEXT,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                width INTEGER NOT NULL DEFAULT 320,
                height INTEGER NOT NULL DEFAULT 240,
                minimized INTEGER NOT NULL DEFAULT 0,
                pinned INTEGER NOT NULL DEFAULT 0,
                auto_hide INTEGER NOT NULL DEFAULT 0,
                locked INTEGER NOT NULL DEFAULT 0,
                opacity REAL NOT NULL DEFAULT 1.0,
                monitor_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS workspaces (
                name TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        // Ensure default workspace exists
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT OR IGNORE INTO workspaces (name, created_at) VALUES ('default', ?)",
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS reminders (
                id TEXT PRIMARY KEY,
                note_id TEXT,
                title TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL DEFAULT '',
                remind_at INTEGER NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                snoozed INTEGER NOT NULL DEFAULT 0,
                deleted INTEGER NOT NULL DEFAULT 0,
                cancelled INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE SET NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        // Add missing columns to existing tables (migration)
        let _ = sqlx::query("ALTER TABLE reminders ADD COLUMN deleted INTEGER NOT NULL DEFAULT 0")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE reminders ADD COLUMN cancelled INTEGER NOT NULL DEFAULT 0")
            .execute(&self.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE notes ADD COLUMN color_ranges TEXT NOT NULL DEFAULT '[]'")
            .execute(&self.pool)
            .await;

        Ok(())
    }
}
