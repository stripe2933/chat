use std::{ffi::OsStr, path::Path};

use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_session::Session;
use actix_web::{web::{self}, post, HttpResponse, Responder, get};
use serde::Deserialize;

use crate::AppState;

mod user;
pub use user::User;

#[derive(Deserialize, Debug)]
struct LoginForm{
    username: String,
    password: String,
}

#[post("/login")]
async fn login(form: web::Form<LoginForm>, app_state: web::Data<AppState>, session: Session) -> impl Responder{
    let encrypted_password = User::encrypt_password(&form.password);
    let user = sqlx::query_as!(User, "SELECT username, nickname, profile_picture_filename FROM users WHERE username=? AND encrypted_password=?", form.username, encrypted_password)
        .fetch_one(&app_state.database)
        .await;

    match user{
        Ok(user) => {
            // Generate session key for the user.
            user.add_username_into_session(session);
            HttpResponse::SeeOther().append_header(("Location", "https://localhost:5173/#/")).finish()
        }
        _ => HttpResponse::Unauthorized().body("Wrong username or password.")
    }
}

#[post("/logout")]
async fn logout(session: Session) -> impl Responder{
    User::expire_session(session);
    HttpResponse::SeeOther().append_header(("Location", "https://localhost:5173/#/login")).finish()
}

#[derive(MultipartForm, Debug)]
struct RegisterForm{
    username: Text<String>,
    password: Text<String>,
    nickname: Text<String>,
    profile: Option<TempFile>
}

#[post("/register")]
async fn register(MultipartForm(form): MultipartForm<RegisterForm>, app_state: web::Data<AppState>) -> impl Responder{
    // Check constraints for username and password.
    if let Err(err) = User::check_username_constraint(&form.username){
        return HttpResponse::BadRequest().body(err.to_string());
    }
    if let Err(err) = User::check_password_constraint(&form.password){
        return HttpResponse::BadRequest().body(err.to_string());
    }

    let encrypted_password = User::encrypt_password(&form.password);

    // Persist user profile into file (if given). If not given, use default profile image.
    let img_filename = match form.profile{
        Some(image) => {
            // Check if mime is image/*.
            if image.content_type.is_none() || image.content_type.as_ref().unwrap().type_() != mime::IMAGE{
                return HttpResponse::BadRequest().body("Only image file are allowed.")
            }
        
            // Check if image file size exceeds the limit.
            const MAX_FILE_SIZE: usize = 1024 * 1024 * 10; // 10MB
            if image.size > MAX_FILE_SIZE{
                return HttpResponse::BadRequest().body("Too large image file. Use less than 10 MB file.")
            }
        
            // Create new uuid for filename.
            let file_basename = uuid::Uuid::new_v4().to_string();
            let file_extension = image.file_name.as_ref()
                .map(|filename| {
                    Path::new(filename)
                        .extension()
                        .and_then(OsStr::to_str)
                        .map(|ext| format!(".{}", ext))
                        .unwrap_or("".to_owned())
                }).unwrap_or("".to_owned()); // ".(ext)" format if success, empty string if failed.
            
            let img_filename = format!("{}{}", file_basename, file_extension);
            image.file.persist(format!("resources/images/profiles/{}", img_filename)).unwrap();

            Some(img_filename)
        }
        None => None
    };

    let final_filename = img_filename.clone().unwrap_or("default-profile.png".to_owned()); // filename to store in the database.
    let result = sqlx::query!("INSERT INTO users VALUES (?, ?, ?, ?);", form.username.0, encrypted_password, form.nickname.0, final_filename)
        .execute(&app_state.database)
        .await;
    
    match result{
        Ok(_) => HttpResponse::SeeOther().append_header(("Location", "https://localhost:5173/#/login")).finish(),
        Err(err) => {
            // Remove the saved profile picture.
            if let Some(filename) = img_filename{
                std::fs::remove_file(format!("resources/images/profiles/{}", filename)).unwrap();
            }

            match err{
                sqlx::Error::Database(err) if err.kind() == sqlx::error::ErrorKind::UniqueViolation => {
                    HttpResponse::BadRequest().body("Username already exists.")
                },
                _ => HttpResponse::InternalServerError().body("Internal server error.")
            }
        }
    }
}

#[get("/login_info")]
async fn get_login_info(session: Session, app_state: web::Data<AppState>) -> impl Responder{
    match User::get_username_from_session(session){
        Some(username) => {
            let user = sqlx::query_as!(User, "SELECT username, nickname, profile_picture_filename FROM users WHERE username=?", username)
                .fetch_one(&app_state.database)
                .await.unwrap();
            HttpResponse::Ok().json(user)
        }
        None => HttpResponse::Unauthorized().finish()
    }
}

#[get("/profile_picture/{filename}")]
async fn profile_picture(filename: web::Path<String>) -> impl Responder{
    // Anyone can access to specific people's profile picture, for now.
    let path = format!("resources/images/profiles/{}", filename);
    match std::fs::read(path){
        Ok(bytes) => HttpResponse::Ok().body(bytes),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

#[get("/all")]
async fn all_users(app_state: web::Data<AppState>) -> impl Responder{
    let users = sqlx::query_as!(User, "SELECT username, nickname, profile_picture_filename FROM users")
        .fetch_all(&app_state.database)
        .await.unwrap();
    HttpResponse::Ok().json(users)
}

pub fn config(cfg: &mut web::ServiceConfig){
    cfg.service(
        web::scope("/user")
            .service(login)
            .service(logout)
            .service(register)
            .service(get_login_info)
            .service(all_users)
            .service(profile_picture)
    );
}