use actix_web::web;

pub mod user;
mod conversation;
pub(crate) mod message;

pub fn config(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/api")
            .configure(user::config)
            .configure(conversation::config)
    );
}