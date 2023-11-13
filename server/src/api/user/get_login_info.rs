use actix_session::Session;
use actix_web::{get, Responder, web, HttpResponse, Error};

use crate::{AppState, api::{user::User, map_internal_error}};

#[get("/login_info")]
pub async fn handler(session: Session, app_state: web::Data<AppState>) -> Result<impl Responder, Error>{
    let username = match User::get_username_from_session(session){
        Some(username) => username,
        None => return Ok(HttpResponse::Unauthorized().finish())
    };

    let user = sqlx::query_as!(User, "SELECT username, nickname, profile_picture_filename FROM users WHERE username=?", username)
        .fetch_one(&app_state.database)
        .await.map_err(map_internal_error)?;
    Ok(HttpResponse::Ok().json(user))
}