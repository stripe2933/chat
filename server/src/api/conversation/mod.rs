use actix_web::web;
use sqlx::SqlitePool;

mod get_joined_conversations;
mod create_new_conversation;
mod get_conversation;
mod get_conversation_messages;
mod send_conversation_message;

async fn is_user_joined_in_conversation(database: &SqlitePool, username: &str, conversation_id: i64) -> Result<bool, sqlx::Error>{
    Ok(sqlx::query!("SELECT 1 AS x 
        FROM group_members 
        WHERE username = ? AND conversation_id = ?;", username, conversation_id)
        .fetch_optional(database)
        .await?
        .is_some())
}

pub fn config(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/conversation")
            .service(get_joined_conversations::handler)
            .service(create_new_conversation::handler)
            .service(get_conversation::handler)
            .service(get_conversation_messages::handler)
            .service(send_conversation_message::handler)
    );
}