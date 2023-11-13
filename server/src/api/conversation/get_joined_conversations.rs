/*
 * Get session user's joined conversations.
 * 
 * Request:
 * POST /api/conversation/joined
 * 
 * Response:
 * HTTP 200 OK
 * [
 *     {
 *         "id": 1,
 *         "name": "Conversation 1",
 *         "members": [
 *             {
 *                 "username": "user1",
 *                 "nickname": "User 1",
 *                 "profile_picture_filename": "user1.png"
 *             },
 *             {
 *                 "username": "user2",
 *                 "nickname": "User 2",
 *                 "profile_picture_filename": "user2.png"
 *             },
 *             ...
 *         ],
 *         "last_message": {
 *             "id": 1,
 *             "sender_username": "user1",
 *             "text": "Hello!",
 *             "sent_at": "2021-01-01 00:00:00"
 *         }
 *     },
 *     ...
 * ]
 */

use actix_session::Session;
use actix_web::{get, web, Responder, HttpResponse, Error};
use serde::{Serialize, Deserialize};
use sqlx::types::Json;

use crate::{api::{user::User, map_internal_error}, AppState};

#[derive(Serialize, Deserialize, Debug)]
struct Message{
    id: i64,
    sender_username: String,
    text: String,
    sent_at: String
}

#[derive(sqlx::FromRow, Serialize, Debug)]
struct JoinedConversation{
    id: i64,
    name: String,
    members: Json<Vec<User>>,
    last_message: Option<Json<Message>>
}

#[get("/joined")]
pub async fn handler(app_state: web::Data<AppState>, session: Session) -> Result<impl Responder, Error>{
    let username = match User::get_username_from_session(session){
        Some(username) => username,
        None => return Ok(HttpResponse::Unauthorized().finish())
    };

    let result = sqlx::query_as::<_, JoinedConversation>(
        "DROP TABLE IF EXISTS joined_conversations;
        
        CREATE TEMP TABLE joined_conversations AS
            SELECT conversations.id, conversations.name
            FROM conversations
            INNER JOIN
                (SELECT conversation_id 
                FROM group_members
                WHERE username = $1) gm
            ON conversations.id = gm.conversation_id;

        DROP TABLE IF EXISTS last_messages_by_conversations;
        
        CREATE TEMP TABLE last_messages_by_conversations AS
            SELECT jc.id AS conversation_id, json_object('id', messages.id, 'sender_username', sender_username, 'text', text, 'sent_at', MAX(sent_at)) AS message
            FROM messages
            INNER JOIN joined_conversations jc ON messages.conversation_id = jc.id
            GROUP BY jc.id;

        DROP TABLE IF EXISTS joined_members;
            
        CREATE TEMP TABLE joined_members AS
            SELECT jm.conversation_id, users.username, users.nickname, users.profile_picture_filename, jm.joined_at
            FROM users
            INNER JOIN
                (SELECT jc.id AS conversation_id, gm.username, gm.joined_at
                FROM group_members gm
                INNER JOIN joined_conversations jc
                ON gm.conversation_id = jc.id) jm
            USING (username)
            WHERE users.username != $1;
            
        SELECT id, name, members, lmbc.message AS last_message
        FROM
            (SELECT jc.id, 
                    jc.name, 
                    json_group_array(json_object('username', jmj.username, 'nickname', jmj.nickname, 'profile_picture_filename', jmj.profile_picture_filename)) AS members
            FROM joined_conversations AS jc
            INNER JOIN 
                (SELECT *
                FROM joined_members
                ORDER BY joined_at ASC) AS jmj -- joined_members_json
            ON jc.id = jmj.conversation_id
            GROUP BY jc.id)
        LEFT JOIN last_messages_by_conversations AS lmbc ON id = lmbc.conversation_id; -- last message may not exists: use left join")
        .bind(username)
        .fetch_all(&app_state.database)
        .await.map_err(map_internal_error)?;

    Ok(HttpResponse::Ok().json(result))
}