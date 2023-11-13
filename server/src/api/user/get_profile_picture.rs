use actix_files::NamedFile;
use actix_web::{web, Responder, get};

#[get("/profile_picture/{filename}")]
async fn handler(path: web::Path<String>) -> impl Responder{
    // Anyone can access to specific people's profile picture, for now.
    // TODO: why not use PathBuf?
    let filename = path.into_inner();
    NamedFile::open_async(format!("resources/images/profiles/{}", filename)).await
}