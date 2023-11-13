use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct Message{
    pub id: i64,
    pub sender_username: String,
    pub text: String,
    pub sent_at: NaiveDateTime
}