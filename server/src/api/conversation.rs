use actix_session::Session;
use actix_web::{get, Responder, web, HttpResponse, post};
use serde::Serialize;
use chrono::NaiveDateTime;

use crate::{AppState, api::user::User};

use super::message::Message;

#[derive(sqlx::FromRow, Serialize, Debug)]
struct Conversation{
    id: i64,
    name: String,
    created_at: NaiveDateTime,
    members: Vec<User>,
}

#[derive(Serialize, Debug)]
struct JoinedConversationResult{
    id: i64,
    name: String,
    created_at: NaiveDateTime,
    members: Vec<User>,
    last_sender_username: Option<String>,
    last_message_text: Option<String>,
    last_sent_at: Option<NaiveDateTime>
}

#[get("/joined")]
async fn joined_conversations(app_state: web::Data<AppState>, session: Session) -> impl Responder{
    match User::get_username_from_session(session){
        Some(username) => {
            let joined_conversations = sqlx::query!(
                "SELECT jc.id, jc.name, jc.created_at, msg.sender_username AS last_sender_username, msg.text AS last_message_text, MAX(msg.sent_at) AS last_message_sent_at
                FROM (SELECT c.id, c.name, c.created_at
                    FROM conversations c
                    INNER JOIN group_members gm ON c.id = gm.conversation_id
                    WHERE gm.username = ?) jc /* joined conversations */
                LEFT JOIN messages msg ON jc.id = msg.conversation_id
                GROUP BY jc.id;", username)
                    .fetch_all(&app_state.database)
                    .await.unwrap();
                
            let joined_conversations = joined_conversations.iter()
                .map(|joined_conversation| async {
                    // Members are sorted in ascending joined datetime order.
                    let members = sqlx::query_as!(User, "SELECT gm.username, users.nickname, users.profile_picture_filename FROM group_members gm
                            INNER JOIN users ON users.username = gm.username 
                            WHERE conversation_id = ?
                            ORDER BY gm.joined_at ASC;", joined_conversation.id)
                            .fetch_all(&app_state.database)
                            .await.unwrap();

                    // TODO: unnecessary clone.
                    JoinedConversationResult {
                        id: joined_conversation.id.unwrap(),
                        name: joined_conversation.name.clone().unwrap(),
                        created_at: joined_conversation.created_at.unwrap(),
                        members,
                        last_sender_username: joined_conversation.last_sender_username.clone(),
                        last_message_text: joined_conversation.last_message_text.clone(),
                        last_sent_at: joined_conversation.last_message_sent_at
                    }
                });

            let response: Vec<JoinedConversationResult> = futures::future::join_all(joined_conversations).await;
            HttpResponse::Ok().json(response)
        }
        None => HttpResponse::Unauthorized().finish()
    }
}

#[get("/{conversation_id}")]
async fn get_conversation(path: web::Path<i64>, app_state: web::Data<AppState>, session: Session) -> impl Responder{
    match User::get_username_from_session(session){
        Some(username) => {
            // Check if user joined to the given conversation.
            let conversation_id = path.into_inner();

            let is_user_joined = sqlx::query("SELECT 1 FROM group_members WHERE username = ? AND conversation_id = ?;")
                .bind(username)
                .bind(conversation_id)
                .fetch_optional(&app_state.database)
                .await.unwrap()
                .is_some();
            if !is_user_joined{
                return HttpResponse::Forbidden().body("You are not joined to this conversation.")
            }

            let conversation = sqlx::query!("SELECT id, name, created_at FROM conversations WHERE id = ?;", conversation_id)
                .fetch_optional(&app_state.database)
                .await.unwrap();

            match conversation{
                Some(conversation) => {
                    let members = sqlx::query_as!(User, 
                        "SELECT gm.username, users.nickname, users.profile_picture_filename 
                        FROM users
                        INNER JOIN group_members gm ON users.username = gm.username
                        WHERE gm.conversation_id = ?;", conversation.id)
                        .fetch_all(&app_state.database)
                        .await.unwrap();

                    HttpResponse::Ok().json(Conversation{
                        id: conversation.id,
                        name: conversation.name,
                        created_at: conversation.created_at,
                        members
                    })
                },
                None => HttpResponse::NotFound().finish()
            }

        }
        None => HttpResponse::Unauthorized().finish()
    }
}

#[get("/{conversation_id}/messages")]
async fn get_conversation_messages(path: web::Path<i64>, app_state: web::Data<AppState>, session: Session) -> impl Responder{
    match User::get_username_from_session(session){
        Some(username) => {
            // Check if user joined to the given conversation.
            let conversation_id = path.into_inner();

            let is_user_joined = sqlx::query("SELECT 1 FROM group_members WHERE username = ? AND conversation_id = ?;")
                .bind(username)
                .bind(conversation_id)
                .fetch_optional(&app_state.database)
                .await.unwrap()
                .is_some();
            if !is_user_joined{
                return HttpResponse::Forbidden().body("You are not joined to this conversation.")
            }

            let messages = sqlx::query_as!(Message, 
                    "SELECT id, sender_username, text, sent_at 
                    FROM messages 
                    WHERE conversation_id = ?
                    ORDER BY sent_at ASC;", conversation_id)
                .fetch_all(&app_state.database)
                .await.unwrap();

            HttpResponse::Ok().json(messages)
        }
        None => HttpResponse::Unauthorized().finish()
    }
}

#[post("/{conversation_id}/message")]
async fn post_conversation_message(path: web::Path<i64>, text: String, app_state: web::Data<AppState>, session: Session) -> impl Responder{
    match User::get_username_from_session(session){
        Some(username) => {
            // Check if user joined to the given conversation.
            let conversation_id = path.into_inner();

            let is_user_joined = sqlx::query("SELECT 1 FROM group_members WHERE username = ? AND conversation_id = ?;")
                .bind(&username)
                .bind(conversation_id)
                .fetch_optional(&app_state.database)
                .await.unwrap()
                .is_some();
            if !is_user_joined{
                return HttpResponse::Forbidden().body("You are not joined to this conversation.")
            }
            
            let message = sqlx::query_as!(Message, 
                    "INSERT INTO messages(sender_username, text, sent_at, conversation_id) VALUES (?, ?, datetime('now'), ?); 
                     SELECT id, sender_username, text, sent_at FROM messages WHERE id = last_insert_rowid();", username, text, conversation_id)
                .fetch_one(&app_state.database)
                .await.unwrap();
            HttpResponse::Ok().json(message)
        }
        None => HttpResponse::Unauthorized().finish()
    }
}

pub fn config(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/conversation")
            .service(joined_conversations)
            .service(get_conversation)
            .service(get_conversation_messages)
            .service(post_conversation_message)
    );
}