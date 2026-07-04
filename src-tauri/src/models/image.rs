use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Image {
    pub id: String,
    pub note_id: String,
    pub path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub hash: Option<String>,
    pub created_at: i64,
}
