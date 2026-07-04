use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Reminder {
    pub id: String,
    pub note_id: Option<String>,
    pub title: String,
    pub content: String,
    pub remind_at: i64,
    pub completed: i32,
    pub snoozed: i32,
    pub deleted: i32,
    pub cancelled: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderListResponse {
    pub reminders: Vec<Reminder>,
    pub counts: ReminderCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderCounts {
    pub all: i64,
    pub today: i64,
    pub overdue: i64,
    pub cancelled: i64,
    pub completed: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateReminderInput {
    pub title: String,
    pub note_id: Option<String>,
    pub content: Option<String>,
    pub remind_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReminderInput {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub remind_at: Option<i64>,
    pub completed: Option<i32>,
    pub snoozed: Option<i32>,
}
