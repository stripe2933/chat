pub fn map_internal_error<T>(_: T) -> actix_web::Error {
    actix_web::error::ErrorInternalServerError("Internal Server Error")
}