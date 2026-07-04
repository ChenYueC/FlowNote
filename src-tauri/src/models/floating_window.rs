use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FloatingWindow {
    pub id: String,
    pub note_id: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub window_type: String,
    pub title: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub minimized: i32,
    pub pinned: i32,
    pub auto_hide: i32,
    pub locked: i32,
    pub opacity: f64,
    pub monitor_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateFloatingWindowInput {
    pub note_id: String,
    pub x: i32,
    pub y: i32,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWindowStateInput {
    pub id: String,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub pinned: Option<i32>,
    pub auto_hide: Option<i32>,
    pub locked: Option<i32>,
    pub opacity: Option<f64>,
    pub minimized: Option<i32>,
}
