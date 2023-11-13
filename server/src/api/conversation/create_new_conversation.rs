use actix_session::Session;
use actix_web::{post, web, Responder, HttpResponse, Error};
use futures::future::join_all;
use serde::Deserialize;
use sqlx_core::any::AnyConnectionBackend;

use crate::{AppState, api::{map_internal_error, user::User}};

#[derive(Deserialize, Debug)]
pub struct Request{
    conversation_name: String,
    members: Vec<String>
}

#[post("/")]
pub async fn handler(request: web::Json<Request>, app_state: web::Data<AppState>, session: Session) -> Result<impl Responder, Error>{
    // VALIDATION: User must be logged in.
    let username = match User::get_username_from_session(session){
        Some(username) => username,
        None => return Ok(HttpResponse::Unauthorized().finish())
    };

    // VALIDATION: Check if user is in the members list.
    if !request.members.contains(&username){
        return Ok(HttpResponse::Forbidden().body("You must be in the members list."));
    }

    // VALIDATION: User cannot create a conversation with only himself.
    if request.members.len() == 1{
        return Ok(HttpResponse::BadRequest().body("You cannot create a conversation with only yourself."));
    }

    // TRANSACTION START.
    let mut tx = app_state.database.acquire()
        .await
        .map_err(map_internal_error)?;
    
    // Create new conversation from conversations table.
    let conversation_id = sqlx::query!("INSERT INTO conversations (name, created_at) VALUES (?, DATETIME('NOW')) RETURNING id;", request.conversation_name)
        .fetch_one(&app_state.database)
        .await
        .map_err(map_internal_error)?
        .id;

    // Add conversation members into group_members table.
    let query_futures = request.members.iter().map(|member_username| {
        async {
            sqlx::query!("INSERT INTO group_members (username, conversation_id, joined_at) VALUES (?, ?, DATETIME('NOW'));", *member_username, conversation_id)
                .execute(&app_state.database)
                .await
        }
    });

    for result in join_all(query_futures).await{
        match result{
            Ok(_) => {},
            Err(err) => {
                // Rollback the transaction.
                tx.rollback().await.map_err(map_internal_error)?;

                match err{
                    sqlx::Error::Database(err) if err.kind() == sqlx::error::ErrorKind::ForeignKeyViolation => {
                        return Ok(HttpResponse::BadRequest().body("The request contains inexisting conversation member."))
                    },
                    _ => return Ok(HttpResponse::InternalServerError().body("Internal server error."))
                }
            }
        }
    }

    tx.commit().await.map_err(map_internal_error)?;
    // TRANSACTION END.

    Ok(HttpResponse::Ok().finish())
}