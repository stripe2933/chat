use actix_web::{get, Responder, web, HttpResponse, Error};

use crate::{AppState, api::{user::User, map_internal_error}};

#[get("/all")]
pub async fn handler(app_state: web::Data<AppState>) -> Result<impl Responder, Error>{
    let users = sqlx::query_as!(User, "SELECT username, nickname, profile_picture_filename FROM users")
        .fetch_all(&app_state.database)
        .await.map_err(map_internal_error)?;
    Ok(HttpResponse::Ok().json(users))
}