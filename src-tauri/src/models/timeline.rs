use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TimelineItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: i64,
    pub item_type: String,
    pub item_action: String,
    pub archived: i32,
}
