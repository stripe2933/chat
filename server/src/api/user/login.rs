use actix_session::Session;
use actix_web::{web, Responder, HttpResponse, post, Error};
use serde::Deserialize;

use crate::{AppState, api::map_internal_error};

use super::User;

#[derive(Deserialize, Debug)]
struct Form{
    username: String,
    password: String,
}

#[post("/login")]
async fn handler(form: web::Form<Form>, app_state: web::Data<AppState>, session: Session) -> Result<impl Responder, Error>{
    let encrypted_password = User::encrypt_password(&form.password);
    let user = sqlx::query_as!(User, "SELECT username, nickname, profile_picture_filename FROM users WHERE username=? AND encrypted_password=?", form.username, encrypted_password)
        .fetch_optional(&app_state.database)
        .await.map_err(map_internal_error)?;

    match user{
        Some(user) => {
            // Generate session key for the user.
            user.add_username_into_session(session);
            Ok(HttpResponse::SeeOther().append_header(("Location", "https://localhost:5173/#/")).finish())
        }
        _ => Ok(HttpResponse::Unauthorized().body("Wrong username or password."))
    }
}