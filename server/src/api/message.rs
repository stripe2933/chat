use actix_web::web;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct Message{
    pub id: i64,
    pub sender_username: String,
    pub text: String,
    pub sent_at: NaiveDateTime
}