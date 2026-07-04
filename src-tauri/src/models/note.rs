use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub workspace: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub archived: i32,
    pub favorite: i32,
    #[sqlx(rename = "type")]
    pub r#type: String,
    pub color_ranges: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteInput {
    pub title: String,
    pub content: String,
    pub workspace: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteInput {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub color_ranges: Option<String>,
}
