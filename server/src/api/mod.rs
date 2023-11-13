use actix_web::web;

pub mod user;
mod conversation;
mod map_internal_error;
pub(crate) mod message;

pub use map_internal_error::map_internal_error;

pub fn config(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/api")
            .configure(user::config)
            .configure(conversation::config)
    );
}