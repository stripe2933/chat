use actix_session::Session;
use actix_web::{Responder, HttpResponse, post};

use crate::api::user::User;

#[post("/logout")]
pub async fn handler(session: Session) -> impl Responder{
    User::expire_session(session);
    HttpResponse::SeeOther().append_header(("Location", "https://localhost:5173/#/login")).finish()
}