use std::{path::Path, ffi::OsStr};

use actix_multipart::form::{MultipartForm, text::Text, tempfile::TempFile};
use actix_web::{web, Responder, HttpResponse, post, Error};

use crate::AppState;

use super::User;

#[derive(MultipartForm, Debug)]
pub struct Form{
    username: Text<String>,
    password: Text<String>,
    nickname: Text<String>,
    profile: Option<TempFile>
}

#[post("/register")]
pub async fn handler(MultipartForm(form): MultipartForm<Form>, app_state: web::Data<AppState>) -> Result<impl Responder, Error>{
    // Check constraints for username and password.
    if let Err(err) = User::check_username_constraint(&form.username){
        return Ok(HttpResponse::BadRequest().body(err.to_string()));
    }
    if let Err(err) = User::check_password_constraint(&form.password){
        return Ok(HttpResponse::BadRequest().body(err.to_string()));
    }

    let encrypted_password = User::encrypt_password(&form.password);

    // Persist user profile into file (if given). If not given, use default profile image.
    let img_filename = match form.profile{
        Some(image) => {
            // Check if mime is image/*.
            if image.content_type.is_none() || image.content_type.as_ref().unwrap().type_() != mime::IMAGE{
                return Ok(HttpResponse::BadRequest().body("Only image file are allowed."))
            }
        
            // Check if image file size exceeds the limit.
            const MAX_FILE_SIZE: usize = 1024 * 1024 * 10; // 10MB
            if image.size > MAX_FILE_SIZE{
                return Ok(HttpResponse::BadRequest().body("Too large image file. Use less than 10 MB file."))
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

    let result = sqlx::query!("INSERT INTO users VALUES (?, ?, ?, ?, DATETIME('NOW'));", form.username.0, encrypted_password, form.nickname.0, img_filename)
        .execute(&app_state.database)
        .await;
    
    match result{
        Ok(_) => Ok(HttpResponse::SeeOther().append_header(("Location", "https://localhost:5173/#/login")).finish()),
        Err(err) => {
            // Remove the saved profile picture.
            if let Some(filename) = img_filename{
                std::fs::remove_file(format!("resources/images/profiles/{}", filename))?;
            }

            match err{
                sqlx::Error::Database(err) if err.kind() == sqlx::error::ErrorKind::UniqueViolation => {
                    Ok(HttpResponse::BadRequest().body("Username already exists."))
                },
                _ => Ok(HttpResponse::InternalServerError().body("Internal server error."))
            }
        }
    }
}