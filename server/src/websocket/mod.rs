use std::time::Instant;

use actix_session::Session;
use actix_web::{web, HttpRequest, Responder, get, HttpResponse};
use actix_web_actors::ws;

use crate::{AppState, api::user::User};

pub mod server;
pub mod session;

/// Entry point for our websocket route
#[get("/")]
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
    session: Session
) -> impl Responder {
    // Check if user logged in.
    match User::get_username_from_session(session){
        Some(username) => {
            ws::start(
                session::WsChatSession {
                    id: 0,
                    hb: Instant::now(),
                    conversation_id: 0,
                    username: username,
                    // server_address: app_state.get_ref().websocket_server.clone(),
                    app_state: app_state.clone()
                },
                &req,
                stream,
            )
        }
        None => Ok(HttpResponse::Unauthorized().finish())
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    // cfg.route("/ws", web::get().to(chat_route));
    cfg.service(
        web::scope("/ws")
            .service(chat_route)
    );
}
