use actix_session::Session;
use actix_web::{web, Responder, HttpResponse, Error, post};

use crate::{AppState, api::{user::User, conversation::is_user_joined_in_conversation, map_internal_error}};
use crate::api::message::Message;

#[post("/{conversation_id}/message")]
pub async fn handler(path: web::Path<i64>, text: String, app_state: web::Data<AppState>, session: Session) -> Result<impl Responder, Error>{
    // VALIDATION: User must be logged in.
    let username = match User::get_username_from_session(session){
        Some(username) => username,
        None => return Ok(HttpResponse::Unauthorized().finish())
    };

    // VALIDATION: Check if user joined to the given conversation.
    let conversation_id = path.into_inner();
    if !is_user_joined_in_conversation(&app_state.database, &username, conversation_id)
        .await.map_err(map_internal_error)?{
        return Ok(HttpResponse::Forbidden().body("You are not joined to this conversation."))
    }
    
    let message = sqlx::query_as!(Message, 
            "INSERT INTO messages(sender_username, text, sent_at, conversation_id) VALUES (?, ?, DATETIME('NOW'), ?)
            RETURNING id, sender_username, text, sent_at;", username, text, conversation_id)
        .fetch_one(&app_state.database)
        .await.unwrap();
    Ok(HttpResponse::Ok().json(message))
}