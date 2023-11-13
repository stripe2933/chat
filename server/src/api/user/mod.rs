mod user;
use actix_web::web;
pub use user::User;

mod login;
mod logout;
mod register;
mod get_login_info;
mod get_all_users;
mod get_profile_picture;

pub fn config(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/user")
            .service(login::handler)
            .service(logout::handler)
            .service(register::handler)
            .service(get_login_info::handler)
            .service(get_all_users::handler)
            .service(get_profile_picture::handler)
    );
}