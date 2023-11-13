use actix_session::Session;
use actix_web::{get, web, Responder, HttpResponse, Error};
use serde::Serialize;

use crate::{AppState, api::{user::User, map_internal_error}};

#[derive(sqlx::FromRow, Serialize, Debug)]
struct Conversation{
    id: i64,
    name: String,
    members: Vec<User>,
}

#[get("/{conversation_id}")]
async fn handler(path: web::Path<i64>, app_state: web::Data<AppState>, session: Session) -> Result<impl Responder, Error>{
    // VALIDATION: User must be logged in.
    let username = match User::get_username_from_session(session){
        Some(username) => username,
        None => return Ok(HttpResponse::Unauthorized().finish())
    };

    // VALIDATION: Check if user joined to the given conversation.
    let conversation_id = path.into_inner();

    let is_user_joined = sqlx::query!("SELECT 1 AS x 
        FROM group_members 
        WHERE username = ? AND conversation_id = ?;", username, conversation_id)
        .fetch_optional(&app_state.database)
        .await.map_err(map_internal_error)?
        .is_some();
    if !is_user_joined{
        return Ok(HttpResponse::Forbidden().body("You are not joined to this conversation."))
    }

    let conversation = sqlx::query!("SELECT id, name FROM conversations WHERE id = ?;", conversation_id)
        .fetch_optional(&app_state.database)
        .await.map_err(map_internal_error)?;

    match conversation{
        Some(conversation) => {
            let members = sqlx::query_as!(User, 
                "SELECT gm.username, users.nickname, users.profile_picture_filename 
                FROM users
                INNER JOIN group_members gm USING (username)
                WHERE gm.conversation_id = ?;", conversation.id)
                .fetch_all(&app_state.database)
                .await.map_err(map_internal_error)?;

            Ok(HttpResponse::Ok().json(Conversation{
                id: conversation.id,
                name: conversation.name,
                members
            }))
        },
        None => Ok(HttpResponse::NotFound().finish())
    }
}