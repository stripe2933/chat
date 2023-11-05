use std::{fs::File, io::BufReader};

use actix::Actor;
use actix_cors::Cors;
use actix_session::{SessionMiddleware, storage::RedisActorSessionStore};
use actix_web::{
    middleware, App, HttpServer, web, http::{self},
};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use sqlx::{SqlitePool, Sqlite, migrate::MigrateDatabase};

use crate::websocket::server;

mod websocket;
mod api;

#[derive(Debug)]
pub struct AppState{
    pub database: SqlitePool,
    pub websocket_server: actix::Addr<server::ChatServer>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    // Initialize logger.
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Configure global app state.
    let app_state = web::Data::new(AppState {
        database: prepare_database().await,
        websocket_server: server::ChatServer::new().start()
    });

    // Configure HTTP2 TLS connection.
    let config = load_rustls_config();

    // Private key for redis.
    let redis_private_key = actix_web::cookie::Key::generate();

    log::info!("Starting HTTPS server at https://localhost:8443");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default()) // enable logger.
            .wrap(
                Cors::default() // <- Construct CORS middleware builder
                    .allowed_origin("https://localhost:5173")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600))
            .wrap(
                SessionMiddleware::builder(
                    RedisActorSessionStore::new("127.0.0.1:6379"),
                    redis_private_key.clone()
                ).build())
            .configure(api::config)
            .configure(websocket::config)
            // .service(web::route("/ws").to(websocket::chat_route))
    });

    // Use different port num and workers for build target.
    {
        if cfg!(debug_assertions){ // Debug mode.
            server.bind_rustls_021("localhost:8443", config)?
                .workers(1) 
        }
        else{ // Release mode.
            server.bind_rustls_021("0.0.0.0:8443", config)?
        }
    }
    .run()
    .await
}

async fn prepare_database() -> SqlitePool{
    const DATABASE_URL: &str = "database.db";

    // If database file not exists, create it.
    if !Sqlite::database_exists(DATABASE_URL).await.unwrap_or(false){
        Sqlite::create_database(DATABASE_URL).await.unwrap();
    }

    SqlitePool::connect(DATABASE_URL).await.unwrap()
}

fn load_rustls_config() -> rustls::ServerConfig {
    // See https://github.com/actix/examples/tree/master/https-tls/rustls

    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(File::open("../cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("../key.pem").unwrap());

    // convert files to key/cert objects
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}